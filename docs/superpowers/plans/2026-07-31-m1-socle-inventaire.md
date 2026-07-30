# Plan d'implémentation — M1 « Socle & Inventaire » (BetterUnistaller)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Livrer le socle de BetterUnistaller : application Tauri 2 + React qui inventorie tous les programmes Win32 installés (F1) avec recherche, tri, filtres (F2), tailles et icônes chargées en arrière-plan, et un panneau de détail.

**Architecture:** Backend Rust découpé en modules à responsabilité unique (`inventory/parse` pur et testé, `inventory/registry` accès winreg, `inventory/size` et `inventory/icon` en tâches de fond émettant des événements). Le frontend React ne touche jamais le système : il appelle des commandes typées (`list_programs`, `compute_sizes`, `load_icons`) et écoute les événements `size-computed` / `icon-ready`. Toute la logique frontend testable (normalisation de recherche, filtres, formatage) vit dans `src/lib/` hors composants.

**Tech Stack:** Tauri 2.x, Rust (winreg, windows, image, base64), React 18 + TypeScript + Vite, Zustand, Tailwind CSS v4, Vitest.

**Référence produit :** `docs/cahier-des-charges.md` (sections F1, F2, §4, §5). Les jalons M2–M5 auront chacun leur propre plan.

## Global Constraints

- Windows 10 21H2+ et Windows 11 ; x64 et ARM64.
- Aucune élévation admin en M1 : uniquement des lectures (registre HKLM/HKCU, tailles de dossiers).
- Liste affichée < 2 s pour 200 programmes ; travail lourd (tailles, icônes) hors du thread UI, résultats poussés par événements.
- Aucune connexion réseau, aucune télémétrie.
- Libellés UI en français, centralisés dans `src/lib/strings.fr.ts` (jamais de texte en dur dans les composants) pour préparer l'i18n de M5.
- Entrées registre sans `DisplayName` ignorées ; `SystemComponent=1`, mises à jour (`ParentKeyName`/`ReleaseType`) masquées par défaut (visibles via un interrupteur).
- Nom du produit partout : `BetterUnistaller`.

---

### Task 1 : Échafaudage du projet Tauri + outillage de test

**Files:**
- Create: projet généré à la racine `D:\BetterUnistaller` (scaffold `create-tauri-app`, template react-ts)
- Modify: `src-tauri/tauri.conf.json`, `package.json`, `vite.config.ts`, `src/App.css`
- Create: `.gitignore` (généré), dépôt git

**Interfaces:**
- Consumes: rien (départ à vide — seul `docs/` existe).
- Produces: projet compilable ; scripts `npm run tauri dev`, `npm test` (Vitest), `cargo test` dans `src-tauri` ; Tailwind opérationnel.

- [ ] **Step 1: Générer le squelette**

Le dossier courant contient déjà `docs/` ; générer dans un sous-dossier temporaire puis remonter les fichiers :

```powershell
cd D:\BetterUnistaller
npm create tauri-app@latest tmp-scaffold -- --template react-ts --manager npm --identifier com.betterunistaller.app --yes
Get-ChildItem -Force tmp-scaffold | Move-Item -Destination . 
Remove-Item tmp-scaffold
npm install
```

Dans `src-tauri/tauri.conf.json`, vérifier/mettre : `"productName": "BetterUnistaller"`, `"identifier": "com.betterunistaller.app"`, et dans `app.windows[0]` : `"title": "BetterUnistaller"`, `"width": 1100`, `"height": 720`, `"minWidth": 860`, `"minHeight": 560`.

- [ ] **Step 2: Vérifier que l'app de base se lance**

Run: `npm run tauri dev` (fermer la fenêtre après vérification)
Expected: la fenêtre « BetterUnistaller » s'ouvre avec le template Tauri+React par défaut, sans erreur console.

- [ ] **Step 3: Installer Tailwind v4, Zustand, Vitest**

```powershell
npm install zustand @tauri-apps/plugin-opener
npm install -D tailwindcss @tailwindcss/vite vitest
```

`vite.config.ts` — ajouter le plugin Tailwind et la config de test :

```ts
/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  test: { environment: "node", include: ["src/**/*.test.ts"] },
});
```

Remplacer le contenu de `src/App.css` par la seule ligne :

```css
@import "tailwindcss";
```

Ajouter dans `package.json` → `"scripts"` : `"test": "vitest run --passWithNoTests"`.

- [ ] **Step 4: Vérifier l'outillage**

Run: `npm test`
Expected: Vitest se lance et sort en code 0 (« No test files found », toléré par `--passWithNoTests`).

Run: `cd src-tauri; cargo check; cd ..`
Expected: compilation sans erreur.

- [ ] **Step 5: Initialiser git et committer**

```powershell
git init -b main
git add -A
git commit -m "chore: scaffold Tauri 2 + React TS, Tailwind, Vitest"
```

---

### Task 2 : Modèle `Program` et logique pure de parsing (Rust, TDD)

**Files:**
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/inventory/mod.rs`
- Create: `src-tauri/src/inventory/parse.rs`
- Modify: `src-tauri/src/lib.rs` (déclarer les modules)
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: rien.
- Produces: `models::Program` (struct sérialisée camelCase, voir Step 4), `models::Scope` (`Machine`/`User`), `parse::parse_install_date(&str) -> Option<String>`, `parse::estimated_size_to_bytes(u32) -> u64`, `parse::EntryVisibility` + `parse::classify_visibility(...)`, `parse::clean_icon_path(&str) -> Option<String>`.

- [ ] **Step 1: Ajouter les dépendances Rust**

Dans `src-tauri/Cargo.toml`, section `[dependencies]` (serde/serde_json déjà présents via le scaffold) :

```toml
winreg = "0.55"
```

- [ ] **Step 2: Écrire les tests qui échouent (`parse.rs`)**

```rust
// src-tauri/src/inventory/parse.rs — en bas du fichier
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_install_date_yyyymmdd() {
        assert_eq!(parse_install_date("20260731"), Some("2026-07-31".into()));
    }

    #[test]
    fn parse_install_date_rejects_garbage() {
        assert_eq!(parse_install_date(""), None);
        assert_eq!(parse_install_date("2026"), None);
        assert_eq!(parse_install_date("2026-07-31"), None); // déjà formatée = inattendu
        assert_eq!(parse_install_date("abcdefgh"), None);
        assert_eq!(parse_install_date("20261332"), None); // mois 13, jour 32
    }

    #[test]
    fn estimated_size_is_kb_to_bytes() {
        // EstimatedSize est en Kio dans le registre
        assert_eq!(estimated_size_to_bytes(1024), 1_048_576);
        assert_eq!(estimated_size_to_bytes(0), 0);
    }

    #[test]
    fn entry_without_display_name_is_skipped() {
        let v = classify_visibility(None, 0, None, None);
        assert_eq!(v, EntryVisibility::Skip);
    }

    #[test]
    fn system_component_is_hidden() {
        let v = classify_visibility(Some("Runtime X"), 1, None, None);
        assert_eq!(v, EntryVisibility::Hidden);
    }

    #[test]
    fn windows_update_entries_are_hidden() {
        assert_eq!(classify_visibility(Some("KB123"), 0, Some("OperatingSystem"), None), EntryVisibility::Hidden);
        assert_eq!(classify_visibility(Some("Hotfix"), 0, None, Some("Security Update")), EntryVisibility::Hidden);
        assert_eq!(classify_visibility(Some("Hotfix"), 0, None, Some("Update Rollup")), EntryVisibility::Hidden);
    }

    #[test]
    fn normal_program_is_visible() {
        assert_eq!(classify_visibility(Some("7-Zip"), 0, None, None), EntryVisibility::Visible);
    }

    #[test]
    fn clean_icon_path_strips_quotes_and_index() {
        assert_eq!(
            clean_icon_path(r#""C:\Apps\x\app.exe",0"#),
            Some(r"C:\Apps\x\app.exe".into())
        );
        assert_eq!(clean_icon_path(r"C:\Apps\x\app.ico"), Some(r"C:\Apps\x\app.ico".into()));
        assert_eq!(clean_icon_path(""), None);
    }
}
```

- [ ] **Step 3: Vérifier l'échec**

Run: `cd src-tauri; cargo test`
Expected: FAIL — fonctions non définies.

- [ ] **Step 4: Implémenter `models.rs` et `parse.rs`**

```rust
// src-tauri/src/models.rs
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    Machine,
    User,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    /// Chemin registre complet de la clé — sert d'identifiant unique côté frontend.
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    /// Format ISO "YYYY-MM-DD".
    pub install_date: Option<String>,
    pub estimated_size_bytes: Option<u64>,
    pub install_location: Option<String>,
    pub uninstall_string: Option<String>,
    pub quiet_uninstall_string: Option<String>,
    /// Chemin d'icône nettoyé (exe ou ico), à résoudre par inventory::icon.
    pub display_icon: Option<String>,
    pub scope: Scope,
    /// true = masqué par défaut dans l'UI (composant système / mise à jour).
    pub is_system_entry: bool,
}
```

```rust
// src-tauri/src/inventory/parse.rs
#[derive(Debug, PartialEq, Eq)]
pub enum EntryVisibility {
    /// Pas un programme (pas de DisplayName) : ne pas retourner du tout.
    Skip,
    /// Programme système / mise à jour : retourné avec is_system_entry = true.
    Hidden,
    Visible,
}

/// InstallDate registre "YYYYMMDD" -> ISO "YYYY-MM-DD".
pub fn parse_install_date(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.len() != 8 || !raw.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let month: u32 = raw[4..6].parse().ok()?;
    let day: u32 = raw[6..8].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(format!("{}-{}-{}", &raw[0..4], &raw[4..6], &raw[6..8]))
}

/// EstimatedSize (REG_DWORD) est exprimé en Kio.
pub fn estimated_size_to_bytes(kib: u32) -> u64 {
    (kib as u64) * 1024
}

pub fn classify_visibility(
    display_name: Option<&str>,
    system_component: u32,
    parent_key_name: Option<&str>,
    release_type: Option<&str>,
) -> EntryVisibility {
    match display_name {
        None | Some("") => return EntryVisibility::Skip,
        _ => {}
    }
    if system_component == 1 || parent_key_name.is_some() {
        return EntryVisibility::Hidden;
    }
    if let Some(rt) = release_type {
        if rt.to_ascii_lowercase().contains("update") {
            return EntryVisibility::Hidden;
        }
    }
    EntryVisibility::Visible
}

/// DisplayIcon: `"C:\...\app.exe",0` -> `C:\...\app.exe`
pub fn clean_icon_path(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let no_index = match raw.rfind(',') {
        Some(pos) if raw[pos + 1..].trim().parse::<i32>().is_ok() => &raw[..pos],
        _ => raw,
    };
    let cleaned = no_index.trim().trim_matches('"').trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}
```

```rust
// src-tauri/src/inventory/mod.rs
pub mod parse;
```

Dans `src-tauri/src/lib.rs`, ajouter en tête (avant `run()`) :

```rust
pub mod inventory;
pub mod models;
```

- [ ] **Step 5: Vérifier le passage**

Run: `cd src-tauri; cargo test`
Expected: PASS (9 tests).

- [ ] **Step 6: Commit**

```powershell
git add src-tauri
git commit -m "feat: Program model and pure registry-entry parsing (TDD)"
```

---

### Task 3 : Lecture du registre et commande `list_programs`

**Files:**
- Create: `src-tauri/src/inventory/registry.rs`
- Modify: `src-tauri/src/inventory/mod.rs`, `src-tauri/src/lib.rs`
- Create: `src/lib/types.ts`, `src/lib/api.ts`
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: `models::{Program, Scope}`, `parse::*` (Task 2).
- Produces: Rust `registry::read_installed_programs() -> Vec<Program>` ; commande Tauri `list_programs() -> Vec<Program>` ; TS `type Program` (miroir camelCase exact du struct Rust) et `api.listPrograms(): Promise<Program[]>`.

- [ ] **Step 1: Écrire le test d'intégration (registre réel)**

```rust
// src-tauri/src/inventory/registry.rs — en bas du fichier
#[cfg(test)]
mod tests {
    use super::*;

    /// Test d'intégration : lit le vrai registre de la machine de dev/CI Windows.
    #[test]
    fn reads_at_least_one_visible_program_from_real_registry() {
        let programs = read_installed_programs();
        assert!(
            programs.iter().any(|p| !p.is_system_entry),
            "aucun programme visible trouvé — inattendu sur une machine Windows réelle"
        );
        // Tous les programmes retournés ont un nom non vide et un id unique.
        assert!(programs.iter().all(|p| !p.name.is_empty()));
        let mut ids: Vec<&str> = programs.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "ids en double");
    }
}
```

- [ ] **Step 2: Vérifier l'échec**

Run: `cd src-tauri; cargo test`
Expected: FAIL — `read_installed_programs` non définie.

- [ ] **Step 3: Implémenter `registry.rs`**

```rust
// src-tauri/src/inventory/registry.rs
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
use winreg::RegKey;

use crate::inventory::parse::{
    classify_visibility, clean_icon_path, estimated_size_to_bytes, parse_install_date, EntryVisibility,
};
use crate::models::{Program, Scope};

const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";

/// Lit les 3 sources du cahier des charges (F1) :
/// HKLM 64 bits, HKLM 32 bits (WOW64), HKCU.
pub fn read_installed_programs() -> Vec<Program> {
    let mut out = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    read_hive(&hklm, KEY_WOW64_64KEY, Scope::Machine, "HKLM64", &mut out);
    read_hive(&hklm, KEY_WOW64_32KEY, Scope::Machine, "HKLM32", &mut out);
    read_hive(&hkcu, 0, Scope::User, "HKCU", &mut out);
    out
}

fn read_hive(root: &RegKey, wow_flag: u32, scope: Scope, id_prefix: &str, out: &mut Vec<Program>) {
    let Ok(uninstall) = root.open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | wow_flag) else {
        return;
    };
    for key_name in uninstall.enum_keys().flatten() {
        let Ok(sub) = uninstall.open_subkey_with_flags(&key_name, KEY_READ | wow_flag) else {
            continue;
        };
        if let Some(program) = read_entry(&sub, &key_name, scope, id_prefix) {
            out.push(program);
        }
    }
}

fn read_entry(key: &RegKey, key_name: &str, scope: Scope, id_prefix: &str) -> Option<Program> {
    let display_name: Option<String> = key.get_value("DisplayName").ok();
    let system_component: u32 = key.get_value("SystemComponent").unwrap_or(0);
    let parent_key_name: Option<String> = key.get_value("ParentKeyName").ok();
    let release_type: Option<String> = key.get_value("ReleaseType").ok();

    let visibility = classify_visibility(
        display_name.as_deref(),
        system_component,
        parent_key_name.as_deref(),
        release_type.as_deref(),
    );
    if visibility == EntryVisibility::Skip {
        return None;
    }

    let estimated_size_kib: Option<u32> = key.get_value("EstimatedSize").ok();
    let install_date_raw: Option<String> = key.get_value("InstallDate").ok();
    let display_icon_raw: Option<String> = key.get_value("DisplayIcon").ok();

    let non_empty = |s: String| if s.trim().is_empty() { None } else { Some(s) };

    Some(Program {
        id: format!("{id_prefix}\\{key_name}"),
        name: display_name.unwrap(),
        publisher: key.get_value::<String, _>("Publisher").ok().and_then(non_empty),
        version: key.get_value::<String, _>("DisplayVersion").ok().and_then(non_empty),
        install_date: install_date_raw.as_deref().and_then(parse_install_date),
        estimated_size_bytes: estimated_size_kib.map(estimated_size_to_bytes),
        install_location: key.get_value::<String, _>("InstallLocation").ok().and_then(non_empty),
        uninstall_string: key.get_value::<String, _>("UninstallString").ok().and_then(non_empty),
        quiet_uninstall_string: key.get_value::<String, _>("QuietUninstallString").ok().and_then(non_empty),
        display_icon: display_icon_raw.as_deref().and_then(clean_icon_path),
        scope,
        is_system_entry: visibility == EntryVisibility::Hidden,
    })
}
```

Note : `open_subkey_with_flags` prend un `REGSAM` (u32) — `KEY_READ | wow_flag` compile car les constantes winreg sont des `u32`. Pour HKCU, `wow_flag = 0` (pas de redirection WOW64 pertinente).

Dans `src-tauri/src/inventory/mod.rs` :

```rust
pub mod parse;
pub mod registry;
```

Dans `src-tauri/src/lib.rs`, remplacer la commande d'exemple `greet` par :

```rust
#[tauri::command]
fn list_programs() -> Vec<models::Program> {
    inventory::registry::read_installed_programs()
}
```

et dans le builder : `.invoke_handler(tauri::generate_handler![list_programs])`.

- [ ] **Step 4: Vérifier le passage**

Run: `cd src-tauri; cargo test`
Expected: PASS (tests parse + test registre réel).

- [ ] **Step 5: Câbler le frontend minimal**

```ts
// src/lib/types.ts
export type Scope = "machine" | "user";

export interface Program {
  id: string;
  name: string;
  publisher: string | null;
  version: string | null;
  installDate: string | null; // "YYYY-MM-DD"
  estimatedSizeBytes: number | null;
  installLocation: string | null;
  uninstallString: string | null;
  quietUninstallString: string | null;
  displayIcon: string | null;
  scope: Scope;
  isSystemEntry: boolean;
}
```

```ts
// src/lib/api.ts
import { invoke } from "@tauri-apps/api/core";
import type { Program } from "./types";

export function listPrograms(): Promise<Program[]> {
  return invoke<Program[]>("list_programs");
}
```

Remplacer `src/App.tsx` par un composant de fumée provisoire :

```tsx
import { useEffect, useState } from "react";
import { listPrograms } from "./lib/api";
import type { Program } from "./lib/types";
import "./App.css";

export default function App() {
  const [programs, setPrograms] = useState<Program[]>([]);
  useEffect(() => {
    listPrograms().then(setPrograms);
  }, []);
  return (
    <main className="p-4">
      <p>{programs.filter((p) => !p.isSystemEntry).length} programmes</p>
    </main>
  );
}
```

- [ ] **Step 6: Vérification manuelle**

Run: `npm run tauri dev`
Expected: la fenêtre affiche « N programmes » avec N > 0 et plausible (comparer grossièrement avec le panneau Windows « Applications installées »).

- [ ] **Step 7: Commit**

```powershell
git add -A
git commit -m "feat: read installed programs from registry, list_programs command"
```

---

### Task 4 : Store Zustand, formatage et UI de la liste

**Files:**
- Create: `src/lib/strings.fr.ts`, `src/lib/format.ts`, `src/lib/format.test.ts`, `src/store/programs.ts`
- Create: `src/components/ProgramList.tsx`, `src/components/ProgramRow.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: `api.listPrograms()`, `type Program` (Task 3).
- Produces: `format.formatBytes(bytes: number | null): string`, `format.formatDate(iso: string | null): string` ; store `useProgramsStore` avec état `{ programs: Program[], loading: boolean, error: string | null, selectedId: string | null }` et actions `{ load(): Promise<void>, select(id: string | null): void, patchProgram(id: string, patch: Partial<Program>): void }` ; textes `STR` (objet de chaînes françaises).

- [ ] **Step 1: Écrire les tests de formatage qui échouent**

```ts
// src/lib/format.test.ts
import { describe, expect, it } from "vitest";
import { formatBytes, formatDate } from "./format";

describe("formatBytes", () => {
  it("affiche — pour null", () => expect(formatBytes(null)).toBe("—"));
  it("affiche en Mo sous le Go", () => expect(formatBytes(52_428_800)).toBe("50,0 Mo"));
  it("affiche en Go au-dessus", () => expect(formatBytes(1_610_612_736)).toBe("1,5 Go"));
  it("affiche en Ko sous le Mo", () => expect(formatBytes(10_240)).toBe("10,0 Ko"));
});

describe("formatDate", () => {
  it("affiche — pour null", () => expect(formatDate(null)).toBe("—"));
  it("formate en JJ/MM/AAAA", () => expect(formatDate("2026-07-31")).toBe("31/07/2026"));
});
```

- [ ] **Step 2: Vérifier l'échec**

Run: `npm test`
Expected: FAIL — module `./format` inexistant.

- [ ] **Step 3: Implémenter `format.ts` et `strings.fr.ts`**

```ts
// src/lib/format.ts
const UNITS = [
  { limit: 1024 ** 3, div: 1024 ** 3, suffix: "Go" },
  { limit: 1024 ** 2, div: 1024 ** 2, suffix: "Mo" },
  { limit: 0, div: 1024, suffix: "Ko" },
];

export function formatBytes(bytes: number | null): string {
  if (bytes === null) return "—";
  const unit = UNITS.find((u) => bytes >= u.limit)!;
  const value = (bytes / unit.div).toFixed(1).replace(".", ",");
  return `${value} ${unit.suffix}`;
}

export function formatDate(iso: string | null): string {
  if (iso === null) return "—";
  const [y, m, d] = iso.split("-");
  return `${d}/${m}/${y}`;
}
```

```ts
// src/lib/strings.fr.ts
export const STR = {
  appTitle: "BetterUnistaller",
  searchPlaceholder: "Rechercher un programme ou un éditeur…",
  loading: "Chargement des programmes…",
  loadError: "Impossible de lire la liste des programmes.",
  emptyResult: "Aucun programme ne correspond.",
  colName: "Nom",
  colPublisher: "Éditeur",
  colVersion: "Version",
  colDate: "Installé le",
  colSize: "Taille",
  filterLarge: "Volumineux (> 1 Go)",
  filterRecent: "Récents (< 30 j)",
  filterNoPublisher: "Sans éditeur",
  filterScopeUser: "Par utilisateur",
  showSystem: "Afficher les entrées système",
  programsCount: (n: number) => `${n} programme${n > 1 ? "s" : ""}`,
  detailOpenFolder: "Ouvrir le dossier d'installation",
  detailTechnical: "Détails techniques",
  detailRegistryKey: "Clé de registre",
  detailUninstallString: "Commande de désinstallation",
  detailNoLocation: "Emplacement inconnu",
  copy: "Copier",
} as const;
```

- [ ] **Step 4: Vérifier le passage des tests**

Run: `npm test`
Expected: PASS (6 tests).

- [ ] **Step 5: Implémenter le store et la liste**

```ts
// src/store/programs.ts
import { create } from "zustand";
import { listPrograms } from "../lib/api";
import type { Program } from "../lib/types";
import { STR } from "../lib/strings.fr";

interface ProgramsState {
  programs: Program[];
  loading: boolean;
  error: string | null;
  selectedId: string | null;
  load: () => Promise<void>;
  select: (id: string | null) => void;
  patchProgram: (id: string, patch: Partial<Program>) => void;
}

export const useProgramsStore = create<ProgramsState>((set) => ({
  programs: [],
  loading: false,
  error: null,
  selectedId: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      set({ programs: await listPrograms(), loading: false });
    } catch {
      set({ error: STR.loadError, loading: false });
    }
  },
  select: (id) => set({ selectedId: id }),
  patchProgram: (id, patch) =>
    set((s) => ({
      programs: s.programs.map((p) => (p.id === id ? { ...p, ...patch } : p)),
    })),
}));
```

```tsx
// src/components/ProgramRow.tsx
import type { Program } from "../lib/types";
import { formatBytes, formatDate } from "../lib/format";

export function ProgramRow({ program, selected, onSelect }: {
  program: Program;
  selected: boolean;
  onSelect: (id: string) => void;
}) {
  return (
    <tr
      onClick={() => onSelect(program.id)}
      aria-selected={selected}
      className={`cursor-pointer border-b border-neutral-200 dark:border-neutral-800 ${
        selected ? "bg-blue-50 dark:bg-blue-950" : "hover:bg-neutral-50 dark:hover:bg-neutral-900"
      }`}
    >
      <td className="px-3 py-2 font-medium">{program.name}</td>
      <td className="px-3 py-2 text-neutral-500">{program.publisher ?? "—"}</td>
      <td className="px-3 py-2 text-neutral-500">{program.version ?? "—"}</td>
      <td className="px-3 py-2 text-neutral-500">{formatDate(program.installDate)}</td>
      <td className="px-3 py-2 text-right tabular-nums">{formatBytes(program.estimatedSizeBytes)}</td>
    </tr>
  );
}
```

```tsx
// src/components/ProgramList.tsx
import type { Program } from "../lib/types";
import { STR } from "../lib/strings.fr";
import { ProgramRow } from "./ProgramRow";

export function ProgramList({ programs, selectedId, onSelect }: {
  programs: Program[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  if (programs.length === 0) {
    return <p className="p-6 text-neutral-500">{STR.emptyResult}</p>;
  }
  return (
    <table className="w-full text-sm">
      <thead className="sticky top-0 bg-white dark:bg-neutral-950 text-left text-neutral-400">
        <tr>
          <th className="px-3 py-2">{STR.colName}</th>
          <th className="px-3 py-2">{STR.colPublisher}</th>
          <th className="px-3 py-2">{STR.colVersion}</th>
          <th className="px-3 py-2">{STR.colDate}</th>
          <th className="px-3 py-2 text-right">{STR.colSize}</th>
        </tr>
      </thead>
      <tbody>
        {programs.map((p) => (
          <ProgramRow key={p.id} program={p} selected={p.id === selectedId} onSelect={onSelect} />
        ))}
      </tbody>
    </table>
  );
}
```

Remplacer `src/App.tsx` :

```tsx
import { useEffect } from "react";
import { ProgramList } from "./components/ProgramList";
import { STR } from "./lib/strings.fr";
import { useProgramsStore } from "./store/programs";
import "./App.css";

export default function App() {
  const { programs, loading, error, selectedId, load, select } = useProgramsStore();
  useEffect(() => {
    void load();
  }, [load]);

  const visible = programs.filter((p) => !p.isSystemEntry);

  return (
    <main className="h-screen flex flex-col bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <header className="px-4 py-3 border-b border-neutral-200 dark:border-neutral-800">
        <h1 className="text-lg font-semibold">{STR.appTitle}</h1>
        <p className="text-sm text-neutral-500">{STR.programsCount(visible.length)}</p>
      </header>
      <section className="flex-1 overflow-auto">
        {loading && <p className="p-6 text-neutral-500">{STR.loading}</p>}
        {error && <p className="p-6 text-red-600">{error}</p>}
        {!loading && !error && (
          <ProgramList programs={visible} selectedId={selectedId} onSelect={select} />
        )}
      </section>
    </main>
  );
}
```

- [ ] **Step 6: Vérification manuelle**

Run: `npm run tauri dev`
Expected: tableau lisible des programmes (nom, éditeur, version, date, taille), clic = surlignage de la ligne, thème sombre suivi du système. Liste affichée en moins de 2 s.

- [ ] **Step 7: Commit**

```powershell
git add -A
git commit -m "feat: program list UI with Zustand store and FR strings"
```

---

### Task 5 : Recherche normalisée et tris (TDD)

**Files:**
- Create: `src/lib/search.ts`, `src/lib/search.test.ts`
- Create: `src/components/Toolbar.tsx`
- Modify: `src/store/programs.ts`, `src/App.tsx`, `src/components/ProgramList.tsx`

**Interfaces:**
- Consumes: store et composants de Task 4.
- Produces: `search.normalize(s: string): string`, `search.matches(program: Program, query: string): boolean`, `search.SortKey = "name" | "size" | "date" | "publisher"`, `search.SortDir = "asc" | "desc"`, `search.compareBy(key: SortKey, dir: SortDir): (a: Program, b: Program) => number` ; store étendu avec `{ query, sortKey, sortDir, setQuery, setSort }`.

- [ ] **Step 1: Écrire les tests qui échouent**

```ts
// src/lib/search.test.ts
import { describe, expect, it } from "vitest";
import { compareBy, matches, normalize } from "./search";
import type { Program } from "./types";

const prog = (over: Partial<Program>): Program => ({
  id: "x", name: "", publisher: null, version: null, installDate: null,
  estimatedSizeBytes: null, installLocation: null, uninstallString: null,
  quietUninstallString: null, displayIcon: null, scope: "machine",
  isSystemEntry: false, ...over,
});

describe("normalize", () => {
  it("retire accents et casse", () => {
    expect(normalize("Éditeur Génial")).toBe("editeur genial");
  });
});

describe("matches", () => {
  it("matche le nom sans tenir compte des accents", () => {
    expect(matches(prog({ name: "Télécharger Vidéo" }), "telecharger")).toBe(true);
  });
  it("matche l'éditeur", () => {
    expect(matches(prog({ name: "X", publisher: "Société Nùmerique" }), "societe")).toBe(true);
  });
  it("ne matche pas ailleurs", () => {
    expect(matches(prog({ name: "Paint", publisher: "Contoso" }), "zzz")).toBe(false);
  });
  it("requête vide matche tout", () => {
    expect(matches(prog({ name: "Paint" }), "  ")).toBe(true);
  });
});

describe("compareBy", () => {
  it("trie par taille desc avec null en dernier", () => {
    const a = prog({ id: "a", estimatedSizeBytes: 100 });
    const b = prog({ id: "b", estimatedSizeBytes: null });
    const c = prog({ id: "c", estimatedSizeBytes: 900 });
    const sorted = [a, b, c].sort(compareBy("size", "desc"));
    expect(sorted.map((p) => p.id)).toEqual(["c", "a", "b"]);
  });
  it("trie par nom asc, insensible aux accents", () => {
    const a = prog({ id: "a", name: "Zèbre" });
    const b = prog({ id: "b", name: "abricot" });
    const sorted = [a, b].sort(compareBy("name", "asc"));
    expect(sorted.map((p) => p.id)).toEqual(["b", "a"]);
  });
  it("trie par date desc avec null en dernier", () => {
    const a = prog({ id: "a", installDate: "2026-01-01" });
    const b = prog({ id: "b", installDate: null });
    const c = prog({ id: "c", installDate: "2026-06-15" });
    const sorted = [a, b, c].sort(compareBy("date", "desc"));
    expect(sorted.map((p) => p.id)).toEqual(["c", "a", "b"]);
  });
});
```

- [ ] **Step 2: Vérifier l'échec**

Run: `npm test`
Expected: FAIL — module `./search` inexistant.

- [ ] **Step 3: Implémenter `search.ts`**

```ts
// src/lib/search.ts
import type { Program } from "./types";

export function normalize(s: string): string {
  return s
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .trim();
}

export function matches(program: Program, query: string): boolean {
  const q = normalize(query);
  if (q === "") return true;
  return (
    normalize(program.name).includes(q) ||
    (program.publisher !== null && normalize(program.publisher).includes(q))
  );
}

export type SortKey = "name" | "size" | "date" | "publisher";
export type SortDir = "asc" | "desc";

/** null est toujours classé en dernier, quel que soit le sens. */
export function compareBy(key: SortKey, dir: SortDir) {
  const sign = dir === "asc" ? 1 : -1;
  return (a: Program, b: Program): number => {
    const [va, vb] = [sortValue(a, key), sortValue(b, key)];
    if (va === null && vb === null) return 0;
    if (va === null) return 1;
    if (vb === null) return -1;
    if (typeof va === "number" && typeof vb === "number") return sign * (va - vb);
    return sign * String(va).localeCompare(String(vb));
  };
}

function sortValue(p: Program, key: SortKey): string | number | null {
  switch (key) {
    case "name": return normalize(p.name);
    case "publisher": return p.publisher === null ? null : normalize(p.publisher);
    case "size": return p.estimatedSizeBytes;
    case "date": return p.installDate; // ISO : ordre lexicographique = ordre chronologique
  }
}
```

- [ ] **Step 4: Vérifier le passage**

Run: `npm test`
Expected: PASS.

- [ ] **Step 5: Câbler recherche + tri dans le store et l'UI**

Étendre `src/store/programs.ts` — ajouter à l'interface `ProgramsState` :

```ts
import type { SortDir, SortKey } from "../lib/search";
// Champs : query: string; sortKey: SortKey; sortDir: SortDir;
// Actions : setQuery: (q: string) => void; setSort: (key: SortKey) => void;
```

et à l'implémentation dans `create<ProgramsState>()` :

```ts
  query: "",
  sortKey: "name" as SortKey,
  sortDir: "asc" as SortDir,
  setQuery: (query) => set({ query }),
  // Re-cliquer la même colonne inverse le sens ; nouvelle colonne : asc pour
  // les textes, desc pour taille/date (usage le plus courant).
  setSort: (key) =>
    set((s) =>
      s.sortKey === key
        ? { sortDir: s.sortDir === "asc" ? "desc" : "asc" }
        : { sortKey: key, sortDir: key === "name" || key === "publisher" ? "asc" : "desc" }
    ),
```

```tsx
// src/components/Toolbar.tsx
import { STR } from "../lib/strings.fr";

export function Toolbar({ query, onQuery }: { query: string; onQuery: (q: string) => void }) {
  return (
    <div className="px-4 py-2 border-b border-neutral-200 dark:border-neutral-800">
      <input
        type="search"
        value={query}
        onChange={(e) => onQuery(e.target.value)}
        placeholder={STR.searchPlaceholder}
        className="w-full max-w-md rounded-md border border-neutral-300 dark:border-neutral-700 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-blue-500"
      />
    </div>
  );
}
```

Dans `ProgramList.tsx`, rendre les en-têtes cliquables : passer les props `sortKey: SortKey`, `sortDir: SortDir`, `onSort: (k: SortKey) => void` et transformer le contenu de chaque `<th>` en `<button>` appelant `onSort("name")`, `onSort("publisher")`, `onSort("date")`, `onSort("size")` respectivement, avec un indicateur `▲`/`▼` affiché sur la colonne active selon `sortDir`. (La colonne Version n'est pas triable.)

Dans `App.tsx`, calculer la liste affichée :

```tsx
import { compareBy, matches } from "./lib/search";
import { Toolbar } from "./components/Toolbar";
// ...
const { query, sortKey, sortDir, setQuery, setSort } = useProgramsStore();
const visible = programs
  .filter((p) => !p.isSystemEntry)
  .filter((p) => matches(p, query))
  .sort(compareBy(sortKey, sortDir));
// <Toolbar query={query} onQuery={setQuery} /> entre le header et la liste,
// et passer sortKey/sortDir/onSort={setSort} à <ProgramList>.
```

- [ ] **Step 6: Vérification manuelle**

Run: `npm run tauri dev`
Expected: taper « visu » trouve « Microsoft Visual C++ … » si visible ; recherche insensible aux accents ; clic sur « Taille » trie décroissant, second clic inverse.

- [ ] **Step 7: Commit**

```powershell
git add -A
git commit -m "feat: accent-insensitive search and column sorting (TDD)"
```

---

### Task 6 : Filtres (TDD)

**Files:**
- Create: `src/lib/filters.ts`, `src/lib/filters.test.ts`
- Modify: `src/store/programs.ts`, `src/components/Toolbar.tsx`, `src/App.tsx`

**Interfaces:**
- Consumes: store et `Toolbar` (Task 5).
- Produces: `filters.FilterId = "large" | "recent" | "noPublisher" | "userScope"`, `filters.applyFilters(programs: Program[], active: Set<FilterId>, nowIso: string): Program[]` ; store étendu avec `{ activeFilters: Set<FilterId>, showSystem: boolean, toggleFilter(id: FilterId), toggleShowSystem() }`.

- [ ] **Step 1: Écrire les tests qui échouent**

```ts
// src/lib/filters.test.ts
import { describe, expect, it } from "vitest";
import { applyFilters } from "./filters";
import type { Program } from "./types";

const prog = (over: Partial<Program>): Program => ({
  id: Math.random().toString(36).slice(2), name: "P", publisher: "E", version: null,
  installDate: null, estimatedSizeBytes: null, installLocation: null,
  uninstallString: null, quietUninstallString: null, displayIcon: null,
  scope: "machine", isSystemEntry: false, ...over,
});

const NOW = "2026-07-31";

describe("applyFilters", () => {
  it("sans filtre actif, rend tout", () => {
    const list = [prog({}), prog({})];
    expect(applyFilters(list, new Set(), NOW)).toHaveLength(2);
  });
  it("large : > 1 Go strictement", () => {
    const big = prog({ estimatedSizeBytes: 2 * 1024 ** 3 });
    const small = prog({ estimatedSizeBytes: 500 * 1024 ** 2 });
    const unknown = prog({ estimatedSizeBytes: null });
    expect(applyFilters([big, small, unknown], new Set(["large"]), NOW)).toEqual([big]);
  });
  it("recent : installé il y a moins de 30 jours", () => {
    const recent = prog({ installDate: "2026-07-15" });
    const old = prog({ installDate: "2026-01-01" });
    const unknown = prog({ installDate: null });
    expect(applyFilters([recent, old, unknown], new Set(["recent"]), NOW)).toEqual([recent]);
  });
  it("noPublisher : éditeur absent", () => {
    const anon = prog({ publisher: null });
    const named = prog({ publisher: "Contoso" });
    expect(applyFilters([anon, named], new Set(["noPublisher"]), NOW)).toEqual([anon]);
  });
  it("userScope : installations par utilisateur", () => {
    const user = prog({ scope: "user" });
    const machine = prog({ scope: "machine" });
    expect(applyFilters([user, machine], new Set(["userScope"]), NOW)).toEqual([user]);
  });
  it("les filtres se cumulent (ET)", () => {
    const match = prog({ publisher: null, scope: "user" });
    const half = prog({ publisher: null, scope: "machine" });
    expect(applyFilters([match, half], new Set(["noPublisher", "userScope"]), NOW)).toEqual([match]);
  });
});
```

- [ ] **Step 2: Vérifier l'échec**

Run: `npm test`
Expected: FAIL — module `./filters` inexistant.

- [ ] **Step 3: Implémenter `filters.ts`**

```ts
// src/lib/filters.ts
import type { Program } from "./types";

export type FilterId = "large" | "recent" | "noPublisher" | "userScope";

const ONE_GIB = 1024 ** 3;
const THIRTY_DAYS_MS = 30 * 24 * 60 * 60 * 1000;

const PREDICATES: Record<FilterId, (p: Program, nowIso: string) => boolean> = {
  large: (p) => p.estimatedSizeBytes !== null && p.estimatedSizeBytes > ONE_GIB,
  recent: (p, nowIso) =>
    p.installDate !== null &&
    Date.parse(nowIso) - Date.parse(p.installDate) < THIRTY_DAYS_MS,
  noPublisher: (p) => p.publisher === null,
  userScope: (p) => p.scope === "user",
};

/** Les filtres actifs se cumulent en ET. `nowIso` est injecté pour la testabilité. */
export function applyFilters(programs: Program[], active: Set<FilterId>, nowIso: string): Program[] {
  if (active.size === 0) return programs;
  return programs.filter((p) => [...active].every((id) => PREDICATES[id](p, nowIso)));
}
```

- [ ] **Step 4: Vérifier le passage**

Run: `npm test`
Expected: PASS.

- [ ] **Step 5: Câbler dans le store et la barre d'outils**

Store — ajouts :

```ts
import type { FilterId } from "../lib/filters";
// État : activeFilters: new Set<FilterId>(), showSystem: false,
// Actions :
  toggleFilter: (id: FilterId) =>
    set((s) => {
      const next = new Set(s.activeFilters);
      if (next.has(id)) next.delete(id); else next.add(id);
      return { activeFilters: next };
    }),
  toggleShowSystem: () => set((s) => ({ showSystem: !s.showSystem })),
```

`Toolbar.tsx` — ajouter sous le champ de recherche une rangée de puces à bascule (checkbox stylées) pour les 4 filtres (`STR.filterLarge`, `STR.filterRecent`, `STR.filterNoPublisher`, `STR.filterScopeUser`) et, alignée à droite, une case `STR.showSystem`. Props ajoutées : `activeFilters: Set<FilterId>`, `onToggleFilter: (id: FilterId) => void`, `showSystem: boolean`, `onToggleShowSystem: () => void`.

`App.tsx` — chaîne d'affichage complète :

```tsx
import { applyFilters } from "./lib/filters";
const nowIso = new Date().toISOString().slice(0, 10);
const visible = applyFilters(
  programs.filter((p) => showSystem || !p.isSystemEntry).filter((p) => matches(p, query)),
  activeFilters,
  nowIso
).sort(compareBy(sortKey, sortDir));
```

- [ ] **Step 6: Vérification manuelle**

Run: `npm run tauri dev`
Expected: activer « Volumineux » réduit la liste aux > 1 Go ; « Afficher les entrées système » fait apparaître runtimes et mises à jour ; les filtres se cumulent.

- [ ] **Step 7: Commit**

```powershell
git add -A
git commit -m "feat: composable list filters and system-entries toggle (TDD)"
```

---

### Task 7 : Calcul asynchrone des tailles manquantes

**Files:**
- Create: `src-tauri/src/inventory/size.rs`
- Modify: `src-tauri/src/inventory/mod.rs`, `src-tauri/src/lib.rs`
- Modify: `src/lib/api.ts`, `src/App.tsx`

**Interfaces:**
- Consumes: `patchProgram` du store (Task 4), `Program.installLocation`.
- Produces: Rust `size::dir_size(path: &Path) -> u64` ; commande `compute_sizes(app: AppHandle, requests: Vec<SizeRequest>)` avec `SizeRequest { id: String, path: String }` (camelCase côté JS) ; événement `size-computed` de payload `{ id: string, sizeBytes: number }` ; TS `api.computeSizes(requests)` et `api.onSizeComputed(cb)`.

- [ ] **Step 1: Écrire le test qui échoue**

```rust
// src-tauri/src/inventory/size.rs — en bas du fichier
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sums_nested_files_and_survives_missing_dir() {
        let dir = std::env::temp_dir().join("bu_size_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.bin"), vec![0u8; 1000]).unwrap();
        fs::write(dir.join("sub").join("b.bin"), vec![0u8; 500]).unwrap();

        assert_eq!(dir_size(&dir), 1500);
        assert_eq!(dir_size(std::path::Path::new(r"C:\bu_inexistant_xyz")), 0);

        fs::remove_dir_all(&dir).unwrap();
    }
}
```

- [ ] **Step 2: Vérifier l'échec**

Run: `cd src-tauri; cargo test`
Expected: FAIL — `dir_size` non définie.

- [ ] **Step 3: Implémenter `size.rs` et la commande**

```rust
// src-tauri/src/inventory/size.rs
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeRequest {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeComputed {
    pub id: String,
    pub size_bytes: u64,
}

/// Somme récursive, tolérante aux erreurs (accès refusé, dossier disparu → ignoré).
/// Ne suit pas les liens symboliques pour éviter boucles et double comptage.
pub fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else { return 0 };
    entries
        .flatten()
        .map(|entry| {
            let Ok(meta) = entry.metadata() else { return 0 };
            if meta.is_symlink() {
                0
            } else if meta.is_dir() {
                dir_size(&entry.path())
            } else {
                meta.len()
            }
        })
        .sum()
}

/// Calcule séquentiellement en tâche de fond ; chaque résultat est poussé
/// via l'événement `size-computed` dès qu'il est prêt.
#[tauri::command]
pub fn compute_sizes(app: AppHandle, requests: Vec<SizeRequest>) {
    tauri::async_runtime::spawn_blocking(move || {
        for req in requests {
            let size_bytes = dir_size(Path::new(&req.path));
            let _ = app.emit("size-computed", SizeComputed { id: req.id, size_bytes });
        }
    });
}
```

`mod.rs` : ajouter `pub mod size;`. Dans `lib.rs`, ajouter `inventory::size::compute_sizes` au `generate_handler![...]`.

- [ ] **Step 4: Vérifier le passage**

Run: `cd src-tauri; cargo test`
Expected: PASS.

- [ ] **Step 5: Câbler le frontend**

```ts
// src/lib/api.ts — ajouts
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface SizeRequest { id: string; path: string }
export interface SizeComputed { id: string; sizeBytes: number }

export function computeSizes(requests: SizeRequest[]): Promise<void> {
  return invoke("compute_sizes", { requests });
}

export function onSizeComputed(cb: (e: SizeComputed) => void): Promise<UnlistenFn> {
  return listen<SizeComputed>("size-computed", (event) => cb(event.payload));
}
```

Dans `App.tsx`, après le chargement initial :

```tsx
useEffect(() => {
  const un = onSizeComputed(({ id, sizeBytes }) =>
    useProgramsStore.getState().patchProgram(id, { estimatedSizeBytes: sizeBytes })
  );
  return () => { void un.then((f) => f()); };
}, []);

const loaded = programs.length > 0;
useEffect(() => {
  if (!loaded) return;
  const missing = useProgramsStore.getState().programs
    .filter((p) => p.estimatedSizeBytes === null && p.installLocation !== null)
    .map((p) => ({ id: p.id, path: p.installLocation! }));
  if (missing.length > 0) void computeSizes(missing);
}, [loaded]); // volontairement déclenché une seule fois après le premier chargement
```

- [ ] **Step 6: Vérification manuelle**

Run: `npm run tauri dev`
Expected: des tailles « — » se remplissent progressivement après l'affichage de la liste, sans geler l'interface (scroll fluide pendant le calcul).

- [ ] **Step 7: Commit**

```powershell
git add -A
git commit -m "feat: background size computation with size-computed events"
```

---

### Task 8 : Extraction des icônes en arrière-plan

**Files:**
- Create: `src-tauri/src/inventory/icon.rs`
- Modify: `src-tauri/src/inventory/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`
- Modify: `src/lib/api.ts`, `src/lib/types.ts`, `src/components/ProgramRow.tsx`, `src/App.tsx`

**Interfaces:**
- Consumes: `Program.displayIcon` (chemin nettoyé par `parse::clean_icon_path`, Task 2), `patchProgram` (Task 4).
- Produces: commande `load_icons(app: AppHandle, requests: Vec<IconRequest>)` avec `IconRequest { id: String, path: String }` ; événement `icon-ready` de payload `{ id: string, dataUri: string }` ; TS `Program.iconDataUri?: string`, `api.loadIcons(requests)`, `api.onIconReady(cb)` ; avatar de repli (initiale du nom) dans `ProgramRow`.

- [ ] **Step 1: Ajouter les dépendances Rust**

```toml
# src-tauri/Cargo.toml [dependencies]
image = { version = "0.25", default-features = false, features = ["png", "ico"] }
base64 = "0.22"
windows = { version = "0.61", features = [
  "Win32_UI_Shell",
  "Win32_UI_WindowsAndMessaging",
  "Win32_Graphics_Gdi",
  "Win32_Storage_FileSystem",
] }
```

Note d'exécution : si la version 0.61 du crate `windows` n'est pas disponible, prendre la dernière stable (`cargo add windows -F Win32_UI_Shell,Win32_UI_WindowsAndMessaging,Win32_Graphics_Gdi,Win32_Storage_FileSystem`) et adapter les imports — l'API `SHGetFileInfoW`/GDI est stable entre versions.

- [ ] **Step 2: Écrire le test qui échoue (chemin .ico pur, sans Win32)**

```rust
// src-tauri/src/inventory/icon.rs — en bas du fichier
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ico_file_is_converted_to_png_data_uri() {
        // Génère un .ico 16x16 monochrome sur disque via le crate image.
        let dir = std::env::temp_dir().join("bu_icon_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("t.ico");
        let img = image::RgbaImage::from_pixel(16, 16, image::Rgba([255, 0, 0, 255]));
        img.save(&path).unwrap();

        let uri = icon_data_uri(path.to_str().unwrap()).expect("doit produire une icône");
        assert!(uri.starts_with("data:image/png;base64,"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_yields_none() {
        assert_eq!(icon_data_uri(r"C:\bu_inexistant\x.exe"), None);
    }
}
```

- [ ] **Step 3: Vérifier l'échec**

Run: `cd src-tauri; cargo test`
Expected: FAIL — `icon_data_uri` non définie.

- [ ] **Step 4: Implémenter `icon.rs`**

```rust
// src-tauri/src/inventory/icon.rs
use base64::Engine;
use image::ImageEncoder;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconRequest {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconReady {
    pub id: String,
    pub data_uri: String,
}

/// Résout un chemin d'icône (fichier .ico direct, ou icône embarquée d'un .exe/.dll
/// via SHGetFileInfoW) en data URI PNG 32x32. None si introuvable.
pub fn icon_data_uri(path: &str) -> Option<String> {
    let p = Path::new(path);
    if !p.exists() {
        return None;
    }
    let rgba = if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("ico")) {
        load_ico(p)?
    } else {
        extract_exe_icon(p)?
    };
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(rgba.as_raw(), rgba.width(), rgba.height(), image::ExtendedColorType::Rgba8)
        .ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

fn load_ico(path: &Path) -> Option<image::RgbaImage> {
    let img = image::open(path).ok()?;
    Some(img.thumbnail(32, 32).to_rgba8())
}

fn extract_exe_icon(path: &Path) -> Option<image::RgbaImage> {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    let wide: Vec<u16> = encode_wide_nul(path.as_os_str());
    let mut info = SHFILEINFOW::default();
    unsafe {
        let ok = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if ok == 0 || info.hIcon.is_invalid() {
            return None;
        }
        let hicon = info.hIcon;

        let mut icon_info = ICONINFO::default();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            let _ = DestroyIcon(hicon);
            return None;
        }

        const SIZE: i32 = 32;
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        let old = SelectObject(mem_dc, icon_info.hbmColor.into());

        let mut bmi = BITMAPINFO::default();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: SIZE,
            biHeight: -SIZE, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut pixels = vec![0u8; (SIZE * SIZE * 4) as usize];
        let lines = GetDIBits(
            mem_dc,
            icon_info.hbmColor,
            0,
            SIZE as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(mem_dc, old);
        let _ = DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());
        let _ = DestroyIcon(hicon);

        if lines == 0 {
            return None;
        }
        // BGRA -> RGBA
        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        image::RgbaImage::from_raw(SIZE as u32, SIZE as u32, pixels)
    }
}

/// OsStr -> UTF-16 terminé par NUL.
fn encode_wide_nul(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(std::iter::once(0)).collect()
}

#[tauri::command]
pub fn load_icons(app: AppHandle, requests: Vec<IconRequest>) {
    tauri::async_runtime::spawn_blocking(move || {
        for req in requests {
            if let Some(data_uri) = icon_data_uri(&req.path) {
                let _ = app.emit("icon-ready", IconReady { id: req.id, data_uri });
            }
        }
    });
}
```

Note d'exécution : les signatures exactes GDI/Shell varient légèrement selon la version du crate `windows` (ex. `SelectObject` acceptant `HGDIOBJ` via `.into()`). Corriger les erreurs de compilation en suivant les messages du compilateur — la logique (SHGetFileInfoW → GetIconInfo → GetDIBits → BGRA→RGBA → PNG) ne change pas. Si l'extraction échoue sur certains exe, retourner `None` (l'avatar de repli couvre le cas).

`mod.rs` : ajouter `pub mod icon;`. `lib.rs` : ajouter `inventory::icon::load_icons` au handler.

- [ ] **Step 5: Vérifier le passage**

Run: `cd src-tauri; cargo test`
Expected: PASS (test .ico et test fichier manquant ; l'extraction exe est couverte par la vérification manuelle Step 7).

- [ ] **Step 6: Câbler le frontend avec avatar de repli**

`types.ts` : ajouter `iconDataUri?: string;` à `Program`.

```ts
// src/lib/api.ts — ajouts
export interface IconRequest { id: string; path: string }
export interface IconReady { id: string; dataUri: string }

export function loadIcons(requests: IconRequest[]): Promise<void> {
  return invoke("load_icons", { requests });
}
export function onIconReady(cb: (e: IconReady) => void): Promise<UnlistenFn> {
  return listen<IconReady>("icon-ready", (event) => cb(event.payload));
}
```

`App.tsx` — même schéma que les tailles : dans l'effet déclenché par `loaded`, ajouter `loadIcons(programs.filter(p => p.displayIcon).map(p => ({ id: p.id, path: p.displayIcon! })))` ; un écouteur `onIconReady` fait `patchProgram(id, { iconDataUri: dataUri })`.

`ProgramRow.tsx` — première cellule :

```tsx
<td className="px-3 py-2 font-medium">
  <span className="inline-flex items-center gap-2">
    {program.iconDataUri ? (
      <img src={program.iconDataUri} alt="" className="h-5 w-5" />
    ) : (
      <span className="h-5 w-5 rounded bg-neutral-200 dark:bg-neutral-700 text-[10px] font-bold inline-flex items-center justify-center text-neutral-600 dark:text-neutral-300">
        {program.name.charAt(0).toUpperCase()}
      </span>
    )}
    {program.name}
  </span>
</td>
```

- [ ] **Step 7: Vérification manuelle**

Run: `npm run tauri dev`
Expected: la majorité des lignes affichent leur vraie icône en quelques secondes ; les autres gardent l'avatar initiale ; pas de gel UI.

- [ ] **Step 8: Commit**

```powershell
git add -A
git commit -m "feat: background icon extraction (ico + exe via SHGetFileInfoW)"
```

---

### Task 9 : Panneau de détail

**Files:**
- Create: `src/components/DetailPanel.tsx`
- Modify: `src/App.tsx`, `src/lib/api.ts`
- Modify: `src-tauri/capabilities/default.json` (vérification)

**Interfaces:**
- Consumes: `Program` sélectionné via `selectedId` (Task 4), plugin opener (installé Task 1), `STR` (Task 4).
- Produces: panneau latéral droit affichant le programme sélectionné ; `api.openFolder(path: string)` ; section « Détails techniques » repliée (`<details>`) avec clé registre et commande de désinstallation + boutons Copier.

- [ ] **Step 1: Autoriser l'ouverture de dossiers**

Vérifier dans `src-tauri/capabilities/default.json` que `"opener:default"` figure dans `permissions` (présent par défaut avec le scaffold), et que le plugin est enregistré dans `lib.rs` (`.plugin(tauri_plugin_opener::init())`, fait par le scaffold). `revealItemInDir` est la seule API opener utilisée.

```ts
// src/lib/api.ts — ajout
import { revealItemInDir } from "@tauri-apps/plugin-opener";

export function openFolder(path: string): Promise<void> {
  return revealItemInDir(path);
}
```

- [ ] **Step 2: Implémenter `DetailPanel.tsx`**

```tsx
// src/components/DetailPanel.tsx
import type { Program } from "../lib/types";
import { formatBytes, formatDate } from "../lib/format";
import { openFolder } from "../lib/api";
import { STR } from "../lib/strings.fr";

function TechRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="mt-2">
      <p className="text-xs text-neutral-400">{label}</p>
      <div className="flex items-start gap-2">
        <code className="text-xs break-all">{value}</code>
        <button
          className="text-xs text-blue-600 hover:underline shrink-0"
          onClick={() => navigator.clipboard.writeText(value)}
        >
          {STR.copy}
        </button>
      </div>
    </div>
  );
}

export function DetailPanel({ program }: { program: Program }) {
  return (
    <aside className="w-96 shrink-0 border-l border-neutral-200 dark:border-neutral-800 p-4 overflow-auto">
      <div className="flex items-center gap-3">
        {program.iconDataUri && <img src={program.iconDataUri} alt="" className="h-8 w-8" />}
        <div>
          <h2 className="font-semibold">{program.name}</h2>
          <p className="text-sm text-neutral-500">{program.publisher ?? "—"}</p>
        </div>
      </div>

      <dl className="mt-4 space-y-1 text-sm">
        <div className="flex justify-between"><dt className="text-neutral-500">{STR.colVersion}</dt><dd>{program.version ?? "—"}</dd></div>
        <div className="flex justify-between"><dt className="text-neutral-500">{STR.colDate}</dt><dd>{formatDate(program.installDate)}</dd></div>
        <div className="flex justify-between"><dt className="text-neutral-500">{STR.colSize}</dt><dd>{formatBytes(program.estimatedSizeBytes)}</dd></div>
      </dl>

      <div className="mt-4">
        {program.installLocation ? (
          <button
            className="rounded-md border border-neutral-300 dark:border-neutral-700 px-3 py-1.5 text-sm hover:bg-neutral-50 dark:hover:bg-neutral-900"
            onClick={() => void openFolder(program.installLocation!)}
          >
            {STR.detailOpenFolder}
          </button>
        ) : (
          <p className="text-sm text-neutral-400">{STR.detailNoLocation}</p>
        )}
      </div>

      <details className="mt-6">
        <summary className="cursor-pointer text-sm text-neutral-500">{STR.detailTechnical}</summary>
        <TechRow label={STR.detailRegistryKey} value={program.id} />
        {program.uninstallString && (
          <TechRow label={STR.detailUninstallString} value={program.uninstallString} />
        )}
      </details>
    </aside>
  );
}
```

- [ ] **Step 3: Intégrer dans `App.tsx`**

La zone principale devient un flex horizontal : liste à gauche (flex-1), et si `selectedId` correspond à un programme, `<DetailPanel program={selected} />` à droite. Un clic sur la ligne déjà sélectionnée désélectionne (`select(null)` — adapter `onSelect` : `select(program.id === selectedId ? null : program.id)`).

```tsx
const selected = programs.find((p) => p.id === selectedId) ?? null;
// <section className="flex-1 flex overflow-hidden">
//   <div className="flex-1 overflow-auto"><ProgramList ... /></div>
//   {selected && <DetailPanel program={selected} />}
// </section>
```

- [ ] **Step 4: Vérification manuelle**

Run: `npm run tauri dev`
Expected: sélection d'une ligne → panneau à droite avec infos et icône ; « Ouvrir le dossier d'installation » ouvre l'Explorateur sur le bon dossier ; « Détails techniques » replié par défaut, boutons Copier fonctionnels.

- [ ] **Step 5: Commit**

```powershell
git add -A
git commit -m "feat: detail panel with open-folder action and technical details"
```

---

### Task 10 : Contrôle qualité M1 et étiquette de fin de jalon

**Files:**
- Modify: correctifs éventuels issus de la liste de contrôle
- Create: `README.md`

**Interfaces:**
- Consumes: tout M1.
- Produces: jalon M1 vérifié contre le cahier des charges, `README.md`, tag git `m1`.

- [ ] **Step 1: Passer toute la suite de tests**

Run: `npm test` puis `cd src-tauri; cargo test; cd ..`
Expected: tout PASS.

- [ ] **Step 2: Vérification croisée avec le cahier des charges (F1, F2, §4)**

Liste de contrôle manuelle, application lancée :

1. Les 3 sources registre sont couvertes : repérer dans la liste un programme 32 bits (id préfixé `HKLM32`) et un programme « par utilisateur » (filtre « Par utilisateur » non vide sur une machine typique).
2. Liste affichée < 2 s après lancement (chronométrer à la main).
3. Tailles et icônes se complètent après coup sans gel (scroller pendant le chargement).
4. Recherche insensible aux accents/casse ; tris sur les 4 colonnes ; 4 filtres cumulables ; interrupteur entrées système.
5. Panneau de détail complet, ouverture de dossier fonctionnelle.
6. Aucune demande d'élévation UAC à aucun moment.

Corriger tout écart avant de continuer.

- [ ] **Step 3: Écrire un `README.md` minimal**

Contenu exact :

```markdown
# BetterUnistaller

Désinstalleur Windows propre : désinstalle, puis nettoie ce que le
désinstalleur officiel laisse derrière lui (fichiers, registre).

- Spécification : `docs/cahier-des-charges.md`
- Plans d'implémentation : `docs/superpowers/plans/`

## Développement

Prérequis : Node.js 20+, Rust stable, outillage Tauri 2 pour Windows.

    npm install
    npm run tauri dev          # application en mode dev
    npm test                   # tests frontend (Vitest)
    cd src-tauri; cargo test   # tests backend (Rust)

État : jalon M1 (inventaire) livré. Prochain jalon : M2 (désinstallation standard).
```

- [ ] **Step 4: Commit final et tag**

```powershell
git add -A
git commit -m "chore: M1 acceptance pass and README"
git tag m1
```

---

## Couverture du cahier des charges (auto-revue)

| Exigence spec | Tâche |
|---|---|
| F1 — 3 sources registre HKLM64/HKLM32/HKCU | Task 3 |
| F1 — nom, éditeur, version, date, taille, icône, emplacement | Tasks 3, 4, 7, 8 |
| F1 — entrées système masquées par défaut, option d'affichage | Tasks 2, 6 |
| F1 — chargement asynchrone (UI immédiate, tailles/icônes ensuite) | Tasks 7, 8 |
| F2 — recherche nom/éditeur insensible accents/casse | Task 5 |
| F2 — tris nom/taille/date/éditeur | Task 5 |
| F2 — filtres volumineux/récents/sans éditeur/par utilisateur | Task 6 |
| §4 — liste < 2 s, travail lourd hors thread UI | Tasks 7, 8, 10 |
| §4 — pas d'élévation, pas de réseau | Global Constraints + Task 10 |
| §5 — modules Rust à responsabilité unique, commandes typées | Tasks 2, 3, 7, 8 |
| §6 — écrans liste + détail | Tasks 4, 9 |

Hors de ce plan (plans suivants) : F3 désinstallation (M2), F4 scan résidus (M3), F5 lots et F6 forcée (M4), F7 garde-fous complets, F8 export, i18n/paramètres/installeur (M5).

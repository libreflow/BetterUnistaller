# Plan d'amélioration — post-audit M1 (BetterUnistaller)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Contexte :** audit du 18 septembre 2026 (jalon M1 livré : F1/F2). Constat principal : code M1 solide (20 tests frontend + 15 tests backend verts, 0 vulnérabilité npm/cargo, 1 seul warning clippy cosmétique) mais **aucune CI** avant d'attaquer M2/M3 (désinstallation, scan de résidus) — précisément les jalons où le cahier des charges identifie le risque le plus critique (faux positif → suppression d'un fichier d'un autre programme). Ce plan referme les trous constatés par l'audit sans toucher au périmètre fonctionnel M2+.

**Non-objectifs :** ce plan n'implémente aucune fonctionnalité F3–F8. Il ne force pas non plus un score parfait sur des dimensions disproportionnées pour un outil desktop mono-utilisateur hors-ligne (pas de SAST, pas de fuzzing, pas de matrice multi-OS pour l'instant — CI Windows seule suffit puisque c'est la seule cible).

## Global Constraints

- Ne pas casser les 20+15 tests existants ; chaque tâche se termine par une exécution réelle (`npm test`, `cargo test`, `cargo clippy`, `tsc --noEmit`) confirmant 0 régression.
- Aucune fonctionnalité nouvelle : uniquement outillage, nettoyage, hygiène de dépôt.
- Respecter les conventions déjà en place : libellés FR centralisés dans `strings.fr.ts`, logique pure testée séparément des composants/commandes.

---

### Task 1 : Committer les changements en cours

**Files:**
- Modify (déjà modifiés, à committer) : `index.html`, `src-tauri/src/inventory/icon.rs`, `src-tauri/src/inventory/size.rs`, `src/App.tsx`
- Delete : `public/vite.svg`, `src/assets/react.svg` (déjà supprimés en working copy)

**Contexte :** 4 changements réels et déjà vérifiés par l'audit (titre FR + suppression des assets scaffold Vite/React résiduels ; extraction d'icône Win32 qui interroge la bitmap réelle au lieu de supposer 32×32 fixe ; distinction `dir_size_checked` "racine illisible" vs "dossier vide" ; front qui relance le calcul de taille aussi quand elle vaut `0`, pas seulement `null`). Rien à coder — juste committer proprement, en 2 commits séparés (le sujet UI/scaffold est indépendant du sujet taille/icône).

- [ ] **Step 1 : Commit scaffold cleanup + titre FR**

```bash
git add index.html public/vite.svg src/assets/react.svg
git commit -m "chore: French title, drop leftover Vite/React scaffold assets"
```

- [ ] **Step 2 : Commit correctifs taille/icône**

```bash
git add src-tauri/src/inventory/icon.rs src-tauri/src/inventory/size.rs src/App.tsx
git commit -m "fix: size icons on real bitmap dims, distinguish unknown vs zero size"
```

- [ ] **Step 3 : Vérifier l'arbre propre**

Run: `git status`
Expected: `nothing to commit, working tree clean`

---

### Task 2 : CI GitHub Actions (Windows)

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: scripts déjà existants (`npm test`, `npm run build`, `cargo test`, `cargo clippy`).
- Produces: un workflow qui tourne sur chaque push/PR vers `main`, sur `windows-latest` (seule cible pertinente — projet Windows-only, pas de dépendances système Linux à deviner).

- [ ] **Step 1 : Écrire le workflow**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  frontend:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
      - run: npm ci
      - run: npx tsc --noEmit
      - run: npm test
      - run: npm run build

  backend:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      - run: cargo test
        working-directory: src-tauri
      - run: cargo clippy --all-targets -- -D warnings
        working-directory: src-tauri
```

Note : `cargo clippy -- -D warnings` échouera tant que Task 3 (nettoyage du warning existant) n'est pas faite — faire Task 3 avant ou dans le même push que ce workflow.

- [ ] **Step 2 : Pousser et vérifier le run réel**

Run: `git add .github/workflows/ci.yml && git commit -m "ci: add Windows workflow (frontend + backend)" && git push`
Puis : `gh run watch --exit-status` (ou lecture via `gh run list`/`gh run view <id> --log`)
Expected : les deux jobs (`frontend`, `backend`) verts. Citer la durée réelle et le résultat par job dans le rapport, pas une supposition.

---

### Task 3 : Nettoyer le warning clippy restant

**Files:**
- Modify: `src-tauri/src/inventory/icon.rs`

**Contexte :** `cargo clippy` remonte `field_reassign_with_default` sur `bmi.bmiHeader = BITMAPINFOHEADER { ... }` après `BITMAPINFO::default()`. Fix mécanique suggéré par clippy lui-même : construire `bmi` directement avec `bmiHeader` inline plutôt que réassigner après `default()`.

- [ ] **Step 1 : Appliquer le fix suggéré par clippy**

Remplacer :
```rust
let mut bmi = BITMAPINFO::default();
bmi.bmiHeader = BITMAPINFOHEADER {
    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
    biWidth: width,
    biHeight: -height,
    biPlanes: 1,
    biBitCount: 32,
    biCompression: BI_RGB.0,
    ..Default::default()
};
```
par :
```rust
let bmi = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    },
    ..Default::default()
};
```
(`bmi` n'est plus muté ensuite — retirer aussi `mut` si le compilateur le signale.)

- [ ] **Step 2 : Vérifier**

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: 0 warning, 15/15 tests toujours verts.

---

### Task 4 : Mises à jour de dépendances mineures (patchs sans risque)

**Files:**
- Modify: `package.json` / `package-lock.json` (via `npm update`)

**Contexte :** `npm outdated` (audit du 18/09) : uniquement des patchs mineurs sans breaking change documenté (`@tauri-apps/plugin-opener` 2.5.4→2.5.5, `@types/react`/`@types/react-dom` →19.3.0, `react`/`react-dom` →19.3.0, `zustand` →5.0.15, `vitest` 4.1.10→4.1.11). Volontairement exclu de ce plan : `@vitejs/plugin-react` 4→6, `typescript` 5→7, `vite` 7→8 — sauts majeurs qui mériteraient leur propre tâche avec vérification migration (contexte7) avant de toucher `package.json`, pas un patch mineur silencieux.

- [ ] **Step 1 : Appliquer les patchs**

Run: `npm update @tauri-apps/plugin-opener @types/react @types/react-dom react react-dom zustand vitest`

- [ ] **Step 2 : Vérifier non-régression**

Run: `npx tsc --noEmit && npm test && npm run build`
Expected: 0 erreur TS, 20/20 tests verts, build réussi.

- [ ] **Step 3 : Commit**

```bash
git add package.json package-lock.json
git commit -m "chore: bump patch-level dependencies"
```

---

### Task 5 : Durcir les `unwrap()` non-test dans le code métier Rust

**Files:**
- Modify: `src-tauri/src/inventory/registry.rs`

**Contexte :** `read_entry` fait `name: display_name.unwrap()` — actuellement sûr car `classify_visibility` a déjà retourné `Skip` si `display_name` est `None`, donc le early-return `if visibility == EntryVisibility::Skip { return None; }` garantit la précondition. C'est correct aujourd'hui mais fragile : si quelqu'un modifie un jour `classify_visibility` sans toucher `read_entry`, le panic n'apparaît qu'à l'exécution sur une vraie machine, pas en test. Remplacer par un `?`/`else` explicite documente l'invariant dans le type système plutôt que dans un commentaire.

- [ ] **Step 1 : Remplacer l'unwrap par un retour sûr**

```rust
let Some(name) = display_name else {
    // Invariant garanti par classify_visibility (Skip déjà filtré ci-dessus) ;
    // retour défensif au lieu d'un panic si l'invariant est un jour cassé ailleurs.
    return None;
};
```
puis utiliser `name` à la place de `display_name.unwrap()` dans le `Some(Program { ... })`.

- [ ] **Step 2 : Vérifier**

Run: `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: 15/15 tests verts, 0 warning (comportement identique, juste plus défensif).

- [ ] **Step 3 : Commit**

```bash
git add src-tauri/src/inventory/registry.rs
git commit -m "refactor: defensive fallback instead of unwrap in read_entry"
```

---

### Task 6 : Aligner la documentation sur l'état réel du code

**Files:**
- Modify: `README.md`

**Contexte :** le cahier des charges (§5.2) décrit les modules `uninstaller`, `scanner`, `cleaner`, `safeguard`, `elevation` comme faisant partie de l'architecture cible — normal pour un document de spec, mais un futur contributeur lisant uniquement le README doit savoir que seul `inventory` existe aujourd'hui. Le README a déjà une ligne "État" ; la compléter avec un renvoi explicite.

- [ ] **Step 1 : Ajouter une note d'état dans le README**

Sous la ligne `État : jalon M1 (inventaire) livré. Prochain jalon : M2 (désinstallation standard).`, ajouter :

```markdown
Seul le module `inventory` (backend `src-tauri/src/inventory/`) existe à ce
stade. Les modules `uninstaller`, `scanner`, `cleaner`, `safeguard`,
`elevation` décrits dans `docs/cahier-des-charges.md` (§5.2) sont la cible
des jalons M2 à M5, pas encore implémentés.
```

- [ ] **Step 2 : Commit**

```bash
git add README.md
git commit -m "docs: clarify only the inventory module exists so far"
```

---

## Ce que ce plan ne force pas (et pourquoi)

- **Pas de bump majeur** (`vite` 7→8, `typescript` 5→7, `vitest` 4→5, `@vitejs/plugin-react` 4→6) : chaque saut majeur mérite sa propre vérification (guide de migration via contexte7, build + smoke test réel), pas un patch silencieux glissé dans un plan d'hygiène.
- **Pas de linter ESLint ajouté** : le projet n'en a jamais eu ; l'introduire nécessite un essai calibré (voir audit) pour éviter un mur de faux positifs — hors périmètre d'un plan de nettoyage rapide.
- **Pas de tests E2E / a11y automatisés** : le cahier des charges les mentionne pour M5 (accessibilité, parcours complets), prématuré tant que F3+ n'existe pas.
- **Pas de CI multi-plateforme** : le produit est Windows-only par nature (accès registre Win32) ; une matrice Linux/macOS n'apporterait aucune couverture réelle.

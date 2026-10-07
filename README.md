# BetterUnistaller

Désinstalleur Windows propre : désinstalle, puis nettoie ce que le
désinstalleur officiel laisse derrière lui (fichiers, registre).

- Spécification : `docs/cahier-des-charges.md`
- Plans d'implémentation : `docs/superpowers/plans/`

## Développement

Prérequis : Node.js 20+, Rust stable, outillage Tauri 2 pour Windows.

```powershell
npm install
npm run tauri dev          # application en mode dev
npm test                   # tests frontend (Vitest)
cd src-tauri; cargo test   # tests backend (Rust)
```

État : jalons M1 (inventaire) et F6 (désinstallation forcée avec élévation
UAC) livrés. Prochain jalon : M2 (désinstallation standard).

Modules backend existants : `src-tauri/src/inventory/` (lecture du registre,
tailles, icônes) et `src-tauri/src/uninstaller/` (désinstallation forcée,
point de restauration, élévation UAC, protections). Les modules `scanner`,
`cleaner`, `safeguard` décrits dans `docs/cahier-des-charges.md` (§5.2) restent
la cible des jalons suivants.

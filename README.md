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

État : jalon M1 (inventaire) livré. Prochain jalon : M2 (désinstallation standard).

Seul le module `inventory` (backend `src-tauri/src/inventory/`) existe à ce
stade. Les modules `uninstaller`, `scanner`, `cleaner`, `safeguard`,
`elevation` décrits dans `docs/cahier-des-charges.md` (§5.2) sont la cible
des jalons M2 à M5, pas encore implémentés.

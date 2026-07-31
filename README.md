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

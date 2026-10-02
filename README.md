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

État : jalon M1 (inventaire) + désinstallation forcée (F6) livrés.
Prochain jalon : M2 (désinstallation standard).

Implémenté à ce stade :
- `src-tauri/src/inventory/` — inventaire (F1) : lecture HKLM64/HKLM32/HKCU,
  tailles, icônes ;
- `src-tauri/src/uninstaller/` — désinstallation forcée (F6) : point de
  restauration, arrêt des processus, corbeille + suppression de la clé
  Uninstall, garde-fous (F7 : composants protégés) et élévation UAC avec
  reprise après relance (§5.3).

Les modules `scanner`, `cleaner`, `safeguard` restent la cible des jalons
M3 à M5, pas encore implémentés.

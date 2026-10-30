# AGENTS.md — BetterUnistaller

Règles de référence pour tout agent de code IA travaillant sur ce repo.

## Projet
Désinstalleur Windows propre (Tauri 2 + React/TS) : désinstalle puis nettoie les résidus (fichiers, registre). Spec : `docs/cahier-des-charges.md` ; plans : `docs/superpowers/plans/`. Modules backend : `src-tauri/src/inventory/`, `src-tauri/src/uninstaller/`.

## Commandes
- `npm run tauri dev` — dev
- `npm test` — tests front (Vitest)
- Rust (depuis `src-tauri`) : `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`

## Règles Tauri 2 (non négociables)
- Toute commande IPC Rust est `async fn` et déclarée dans `generate_handler!`.
- Capabilities au moindre privilège : pas d'élargissement de permissions ni de wildcard sans justification dans la PR.
- Ne pas modifier `tauri.conf.json` sans accord explicite du mainteneur.

## Sécurité — criticalité de ce projet
- **Ce logiciel supprime des fichiers et des clés de registre.** Toute suppression doit être précédée d'un point de restauration et passer par les garde-fous existants (composants protégés, allowlist).
- Ne jamais retirer ni affaiblir un garde-fou (protections F7, path guards) sans accord explicite du mainteneur.
- Jamais de secret ni de clé d'API dans le code ; toute nouvelle dépendance (npm ou crate) doit être auditée avant ajout.

## Workflow
- Une tâche = une issue = une petite PR focalisée.
- Spécification courte (contexte + critères d'acceptation) écrite avant de coder.
- CI verte obligatoire avant tout merge ; vérification manuelle (`npm run tauri dev`) requise avant toute release.

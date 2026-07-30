# Cahier des charges — BetterUnistaller

**Version :** 1.0 (brouillon)
**Date :** 31 juillet 2026
**Type d'application :** Application de bureau Windows (Tauri)

---

## 1. Présentation du projet

### 1.1 Contexte

Le panneau « Applications installées » de Windows 11 souffre de limites connues :

- il ne supprime que ce que le désinstalleur du programme veut bien supprimer — fichiers, dossiers et clés de registre résiduels s'accumulent au fil du temps ;
- il ne permet pas de désinstaller plusieurs programmes à la suite sans intervention manuelle entre chaque ;
- il échoue silencieusement sur les programmes dont le désinstalleur est cassé ou manquant ;
- il offre peu d'informations (pas de date d'installation fiable, tailles souvent absentes, pas de tri avancé ni de filtre).

**BetterUnistaller** est une application de bureau qui désinstalle les programmes Windows *proprement* : elle exécute le désinstalleur officiel, puis détecte et propose de supprimer tout ce qu'il a laissé derrière lui, avec des garde-fous adaptés au grand public.

### 1.2 Objectifs

| # | Objectif | Mesure de succès |
|---|----------|------------------|
| O1 | Désinstaller un programme plus proprement que Windows | Après désinstallation + nettoyage, aucun résidu détectable du programme (fichiers, registre) |
| O2 | Être plus rapide à l'usage que le panneau Windows | Désinstaller 5 programmes en une seule action (file d'attente) |
| O3 | Rattraper les cas d'échec de Windows | Un programme au désinstalleur cassé peut être retiré via la désinstallation forcée |
| O4 | Rester sûr pour un utilisateur non technique | Aucune suppression sans confirmation ; point de restauration proposé avant toute opération |

### 1.3 Public cible

**Grand public** : utilisateurs Windows non techniciens souhaitant garder un système propre. Conséquences sur la conception :

- vocabulaire simple, pas de jargon (on affiche « restes du programme », pas « clés HKLM ») ;
- protections activées par défaut (point de restauration, envoi à la corbeille) ;
- les éléments critiques du système ne sont jamais proposés à la suppression ;
- les détails techniques (chemins, clés de registre) restent accessibles mais repliés.

---

## 2. Périmètre

### 2.1 Inclus dans la v1

1. Inventaire complet des programmes installés (Win32, 32 et 64 bits, machine et utilisateur).
2. Recherche, tri et filtres sur la liste.
3. Désinstallation standard (exécution du désinstalleur officiel).
4. Scan des résidus après désinstallation, et nettoyage sur confirmation.
5. Désinstallation par lots (file d'attente de plusieurs programmes).
6. Désinstallation forcée des programmes au désinstalleur cassé.
7. Garde-fous : point de restauration, corbeille, liste de protection, journal des opérations.
8. Interface en français et en anglais, thèmes clair et sombre.

### 2.2 Hors périmètre v1 (candidats v2)

- Applications Microsoft Store / UWP / MSIX et bloatware préinstallé Windows 11.
- Surveillance des installations en temps réel (snapshot avant/après).
- Nettoyage général du système (caches, fichiers temporaires) — l'application reste un désinstalleur, pas un « cleaner » tout-en-un.
- Gestion des extensions de navigateur.
- Version multi-postes / déploiement en entreprise.

---

## 3. Spécifications fonctionnelles

### F1 — Inventaire des programmes

- Sources de données :
  - `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` (64 bits) ;
  - `HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall` (32 bits) ;
  - `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` (installations par utilisateur).
- Informations affichées par programme : nom, éditeur, version, date d'installation, taille (estimée ou calculée depuis `InstallLocation`), icône, emplacement d'installation.
- Les entrées système (`SystemComponent = 1`, mises à jour Windows, redistributables critiques) sont masquées par défaut, affichables via une option.
- Le chargement de la liste doit être asynchrone : l'interface s'affiche immédiatement, les tailles et icônes se complètent en arrière-plan.

### F2 — Recherche, tri et filtres

- Recherche instantanée par nom et par éditeur (tolérante aux accents et à la casse).
- Tris : nom, taille, date d'installation, éditeur.
- Filtres : programmes volumineux (> 1 Go), installés récemment (< 30 jours), rarement identifiables (sans éditeur), par utilisateur vs machine.

### F3 — Désinstallation standard

- Exécution de la `UninstallString` (ou `QuietUninstallString` si l'utilisateur choisit le mode silencieux).
- Prise en charge des désinstalleurs MSI (`msiexec /x {GUID}`) et EXE.
- Détection de fin de désinstallation (fin du processus + disparition de la clé de registre) avec délai de garde pour les désinstalleurs qui se relancent eux-mêmes.
- Gestion des cas : désinstalleur annulé par l'utilisateur, échec (code retour ≠ 0), redémarrage requis (code 3010 MSI).

### F4 — Scan et nettoyage des résidus

Déclenché automatiquement après chaque désinstallation (standard ou par lots), et disponible manuellement pour un programme déjà désinstallé.

- Éléments recherchés, à partir du nom, de l'éditeur et de l'`InstallLocation` du programme :
  - dossiers résiduels dans `Program Files`, `Program Files (x86)`, `%AppData%`, `%LocalAppData%`, `%ProgramData%` ;
  - clés de registre résiduelles (`HKCU\Software`, `HKLM\Software`, entrées de démarrage automatique) ;
  - raccourcis orphelins (bureau, menu Démarrer) ;
  - services Windows et tâches planifiées associés au programme.
- Chaque résidu est présenté avec un **niveau de confiance** :
  - **Sûr** : correspondance exacte avec l'`InstallLocation` ou le nom complet — précoché ;
  - **Probable** : correspondance sur le nom de l'éditeur ou nom partiel — décoché par défaut ;
  - jamais de correspondance « floue » sur des mots courts ou génériques (risque de faux positif).
- Rien n'est supprimé sans confirmation explicite. La sélection affiche la taille totale récupérable.
- Suppression : fichiers envoyés à la corbeille (par défaut) ; clés de registre exportées dans un fichier `.reg` de sauvegarde avant suppression.

### F5 — Désinstallation par lots

- Sélection multiple dans la liste (cases à cocher), ajout à une file d'attente.
- Les désinstallations s'exécutent **séquentiellement** (jamais deux désinstalleurs simultanés — conflits MSI).
- Mode silencieux utilisé quand disponible ; sinon le désinstalleur interactif s'affiche et la file attend.
- Écran de progression : programme en cours, restants, réussis, échoués. Un échec n'interrompt pas la file.
- Le scan des résidus est cumulé et présenté en une seule fois à la fin du lot.

### F6 — Désinstallation forcée

Réservée aux programmes dont la désinstallation standard a échoué ou dont le désinstalleur est absent.

- Accessible uniquement après un échec constaté, ou via une action explicite « Désinstallation forcée » avec avertissement clair.
- Opérations : arrêt des processus du programme, suppression de l'`InstallLocation`, suppression de la clé `Uninstall`, puis scan de résidus complet (F4).
- Point de restauration **obligatoire** (non désactivable) avant toute désinstallation forcée.

### F7 — Garde-fous et sécurité des opérations

- **Point de restauration système** : proposé (précoché) avant toute désinstallation ; obligatoire pour la désinstallation forcée. Au plus un point créé par session de 24 h (limite Windows).
- **Liste de protection** intégrée et non modifiable : composants Windows, pilotes, runtimes critiques (Visual C++ Redistributable, .NET, Edge WebView2 — dont dépend l'application elle-même), antivirus. Ces entrées ne sont jamais proposées à la désinstallation forcée ni au nettoyage.
- **Corbeille par défaut** pour les fichiers ; sauvegarde `.reg` pour le registre.
- **Journal des opérations** : chaque désinstallation et chaque nettoyage est consigné (date, programme, éléments supprimés, résultat), consultable dans l'application et exportable en texte.

### F8 — Fonctions annexes

- Export de la liste des programmes (CSV / texte) — utile pour support ou réinstallation.
- Ouverture directe : dossier d'installation, page de l'éditeur, entrée de registre (pour utilisateurs curieux — replié dans un panneau « Détails techniques »).
- Paramètres : langue, thème, comportement du point de restauration, corbeille vs suppression définitive, affichage des entrées système.

---

## 4. Spécifications non fonctionnelles

| Domaine | Exigence |
|---------|----------|
| Performance | Affichage de la liste < 2 s sur un système avec 200 programmes ; scan de résidus d'un programme < 15 s ; interface fluide pendant les opérations (tout le travail lourd hors du thread UI) |
| Système | Windows 11 (cible principale) et Windows 10 21H2+ ; x64 et ARM64 |
| Privilèges | Lancement en utilisateur standard ; élévation UAC demandée uniquement au moment d'une opération qui l'exige (manifeste `asInvoker` + relance élevée) |
| Empreinte | Exécutable < 15 Mo (hors WebView2), mémoire au repos < 150 Mo |
| Langues | Français et anglais (architecture i18n extensible) |
| Accessibilité | Navigation clavier complète, contrastes AA, tailles de texte respectant les réglages système |
| Confidentialité | Aucune télémétrie, aucune connexion réseau sauf vérification de mise à jour (désactivable) |
| Fiabilité | Aucune perte de données système possible : toute suppression est réversible (corbeille, `.reg`, point de restauration) sauf choix contraire explicite |

---

## 5. Architecture technique

### 5.1 Stack

| Couche | Choix | Justification |
|--------|-------|---------------|
| Framework applicatif | **Tauri 2.x** | Exigence du projet ; binaire léger, backend Rust adapté aux accès système |
| Backend | **Rust** — crates `windows` (Win32), `winreg` (registre), `trash` (corbeille), `serde` (sérialisation) | Accès natif registre/processus/fichiers, sûreté mémoire |
| Frontend | **React 18 + TypeScript + Vite** | Écosystème mature, typage strict des données échangées avec Rust |
| État frontend | Zustand (léger) + TanStack Query pour les données issues du backend | Simplicité, cache des invocations |
| UI | Tailwind CSS + composants Radix UI | Accessibilité native, thème clair/sombre |

### 5.2 Découpage

```
┌─────────────────────────── Frontend (React/TS) ───────────────────────────┐
│  Liste programmes │ Détail │ File d'attente │ Résultats scan │ Paramètres │
└───────────────┬───────────────────────────────────────▲───────────────────┘
                │ invoke (commands)                      │ events (progression)
┌───────────────▼───────────────────────────────────────┴───────────────────┐
│                            Backend (Rust / Tauri)                         │
│  inventory   : lecture registre, tailles, icônes                          │
│  uninstaller : exécution désinstalleurs, file d'attente, détection fin    │
│  scanner     : détection résidus (fichiers, registre, services, tâches)   │
│  cleaner     : suppression sécurisée (corbeille, export .reg)             │
│  safeguard   : point de restauration, liste de protection, journal        │
│  elevation   : relance élevée / exécution des opérations UAC              │
└───────────────────────────────────────────────────────────────────────────┘
```

Chaque module Rust a une responsabilité unique et une interface testable indépendamment de Tauri. Le frontend ne manipule jamais directement le système : toute opération passe par une commande typée.

### 5.3 Points d'attention techniques

- **Élévation** : les lectures (inventaire, scan) fonctionnent sans droits admin ; les suppressions machine (HKLM, Program Files) et les points de restauration exigent l'élévation. Stratégie retenue : relance élevée de l'application sur demande, avec reprise du contexte (opération en attente sérialisée).
- **Détection de fin de désinstallation** : certains désinstalleurs EXE lancent un processus fils et se terminent immédiatement — surveiller l'arbre de processus et la clé de registre, avec délai de garde.
- **Faux positifs du scan** : le moteur de correspondance est le composant le plus risqué du produit. Il doit être conservateur (voir F4) et couvert par une suite de tests exhaustive sur des cas réels.
- **Portable vs installé** : la version portable stocke paramètres et journaux à côté de l'exécutable ; la version installée dans `%AppData%\BetterUnistaller`.

---

## 6. Interface utilisateur

### 6.1 Écrans

1. **Liste des programmes** (écran principal) : barre de recherche, filtres, tri, cases de sélection multiple, bouton « Désinstaller » contextuel (1 ou N programmes).
2. **Panneau de détail** : informations complètes du programme, actions (désinstaller, ouvrir le dossier, détails techniques repliés).
3. **Assistant de désinstallation** : confirmation → option point de restauration → progression → résultats du scan de résidus → confirmation du nettoyage → récapitulatif.
4. **File d'attente** (mode lots) : progression globale et par programme, erreurs non bloquantes.
5. **Journal** : historique des opérations, export.
6. **Paramètres** : langue, thème, garde-fous.

### 6.2 Principes

- Un parcours unique et guidé : l'assistant enchaîne désinstallation puis nettoyage sans que l'utilisateur ait à comprendre la différence.
- Les actions destructives sont toujours au même endroit, jamais en action par défaut d'un double-clic.
- Ton rassurant : chaque écran de confirmation explique ce qui va se passer et comment revenir en arrière.

---

## 7. Distribution et mises à jour

- **Installeur NSIS** signé (signature de code indispensable pour un outil qui demande l'élévation — sinon SmartScreen bloquera l'adoption).
- **Version portable** : archive ZIP contenant l'exécutable, mode détecté automatiquement.
- **Mises à jour** : Tauri Updater pour la version installée (vérification au lancement, désactivable) ; la version portable notifie sans installer.
- WebView2 : requis par Tauri, préinstallé sur Windows 11 ; l'installeur embarque le bootstrapper pour Windows 10.

---

## 8. Tests et critères d'acceptation

### 8.1 Stratégie de test

- **Tests unitaires Rust** : moteur de correspondance des résidus (priorité absolue — jeu de cas réels avec faux positifs connus), parsing du registre, file d'attente.
- **Tests d'intégration** : désinstallation réelle de programmes témoins (MSI et EXE) dans une VM Windows 11 jetable, vérification de l'absence de résidus.
- **Tests E2E frontend** : parcours complets via WebDriver/tauri-driver.
- **Tests manuels** : matrice Windows 10/11, x64/ARM64, utilisateur standard/admin, installé/portable.

### 8.2 Critères d'acceptation de la v1

1. Désinstaller un programme MSI et un programme EXE témoins, avec nettoyage : aucun résidu détectable ensuite.
2. Désinstaller 5 programmes en file d'attente sans intervention (mode silencieux disponible) ; un échec au milieu n'arrête pas la file.
3. Supprimer un programme au désinstalleur volontairement cassé via la désinstallation forcée, avec point de restauration créé.
4. Le scan ne propose jamais un élément appartenant à un autre programme installé (0 faux positif sur le jeu de test).
5. Toute suppression est annulable : fichiers restaurables depuis la corbeille, registre restaurable depuis le `.reg`.
6. L'application se lance et affiche la liste sans droits administrateur.

---

## 9. Risques

| Risque | Impact | Mitigation |
|--------|--------|------------|
| Faux positif du scan → suppression d'un fichier d'un autre programme | Critique | Moteur conservateur, niveaux de confiance, corbeille par défaut, tests exhaustifs |
| SmartScreen / antivirus signalent l'application (outil manipulant le registre) | Fort | Signature de code (certificat EV si possible), soumission aux éditeurs AV |
| Désinstalleurs exotiques (fin non détectable, relances multiples) | Moyen | Délai de garde, vérification par la clé de registre, timeout avec message clair |
| Limite Windows d'un point de restauration par 24 h | Moyen | Détection et information de l'utilisateur, sauvegarde `.reg` en complément |
| Évolutions du registre/API entre versions de Windows | Faible | CI sur images Windows 10 et 11 |

---

## 10. Jalons indicatifs

| Jalon | Contenu |
|-------|---------|
| M1 — Socle | Projet Tauri + React, inventaire des programmes (F1, F2), UI liste + détail |
| M2 — Désinstallation | Désinstallation standard (F3), assistant, journal (F7 partiel) |
| M3 — Nettoyage | Scan et nettoyage des résidus (F4), garde-fous complets (F7) |
| M4 — Puissance | Lots (F5), désinstallation forcée (F6) |
| M5 — Finition | i18n, accessibilité, paramètres, installeur + portable, signature, bêta |

---

## 11. Glossaire

- **Résidu** : fichier, dossier, clé de registre, raccourci, service ou tâche planifiée laissé sur le système après la désinstallation officielle d'un programme.
- **Désinstallation forcée** : suppression manuelle des composants d'un programme sans passer par son désinstalleur.
- **Point de restauration** : instantané Windows permettant de ramener le système (fichiers système et registre) à un état antérieur.
- **UAC** : mécanisme Windows de demande d'élévation de privilèges administrateur.

# Architecture technique cible

## 1. Principes

Construire un **monolithe modulaire** Rust : un seul exécutable, des frontières internes strictes et des interfaces permettant de remplacer les infrastructures.

Le domaine ne doit pas dépendre de Ratatui, SQLite, Docker/Podman, Tokio, d'un hyperviseur ou d'un fournisseur IA.

## 2. Couches

### Domain

Contient les règles métier pures :
- Profile, Progression, XP, Skill, Achievement.
- Course, LearningPath, Exercise, Scenario.
- Lab, Machine, NetworkTopology, Snapshot.
- ScriptProject, GitRepository, Note, Goal.
- AIConversation, AIProviderConfig (sans secrets), ModelCapability.
- BackupMetadata, Notification.

Pas d'accès disque, réseau, terminal, base de données ou API externe.

### Application

Cas d'utilisation et orchestration :
- CreateProfile, LoadDashboard, CompleteExercise.
- StartLab, StopLab, ResetLab, SnapshotLab.
- ImportScenario, ValidateScenario, ExportScenario.
- CreateBackup, RestoreBackup.
- CloneRepository, PullRepository, CommitRepository, PushRepository.
- GenerateAIResponse, GenerateScenario, ValidateAIOutput.
- SearchContent, UpdateProgress, ManageGoals.

Définit les ports (traits) dont elle a besoin.

### Infrastructure

Adaptateurs concrets :
- SQLite + migrations.
- Docker et Podman.
- PTY et gestion des shells.
- Réseau de labos.
- Hyperviseurs et formats VM.
- Stockage et archives ZIP/Zstd.
- Git.
- Fournisseurs IA HTTP/local.
- Coffre de secrets OS.
- Mises à jour GitHub Releases.
- Logs et diagnostics.

### Presentation

- Ratatui : widgets, écrans, navigation, événements clavier/souris.
- Gestion des états d'écran et des modales.
- Terminal intégré et rendu ANSI.
- Panneaux adaptatifs.
- Localisation des chaînes.

La présentation appelle les cas d'utilisation ; elle ne contient pas les règles métier.

## 3. Structure indicative

```text
tssrlabs/
├── Cargo.toml
├── crates/
│   ├── tssrlabs-domain/
│   ├── tssrlabs-application/
│   ├── tssrlabs-infrastructure/
│   └── tssrlabs-presentation/
├── migrations/
├── content/
│   ├── courses/
│   └── scenarios/
├── assets/
├── docs/
└── tests/
```

Une workspace Cargo multi-crates est recommandée si elle apporte une séparation claire ; ne pas multiplier les crates artificiellement. Une structure `src/` modulaire peut être utilisée au départ si elle conserve les mêmes frontières.

## 4. Asynchronisme et cycle de vie

- Tokio pour les opérations réseau, IA, téléchargement et orchestration asynchrone.
- Aucun appel bloquant dans le thread de rendu Ratatui.
- Opérations bloquantes (PTY, processus, I/O lourdes, compression) isolées dans des threads ou `spawn_blocking` avec limites.
- Chaque tâche possède un propriétaire clair, un mécanisme d'annulation et une stratégie d'arrêt.
- À la fermeture : annuler les tâches, fermer PTY, arrêter proprement les ressources gérées, flush SQLite/logs.
- Ne pas laisser de tâches orphelines ni de processus enfants détachés.

## 5. Gestion des erreurs

- Erreurs typées par couche.
- Contexte utile sans divulguer secrets ou données sensibles.
- Distinguer erreurs récupérables, incompatibilités, absence de dépendance et erreurs de sécurité.
- Logs structurés locaux avec redaction.
- UI avec message compréhensible et détails techniques consultables.

## 6. Stockage local

SQLite versionné :
- Transactions explicites.
- Contraintes et index.
- Migrations testées depuis les versions précédentes.
- Pas de clés API ou tokens en clair.
- Les données volumineuses (archives, disques VM, enregistrements) restent en fichiers ; SQLite conserve métadonnées et index.

## 7. Capabilities

Créer un modèle explicite de capacités de l'hôte :
- OS, architecture CPU.
- Terminal et fonctionnalités ANSI.
- Docker/Podman et versions.
- Hyperviseurs détectés.
- Outils facultatifs (ShellCheck, PSScriptAnalyzer, qemu-img, Git).
- Coffre de secrets.
- Réseau et snapshots supportés.

Les fonctionnalités sont activées en fonction des capacités constatées, pas uniquement de l'OS déclaré. Chaque backend expose ses limites.

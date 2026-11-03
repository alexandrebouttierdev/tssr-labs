# Qualité, tests et critères de livraison

## Tests requis

### Unitaires
- règles XP et progression ;
- prérequis et déblocage ;
- calculs de maîtrise ;
- validation de scénarios ;
- règles de compatibilité ;
- logique de rétention ;
- gestion des capacités.

### Intégration
- SQLite et migrations ;
- import/export `.tssrlab` ;
- sauvegarde/restauration ;
- Git et coffre secrets via doubles ou environnements dédiés ;
- orchestration Docker/Podman ;
- PTY ;
- fournisseurs IA mockés ;
- mise à jour et vérification de signature.

### Sécurité
- traversal ZIP ;
- archives bombes ;
- liens symboliques et chemins malveillants ;
- injection shell ;
- secrets dans logs ;
- scénarios malformés ;
- prompt injection dans contenu externe ;
- permissions et limites de conteneurs ;
- absence de montage du socket hôte ;
- annulation et nettoyage des tâches/processus.

### End-to-end
- création profil → dashboard ;
- cours → exercice → validation → XP ;
- import scénario → démarrage labo → commande réelle → validation ;
- arrêt/reprise labo ;
- sauvegarde → restauration ;
- génération IA → validation → confirmation import.

## CI

- Linux, macOS, Windows.
- `cargo fmt --check`.
- `cargo clippy` avec politique raisonnable.
- `cargo test`.
- Vérification des dépendances et licences.
- Builds de release.
- Tests d'intégration dépendants de Docker/Podman dans jobs dédiés, sans bloquer les tests purement locaux.

## Qualité du code

- Types explicites et erreurs contextualisées.
- Pas de `unwrap()`/`expect()` dans les chemins d'exécution utilisateur sauf invariant documenté.
- Pas de logique métier dans les widgets Ratatui.
- Pas de commande shell construite par concaténation de chaînes.
- Pas de secrets dans les logs.
- Pas de tâches asynchrones sans propriétaire/annulation.
- Interfaces seulement lorsqu'elles représentent une frontière utile ; éviter l'abstraction prématurée.
- Documentation des limites OS/backend.

## Definition of Done

Une fonctionnalité est terminée lorsque :
- le comportement est documenté ;
- les erreurs et états indisponibles sont traités ;
- les tests pertinents existent ;
- le code compile sur les cibles prévues ;
- aucune donnée ou action destructive n'est silencieuse ;
- les dépendances facultatives sont détectées ;
- l'interface reste utilisable en mode compact ;
- les logs ne divulguent aucun secret.

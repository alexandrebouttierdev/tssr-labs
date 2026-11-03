# Feuille de route

Le périmètre cible est large. Ne pas tenter de tout implémenter simultanément. Chaque phase doit laisser le dépôt compilable, testable et utilisable.

## Phase 0 — Socle du dépôt

- Workspace Rust et conventions.
- CI Linux/macOS/Windows.
- Configuration, logs locaux et gestion d'erreurs.
- Boucle Ratatui, navigation clavier et thème.
- Détection terminal/OS/architecture.
- SQLite et migrations.
- Profils locaux.

**Livrable :** application qui démarre, crée/choisit un profil, affiche un dashboard vide et sauvegarde ses préférences.

## Phase 1 — Noyau pédagogique

- Modèle de cours, parcours, exercices et scénarios.
- Catalogue local embarqué.
- Recherche et filtres.
- Exercices guidés : QCM, commandes à compléter, analyse de sortie.
- Progression XP, niveaux, compétences et historique.
- Favoris, « À reprendre », notes.
- Dashboard réel.

**Livrable :** parcours pédagogique utilisable sans Docker, réseau ni IA.

## Phase 2 — Scénarios et laboratoires Linux

- Schéma JSON versionné.
- Archive `.tssrlab`.
- Import/export et validation.
- Détection Docker/Podman.
- Adaptateurs initiaux.
- Images Debian/Ubuntu/Rocky à la demande.
- Lab persistant et terminal PTY Bash.
- Réseau isolé et adressage défini par scénario.
- Validation de l'état final.
- Limites ressources et arrêt automatique.

**Livrable :** premier scénario de dépannage Linux complet, réinitialisable et testé.

## Phase 3 — Éditeur et outils

- Workspace de scripts multi-fichiers.
- Coloration, recherche, analyse ShellCheck.
- Exécution isolée dans un labo.
- Historique, recherche terminal et presse-papiers.
- Enregistrement/relecture structurée des sessions.
- Git intégré et coffre de secrets OS.

**Livrable :** espace de travail Bash fonctionnel.

## Phase 4 — IA multi-fournisseurs

- Configuration assistée et avancée.
- Coffre de secrets.
- Découverte et capacités des modèles.
- Modèle par fonctionnalité.
- Chat streaming.
- Sélection explicite du contexte et aperçu du payload.
- Détection de secrets.
- Diff obligatoire pour modifications proposées.
- Mémoire pédagogique configurable.
- Suivi local des tokens/coûts.
- Génération de scénarios avec validation et tests isolés.

**Livrable :** IA facultative et sûre, sans dépendance du cœur.

## Phase 5 — Sauvegardes, snapshots et mises à jour

- Racine de stockage configurable.
- Snapshots selon capacités backend.
- Archives Zstd, destinations multiples.
- Vérification, prévisualisation, backup avant restauration et rollback.
- Stable/beta, signature, SHA-256 et rollback de version.

## Phase 6 — Windows et réseaux avancés

- PowerShell 7 isolé.
- Import de VM Windows existantes.
- Détection hyperviseurs et formats.
- Topologies multi-machines.
- DNS/DHCP/routage et scénarios réseau.
- Visualisation topologie Ratatui.

## Phase 7 — Contenu complet et certification blanche

- Catalogue TSSR complet.
- Évaluations par chapitre.
- Simulateur d'examen chronométré.
- Packs de contenus téléchargeables.
- Sources officielles versionnées et dates de vérification.

## Critères de sortie de chaque phase

- Build sans erreur sur les plateformes cibles concernées.
- Tests automatisés pertinents.
- Documentation mise à jour.
- Aucun secret dans le dépôt.
- Aucun contournement des frontières de sécurité.
- Démonstration de bout en bout reproductible.

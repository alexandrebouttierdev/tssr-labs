# Product brief — TSSR Labs

## 1. Vision

Créer une plateforme TUI professionnelle et pédagogique permettant de progresser dans les métiers systèmes et réseaux. L'application combine apprentissage théorique, exercices guidés, véritables terminaux isolés, laboratoires réalistes, progression RPG et outils de création de contenu.

L'expérience doit évoquer un outil professionnel de technicien/administrateur système, avec une progression ludique discrète. Éviter une esthétique de jeu excessive.

## 2. Public et objectifs

Public principal : personnes en formation TSSR, apprenants en administration systèmes et réseaux et personnes souhaitant pratiquer en autonomie.

Objectifs :
- Comprendre les concepts systèmes et réseaux.
- S'entraîner avec de vraies commandes Bash et PowerShell.
- Diagnostiquer et corriger des pannes réalistes.
- Acquérir des automatismes de dépannage.
- Suivre ses compétences et sa progression.
- Créer, modifier, importer et partager des scénarios pédagogiques.

## 3. Domaines pédagogiques

Le catalogue initial doit couvrir progressivement :
- Linux et Bash.
- Windows et PowerShell.
- Réseaux : TCP/IP, adressage, sous-réseaux, DNS, DHCP, VLAN, routage.
- Systèmes : stockage, processus, journaux, services, permissions.
- Sécurité : SSH, pare-feu, durcissement, gestion des accès.
- Virtualisation et conteneurs.
- Diagnostic et dépannage.
- Automatisation : Bash, PowerShell et Ansible.

Chaque chapitre comprend idéalement :
1. Théorie et objectifs.
2. Concepts essentiels.
3. Commandes et exemples.
4. Exercices guidés.
5. Laboratoires pratiques.
6. Évaluation.

Les contenus sont originaux et accompagnés de références officielles, versions et dates de vérification. Le parcours TSSR doit s'appuyer sur le référentiel officiel applicable, sans prétendre délivrer une certification officielle.

## 4. Organisation de l'apprentissage

Deux modes complémentaires :
- **Parcours guidés** : chapitres, objectifs, prérequis, progression et déblocage.
- **Exploration libre** : bibliothèque filtrable par domaine, difficulté, durée, type et mots-clés.

La progression obtenue en exploration libre peut contribuer aux parcours guidés lorsque le contenu correspond.

## 5. Types d'activités

Exercices guidés :
- QCM.
- Compléter une commande.
- Corriger un script.
- Rédiger une commande.
- Analyser une sortie terminal.

Activités pratiques :
- Terminal Bash réel dans un environnement Linux isolé.
- PowerShell 7 isolé.
- Laboratoires multi-machines.
- Dépannage avec état initial volontairement dégradé.
- Vérification automatique des résultats.

## 6. Progression RPG

- XP global et niveaux.
- Arbre de compétences par domaine.
- Maîtrise par compétence indépendante du niveau global.
- Succès/trophées, historique, séries d'activité et statistiques.
- Récompenses adaptatives selon difficulté, objectifs et indices.
- XP partiel lorsque certains objectifs sont atteints.
- Bonus de première réussite et éventuellement sans indice.
- Les tentatives répétées ne doivent pas pénaliser excessivement l'apprenant.

## 7. Tableau de bord

Écran d'accueil central :
- Niveau et barre d'XP.
- Vue des compétences.
- Exercices recommandés.
- Accès rapide aux laboratoires.
- Activité récente et statistiques.
- Favoris et exercices « À reprendre ».
- Objectifs quotidiens et hebdomadaires facultatifs.
- Notifications internes uniquement.

## 8. Profils et données personnelles

- Aucun compte distant et aucune inscription obligatoire.
- Profils locaux multiples.
- SQLite pour progression, paramètres, historique, notes, conversations IA et métadonnées.
- Notes personnelles par scénario/exercice.
- Favoris, reprise d'activités et objectifs personnels.
- Statistiques visibles dans l'application uniquement ; pas d'export statistique dédié.
- Sauvegarde/restauration locale complète.

## 9. Plateformes

Linux, macOS et Windows dès le MVP pour le binaire et les fonctions communes. Les fonctionnalités avancées sont conditionnées par les capacités de l'hôte.

Les laboratoires Windows en VM dépendent du système hôte, de l'architecture, de l'hyperviseur et des licences. L'application doit présenter clairement les prérequis.

## 10. Langue et distribution

- Interface initiale en français.
- Architecture internationalisable, anglais possible ultérieurement.
- Binaires autonomes précompilés pour Linux, macOS Intel/Apple Silicon selon faisabilité, Windows.
- Pas besoin de Rust/Cargo pour l'utilisateur final.
- Licence du code : MIT.
- Chaque scénario conserve sa propre licence et ses crédits.

# UX et interface terminal

## Direction visuelle

- TUI premium, sombre par défaut, nette et professionnelle.
- Hiérarchie visuelle forte, espaces maîtrisés, bordures sobres.
- Couleurs limitées et cohérentes ; ne jamais utiliser la couleur comme seul indicateur.
- Progression RPG discrète, sans transformer l'outil en jeu vidéo.
- Éviter l'apparence d'un tableau de bord web transposé dans un terminal.

## Navigation

- Clavier prioritaire, raccourcis visibles dans les écrans.
- Souris facultative.
- Raccourcis configurables et sensibles aux dispositions AZERTY/QWERTY.
- Navigation cohérente entre dashboard, cours, exercices, labs, éditeur, Git, IA et paramètres.
- Modales courtes, confirmations claires pour les opérations destructives.

## Écrans principaux

1. Dashboard.
2. Parcours pédagogiques.
3. Bibliothèque/exploration.
4. Détail chapitre/scénario.
5. Exercice guidé.
6. Laboratoire split-view.
7. Topologie réseau.
8. Éditeur de scripts.
9. Gestionnaire Git.
10. Assistant IA.
11. Progression et statistiques.
12. Notes et favoris.
13. Sauvegardes et snapshots.
14. Paramètres et diagnostic.

## Laboratoire split-view

- Panneau pédagogique : objectif, consignes, indices, récompense, état de validation.
- Terminal interactif réel.
- Plusieurs terminaux par onglets et identification machine/shell.
- La taille du terminal détermine la disposition : deux colonnes si possible, empilement ou panneaux alternés en mode compact.
- Le terminal doit conserver son comportement PTY/ANSI, y compris les applications interactives.

## Responsive terminal

- Référence : 100 colonnes × 30 lignes.
- Sous cette taille, passer en mode compact, réduire les éléments secondaires et préserver les actions essentielles.
- Ne jamais laisser le contenu déborder.
- Si l'espace devient insuffisant pour une opération sûre, afficher une explication plutôt que tronquer silencieusement.

## Accessibilité

- Taille de texte réglable.
- Contraste renforcé et palettes adaptées au daltonisme.
- Animations désactivables.
- Navigation complète au clavier.
- Étiquettes textuelles en plus des couleurs/icônes.
- Compatibilité lecteur d'écran dans la limite des capacités terminal/OS.

## États UX obligatoires

Chaque fonctionnalité externe doit représenter explicitement :
- Disponible.
- En cours.
- Indisponible.
- Dépendance manquante.
- Non supporté par l'hôte.
- Erreur récupérable.
- Erreur nécessitant intervention.

Ne pas afficher une action comme disponible si le backend ne peut pas réellement l'exécuter.

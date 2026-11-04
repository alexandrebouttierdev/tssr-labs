# TSSR Labs

TSSR Labs est une plateforme pédagogique en terminal, développée en **Rust + Ratatui**, destinée à apprendre, pratiquer et évaluer les compétences TSSR à travers des cours, exercices guidés, laboratoires Linux/Windows et outils d'administration système et réseau.

Ce dépôt contient le cadrage initial, les décisions produit et techniques, une feuille de route et un prompt maître destiné à Claude Code.

## Documents

- [`docs/PRODUCT_BRIEF.md`](docs/PRODUCT_BRIEF.md) — vision, utilisateurs, périmètre et fonctionnalités.
- [`docs/DECISIONS.md`](docs/DECISIONS.md) — décisions produit et contraintes validées.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — architecture cible Rust, couches et modules.
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — phases de développement et priorités du MVP.
- [`docs/UX_TERMINAL.md`](docs/UX_TERMINAL.md) — principes d'interface Ratatui.
- [`docs/SCENARIO_FORMAT.md`](docs/SCENARIO_FORMAT.md) — format de scénario `.tssrlab`.
- [`docs/CONTENT_FORMAT.md`](docs/CONTENT_FORMAT.md) — format des cours embarqués (`content/`).
- [`docs/LABS_AND_ISOLATION.md`](docs/LABS_AND_ISOLATION.md) — exécution des laboratoires et sécurité.
- [`docs/AI_INTEGRATION.md`](docs/AI_INTEGRATION.md) — fournisseurs, confidentialité et usages IA.
- [`docs/DATA_AND_BACKUPS.md`](docs/DATA_AND_BACKUPS.md) — SQLite, profils, sauvegardes et snapshots.
- [`docs/TESTING_AND_QUALITY.md`](docs/TESTING_AND_QUALITY.md) — stratégie de tests et critères de qualité.
- [`docs/IMPLEMENTATION_PLAN.md`](docs/IMPLEMENTATION_PLAN.md) — plan par tranches, état d'avancement et limites connues.
- [`prompts/CLAUDE_CODE_MASTER_PROMPT.md`](prompts/CLAUDE_CODE_MASTER_PROMPT.md) — prompt de démarrage pour Claude Code.

## Principes non négociables

1. Le cœur pédagogique doit fonctionner sans IA et sans moteur de conteneurs.
2. Aucune commande d'apprenant ne doit être exécutée directement sur l'hôte.
3. Aucun contenu externe (scénario, dépôt Git, script, fichier) ne doit être considéré comme une instruction fiable pour l'IA.
4. Aucun envoi de contexte à un fournisseur IA sans sélection et confirmation explicites.
5. Aucun écrasement ou changement destructif de données sans confirmation et mécanisme de récupération approprié.
6. Les fonctionnalités dépendantes de l'OS ou de l'hyperviseur doivent déclarer leurs capacités réelles, sans simuler une prise en charge.
7. Le MVP doit rester réalisable : construire d'abord un noyau fonctionnel, puis étendre progressivement.

## Démarrage recommandé

1. Lire tous les documents de `docs/`.
2. Utiliser le prompt maître avec Claude Code.
3. Demander à Claude Code de commencer par inspecter le dépôt et proposer une structure initiale, sans générer tout le produit en une seule passe.
4. Développer par tranches verticales testées et compilables.

## Développement

État actuel : **tranche 3 (contenu pédagogique hors ligne)** — profils et préférences, parcours guidés, bibliothèque avec recherche et filtres, lecteur de chapitres. Premier parcours : « Linux — les bases ».

Prérequis : [rustup](https://rustup.rs). La toolchain est épinglée dans `rust-toolchain.toml` et installée automatiquement.

```sh
cargo run                      # lance l'interface (q ou Échap pour quitter)
cargo run -- --paths           # affiche les répertoires de données, configuration et journaux
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release          # binaire : target/release/tssrlabs
```

Variables d'environnement :

- `TSSRLABS_HOME` : regroupe données, configuration et journaux sous un seul répertoire (installation portable, tests).
- `TSSRLABS_LOG` : niveau des journaux locaux (`info` par défaut ; ex. `debug`).
- `TSSRLABS_COLORS` : impose le niveau de couleurs (`none`, `16`, `256`, `truecolor`) si la détection automatique se trompe.
- `NO_COLOR` : désactive les couleurs ([no-color.org](https://no-color.org)).

Raccourcis : `Tab`/`Maj+Tab` changer d'écran, `t` thème, `F1` ou `?` aide, `q` quitter ; sur l'écran Profils : `Entrée` activer, `n` créer, `r` renommer, `s` ou `Suppr` supprimer ; dans la Bibliothèque : `/` rechercher, `d` domaine, `f` niveau, `m` durée, `x` effacer, `Entrée` ouvrir ; dans un chapitre : flèches, `PgSuiv`/`Espace`, `Début`/`Fin`, `Échap` retour. `F1` affiche les raccourcis de l'écran courant. L'affichage complet demande 100×30 ; le mode compact s'active jusqu'à 60×16.

Les journaux sont uniquement locaux ; les secrets reconnaissables y sont masqués. Aucune donnée n'est envoyée sur le réseau.

Données locales : base SQLite `tssrlabs.sqlite3` dans le répertoire de données (voir `tssrlabs --paths`). Avant une migration de schéma ou la suppression d'un profil, une copie de sécurité est créée dans `copies-de-securite/` ; pour restaurer, fermer l'application et remplacer `tssrlabs.sqlite3` par la copie voulue.

## Licence

Code sous licence [MIT](LICENSE). Chaque scénario conserve sa propre licence et ses crédits.

# Plan d'implémentation

Ce document traduit la [feuille de route](ROADMAP.md) en tranches verticales
compilables et démontrables. Il est mis à jour à la fin de chaque tranche.
Les décisions de cadrage restent dans [DECISIONS.md](DECISIONS.md).

## 1. Structure du code

Workspace Cargo ; chaque couche est une crate afin que le compilateur garantisse
le sens des dépendances (le domaine ne peut pas dépendre de Tokio, Ratatui ou
SQLite).

```text
crates/
├── tssrlabs-domain/          # règles métier pures (modèle de l'hôte, puis profils…)
├── tssrlabs-application/     # cas d'utilisation + ports       (tranche 2)
├── tssrlabs-infrastructure/  # adaptateurs : chemins, journaux, SQLite, conteneurs…
├── tssrlabs-tui/             # présentation Ratatui (architecture de type Elm)
└── tssrlabs/                 # binaire : racine de composition, runtime Tokio
```

Sens des dépendances : `tssrlabs` → (`tssrlabs-tui`, `tssrlabs-infrastructure`)
→ `tssrlabs-application` → `tssrlabs-domain`. La présentation ne dépend pas de
l'infrastructure : c'est le binaire qui les assemble.

Une crate n'est créée que lorsqu'elle a un contenu réel : `domain` existe depuis
la tranche 1 (modèle de l'hôte), `application` arrivera avec les premiers cas
d'utilisation (profils, tranche 2).

Les textes de l'interface sont dans `crates/tssrlabs-tui/i18n/<langue>/*.ftl`
(Fluent), embarqués dans le binaire ; chaque identifiant est déclaré dans
`crates/tssrlabs-tui/src/i18n.rs` et vérifié par les tests.

### Principes d'exécution

- Boucle d'interface : état (`App`), `Action`, `update()`, `render()`. Le rendu
  est pur ; la boucle ne fait jamais d'opération longue.
- Les tâches de fond sont lancées via un `TaskTracker` et arrêtées via un
  `CancellationToken` ; leurs résultats reviennent par canal.
- SQLite (à partir de la tranche 2) : un thread dédié possède la connexion.
- Labos : l'exec Docker/Podman avec TTY alloue un **vrai PTY dans le
  conteneur** ; l'application émule seulement le terminal (`vt100`). Aucun PTY
  ni aucune commande d'apprenant côté hôte.

## 2. Politique de dépendances

- Une dépendance n'est ajoutée qu'avec la tranche qui l'utilise.
- `cargo-deny` vérifie licences, vulnérabilités et sources en CI (`deny.toml`).
- Préférer les crates pures Rust pour faciliter les binaires autonomes.

| Besoin | Choix | Tranche |
|---|---|---|
| Interface | `ratatui` 0.30, `crossterm` 0.29 | 0 |
| Async | `tokio`, `tokio-util`, `futures-util` | 0 |
| Erreurs | `thiserror` | 0 |
| Journaux | `tracing`, `tracing-subscriber`, `tracing-appender`, `regex` (masquage) | 0 |
| Chemins | `directories` | 0 |
| i18n | `fluent-bundle`, `unic-langid` | 1 |
| SQLite | `rusqlite` (bundled), `rusqlite_migration` | 2 |
| Dates | `jiff` | 2 |
| Contenu | `serde`, `serde_json`, `pulldown-cmark`, `include_dir` | 3 |
| Archives `.tssrlab` | `zip`, `sha2`, `semver` | 6 |
| Conteneurs | `bollard` | 7 |
| Émulation terminal | `vt100` (+ `tui-term` ou rendu maison) | 9 |
| Plus tard | `keyring`, `secrecy`, `reqwest` (rustls), `zstd`, `minisign-verify`, `arboard`, `git` système | 3+ |

## 3. Tranches du MVP

| # | Tranche | Démonstration | État |
|---|---|---|---|
| 0 | Socle : workspace, lints, CI 3 OS, licence MIT, restauration du terminal (y compris panique), journaux locaux avec masquage | L'application démarre et quitte proprement | **Terminée** |
| 1 | Boucle d'événements, navigation, thème sombre + contraste élevé, responsive 100×30 / compact / « terminal trop petit », aide, i18n FR (Fluent), écran Diagnostic | Navigation et responsive | **Terminée** |
| 2 | SQLite (thread dédié, migration v1), profils (créer/choisir/renommer/supprimer avec confirmation), préférences | Livrable phase 0 | À faire |
| 3 | Contenu : cours, chapitres, parcours, compétences ; Markdown + JSON ; parcours d'exemple « Linux — les bases » ; bibliothèque, filtres, recherche ; validation du contenu embarqué | Lecture hors ligne | À faire |
| 4 | Exercices guidés : QCM, compléter, analyser une sortie, rédiger une commande (comparaison textuelle signalée) ; indices, tentatives | Faire un exercice | À faire |
| 5 | XP (domaine pur), niveaux, maîtrise, historique ; dashboard réel ; favoris, « À reprendre », notes ; parcours et déblocage | Livrable phase 1 | À faire |
| 6 | Schéma de scénario v1, import/export `.tssrlab` sécurisé, rapport de validation sans exécution | Import/export + tests sécurité | À faire |
| 7 | Détection Docker/Podman (socket, contexte, rootless, options de sécurité, cgroups), écran des capacités, job CI Docker | Diagnostic moteur | À faire |
| 8 | Premier labo Debian persistant et durci, scripts copiés et exécutés en argv, cycle de vie complet, nettoyage des orphelins | Cycle de vie labo | À faire |
| 9 | Terminal Bash réel : exec TTY, `vt100`, encodage clavier, redimensionnement, onglets, split-view | vim/top dans le labo | À faire |
| 10 | Validation de l'état final (sortie JSON par objectif), XP partiel, historique `.bash_history`, arrêt sur inactivité, premier scénario complet | Scénario de bout en bout | À faire |
| 11 | Builds de release multiplateformes, `docs/LIMITS.md`, tests E2E | Binaires et documentation | À faire |

Ensuite : phases 2 (fin) à 7 de la [feuille de route](ROADMAP.md).

## 4. Risques techniques suivis

- **PTY / émulation** : couverture de `vt100`, encodage des touches,
  disposition AZERTY pour les raccourcis, débit de sortie (limiter le rendu),
  touche de sortie du focus terminal sans conflit avec le shell.
- **Moteurs de conteneurs** : contextes Docker (colima, OrbStack) non lus par
  `bollard`, socket Podman à activer, named pipe Windows, images
  multi-architecture épinglées par digest, limites de stockage rarement
  disponibles.
- **Isolation** : noyau partagé, démon Docker rootful équivalent root, `sudo`
  de l'apprenant dans le conteneur ; la validation est pédagogique, pas
  anti-triche. Pour du contenu hostile, seule une VM est une vraie frontière.
- **VM Windows** : taille des disques, conversion via `qemu-img`, Windows ARM
  seulement sur Apple Silicon, droits Hyper-V, licences.
- **Réseau** : DHCP/VLAN incompatibles avec l'IPAM Docker (backend netns
  nécessaire, Linux uniquement ou dans une VM), différences Podman rootless.
- **Autres** : coffre de secrets absent sous Linux sans session graphique,
  gestion de la clé de signature des releases, volume de contenu pédagogique à
  rédiger.

## 5. Questions ouvertes

- **Référentiel TSSR** : version du REAC à utiliser comme source officielle du
  parcours complet. En attendant, seul un petit contenu d'exemple original est
  rédigé.

## 6. Journal des tranches

### Tranche 0 — Socle (terminée)

- Workspace (`edition 2024`, toolchain épinglée 1.97.0), lints communs
  (`unsafe_code` interdit, `unwrap`/`expect` signalés hors tests), CI GitHub
  Actions Linux/macOS/Windows (fmt, clippy, tests, build release, test de
  fumée) et `cargo-deny`.
- Binaire `tssrlabs` : `--help`, `--version`, `--paths` ; refus explicite hors
  terminal interactif, sans effet de bord.
- Répertoires standards de l'OS (ou `TSSRLABS_HOME`), créés en `0700` sous Unix.
- Journaux locaux quotidiens (14 fichiers max), non bloquants, niveau via
  `TSSRLABS_LOG`, masquage des secrets reconnaissables sur chaque ligne.
- Terminal restauré à la sortie normale, sur erreur, sur panique et sur signal
  (SIGINT/SIGTERM/SIGHUP ; Ctrl+C/fermeture de console sous Windows).
- Écran d'accueil provisoire ; aucune fonctionnalité simulée.

Limites connues :
- Le masquage des journaux ne détecte que les secrets de forme reconnaissable ;
  la règle principale reste de ne jamais journaliser de secret.
- Les fichiers de journaux héritent de l'umask ; leur confidentialité repose
  sur le répertoire parent en `0700` (Unix) et sur les ACL du profil
  utilisateur (Windows).
- Textes encore codés en dur en français : l'i18n arrive en tranche 1
  (fait : voir ci-dessous).

### Tranche 1 — Interface de base (terminée)

- Crate `tssrlabs-domain` : modèle de l'hôte (`HostInfo`, `ColorSupport`),
  sans aucune dépendance.
- Détection de l'hôte (infrastructure) : OS, architecture, émulateur,
  `TERM`, niveau de couleurs (`NO_COLOR`, `COLORTERM`, émulateurs connus,
  `TERM`), correction manuelle par `TSSRLABS_COLORS`.
- Thèmes : sombre (défaut), contraste élevé, daltonisme (Okabe-Ito),
  monochrome. Conversion automatique 24 bits → 256 → 16 couleurs ; sans
  couleur, attributs de texte uniquement et changement de thème masqué.
- Affichage adaptatif : complet (≥ 100×30), compact (≥ 60×16, sans cadre ni
  éléments secondaires), message explicatif en dessous.
- Navigation : écrans Accueil et Diagnostic (seuls écrans réels), aide
  superposée, raccourcis visibles en pied de page.
- Raccourcis : table `Keymap` avec détection des conflits ; liaisons sur le
  caractère produit (AZERTY/QWERTY), lettres insensibles à la casse,
  alternatives non alphabétiques (F1, Tab, flèches).
- i18n : Fluent, français, identifiants vérifiés (aucun manquant, aucun
  orphelin), sans caractères d'isolation bidirectionnelle.
- Vérifié dans un vrai pseudo-terminal : rendu à 100×30, 70×20, 50×12,
  redimensionnement à chaud, `NO_COLOR`, parcours des thèmes.

| Touche | Action |
|---|---|
| Tab / Maj+Tab | Écran suivant / précédent |
| ↑ ↓ (ou k j) | Monter / descendre dans une liste |
| t | Thème suivant (si le terminal gère les couleurs) |
| F1 ou ? | Aide |
| Échap | Fermer la fenêtre superposée |
| q ou Ctrl+C | Quitter |

Limites connues :
- Le thème et les raccourcis ne sont pas encore enregistrés : ils reviennent
  aux valeurs par défaut au redémarrage (préférences en tranche 2).
- Pas encore de réglage manuel de densité : le mode compact dépend
  uniquement de la taille du terminal.
- Souris non activée (la capture empêcherait la sélection de texte du
  terminal) ; la navigation est entièrement au clavier.
- Échap suivi immédiatement d'une lettre peut être lu par le terminal comme
  Alt+lettre (ambiguïté des séquences d'échappement, propre aux terminaux).
- La détection des couleurs est heuristique ; `TSSRLABS_COLORS` permet de la
  corriger. Le diagnostic indique quand la valeur est imposée.
- Lecteurs d'écran : l'interface reste textuelle et chaque état a une
  étiquette, mais la compatibilité dépend du terminal et de l'OS ; non testée
  avec un lecteur d'écran à ce stade.

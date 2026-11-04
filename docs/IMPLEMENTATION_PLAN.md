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
├── tssrlabs-domain/          # règles métier pures (hôte, profils, contenu pédagogique…)
├── tssrlabs-application/     # cas d'utilisation + ports (profils…)
├── tssrlabs-infrastructure/  # adaptateurs : chemins, journaux, SQLite, conteneurs…
├── tssrlabs-tui/             # présentation Ratatui (architecture de type Elm)
└── tssrlabs/                 # binaire : racine de composition, runtime Tokio
```

Sens des dépendances : `tssrlabs` → (`tssrlabs-tui`, `tssrlabs-infrastructure`)
→ `tssrlabs-application` → `tssrlabs-domain`. La présentation ne dépend pas de
l'infrastructure : c'est le binaire qui les assemble.

Une crate n'est créée que lorsqu'elle a un contenu réel : `domain` existe depuis
la tranche 1 (modèle de l'hôte), `application` depuis la tranche 2 (profils).

Les textes de l'interface sont dans `crates/tssrlabs-tui/i18n/<langue>/*.ftl`
(Fluent), embarqués dans le binaire ; chaque identifiant est déclaré dans
`crates/tssrlabs-tui/src/i18n.rs` et vérifié par les tests.

### Principes d'exécution

- Boucle d'interface : état (`App`), `Action`, `update()`, `render()`. Le rendu
  est pur ; la boucle ne fait jamais d'opération longue.
- Les tâches de fond sont lancées via un `TaskTracker` et arrêtées via un
  `CancellationToken` ; leurs résultats reviennent par canal.
- SQLite : un thread dédié possède l'unique connexion ; les requêtes lui
  sont envoyées et leur résultat attendu de façon asynchrone.
- L'état de l'interface ne fait aucune entrée/sortie : il produit des
  « effets » exécutés dans l'ordre par un travailleur séquentiel, dont les
  résultats reviennent comme événements. À la fermeture, les effets déjà
  demandés sont menés à terme avant l'arrêt de la base.
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
| SQLite | `rusqlite` (bundled) + migrations internes (`user_version`) | 2 |
| Dates | `jiff` | 2 |
| Contenu | `serde`, `serde_json`, `pulldown-cmark`, `include_dir`, `unicode-normalization` (recherche sans accents) | 3 |
| Archives `.tssrlab` | `zip`, `sha2`, `semver` | 6 |
| Conteneurs | `bollard` | 7 |
| Émulation terminal | `vt100` (+ `tui-term` ou rendu maison) | 9 |
| Plus tard | `keyring`, `secrecy`, `reqwest` (rustls), `zstd`, `minisign-verify`, `arboard`, `git` système | 3+ |

## 3. Tranches du MVP

| # | Tranche | Démonstration | État |
|---|---|---|---|
| 0 | Socle : workspace, lints, CI 3 OS, licence MIT, restauration du terminal (y compris panique), journaux locaux avec masquage | L'application démarre et quitte proprement | **Terminée** |
| 1 | Boucle d'événements, navigation, thème sombre + contraste élevé, responsive 100×30 / compact / « terminal trop petit », aide, i18n FR (Fluent), écran Diagnostic | Navigation et responsive | **Terminée** |
| 2 | SQLite (thread dédié, migration v1), profils (créer/choisir/renommer/supprimer avec confirmation), préférences | Livrable phase 0 | **Terminée** |
| 3 | Contenu : cours, chapitres, parcours, compétences ; Markdown + JSON ; parcours d'exemple « Linux — les bases » ; bibliothèque, filtres, recherche ; validation du contenu embarqué | Lecture hors ligne | **Terminée** |
| 4 | Exercices guidés : QCM, compléter, analyser une sortie, rédiger une commande (comparaison textuelle signalée) ; indices, tentatives | Faire un exercice | **Terminée** |
| 5a | XP (domaine pur), niveaux, maîtrise, historique, série d'activité ; dashboard réel ; parcours et déblocage | Progression visible | **Terminée** |
| 5b | Favoris, « À reprendre », notes personnelles | Livrable phase 1 | À faire |
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
- **Licence du contenu pédagogique** : les chapitres embarqués sont pour
  l'instant sous la licence du dépôt (MIT). Une licence Creative Commons
  (CC BY-SA 4.0, par exemple) serait plus courante pour du contenu éditorial.
- **Vérification des sources** : les sources citées n'ont pas pu être
  consultées depuis l'environnement de développement (réseau restreint) ;
  elles sont marquées « non vérifiées » jusqu'à vérification humaine.

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

### Tranche 2 — Profils et préférences (terminée) : livrable de la phase 0

- Domaine : `ProfileName` (espaces normalisés, 1 à 32 caractères, aucun
  caractère de contrôle, unicité insensible à la casse), `Preferences`.
- Application : ports `ProfileRepository` et `SafetyBackup`, service des
  profils (créer et activer, activer, renommer, supprimer, préférences).
- Infrastructure SQLite :
  - base `données/tssrlabs.sqlite3`, mode WAL, clés étrangères actives ;
  - thread dédié propriétaire de la connexion, robuste aux paniques de tâche ;
  - migrations internes versionnées par `PRAGMA user_version`, une
    transaction par migration (annulée en cas d'échec) ;
  - copie de sécurité (`VACUUM INTO`) avant toute migration d'une base
    existante ;
  - base créée par une version plus récente : refus explicite, aucune
    modification.
- Interface :
  - premier lancement : écran Profils et fenêtre de création ;
  - lancements suivants : profil actif et son thème restaurés, tableau de
    bord (vide tant qu'aucune activité n'existe) ;
  - écran Profils : liste (état « actif » écrit en toutes lettres, dates
    locales), Entrée activer, n créer, r renommer, s ou Suppr supprimer ;
  - suppression : saisie exacte du nom, copie de sécurité **avant**
    suppression (si elle échoue, rien n'est supprimé), chemin de la copie
    affiché ;
  - le thème est enregistré par profil à chaque changement ;
  - pendant une saisie, les lettres sont du texte (pas des raccourcis) ;
    Ctrl+C quitte toujours ;
  - barre de raccourcis adaptée à la largeur : Aide et Quitter toujours
    visibles.
- Vérifié dans un vrai pseudo-terminal sur deux lancements successifs
  (création, persistance du profil et du thème, doublon refusé, renommage,
  confirmation erronée refusée, suppression avec copie, réactivation).

Pourquoi des migrations internes plutôt que `rusqlite_migration` : la
version disponible impose une version plus ancienne de `rusqlite`, et le
besoin (une liste ordonnée de scripts, `user_version`) tient en quelques
dizaines de lignes testées, avec la copie de sécurité et le refus d'un
schéma plus récent maîtrisés.

Limites connues :
- Les copies de sécurité (`données/copies-de-securite/`) ne sont pas encore
  purgées automatiquement (rétention prévue avec les sauvegardes, phase 5).
  Restauration manuelle : application fermée, remplacer
  `tssrlabs.sqlite3` par la copie choisie.
- L'ouverture de la base est synchrone au démarrage (avant l'interface).
- Deux instances simultanées partagent la base sans corruption (WAL, délai
  d'attente), mais l'une ne voit pas les changements de l'autre avant
  relance.
- Les fichiers de la base héritent de l'umask ; leur confidentialité repose
  sur le répertoire parent en `0700` (Unix) et sur les droits du profil
  utilisateur (Windows).
- Seul le thème est enregistré pour l'instant ; la personnalisation des
  raccourcis et la densité suivront.
- Les journaux mentionnent les identifiants de profil, jamais leurs noms.

### Tranche 3 — Contenu, bibliothèque et lecture (terminée)

- Domaine : `ContentId`, `Domain`, `Difficulty`, `Skill`, `Chapter`,
  `LearningPath`, `Source`, `Catalog` validé (références, cycles de
  prérequis, ordre des parcours, champs vides, caractères de contrôle) et
  recherche insensible à la casse et aux accents avec filtres (domaine,
  niveau, durée maximale).
- Format de contenu v1 documenté dans [`CONTENT_FORMAT.md`](CONTENT_FORMAT.md) :
  `catalog.json` + `chapters/<id>/chapter.json` + `chapter.md`, lecture
  stricte (champ inconnu refusé), contenu embarqué dans le binaire.
- Contenu original : parcours « Linux — les bases » (terminal et shell ;
  arborescence et fichiers ; utilisateurs, groupes et permissions).
- Interface :
  - écran **Parcours** : chapitres dans l'ordre, niveau, durée, total du
    parcours, résumé et prérequis du chapitre sélectionné ;
  - écran **Bibliothèque** : recherche en direct (`/`), filtres domaine
    (`d`), niveau (`f`), durée (`m`), effacement (`x`), compteur de
    résultats ;
  - **lecteur** : métadonnées, objectifs, prérequis, compétences, corps
    Markdown, sources avec statut de vérification, crédits et licence ;
    défilement (↑ ↓, PgPréc/PgSuiv, Espace, Début/Fin), barre de défilement
    et position en pourcentage ; Échap revient à l'écran d'origine ;
  - raccourcis par **contexte** (une touche peut servir sur deux écrans
    différents ; conflits détectés par contexte) ; aide limitée aux
    raccourcis de l'écran courant ; libellés adaptés (« Ouvrir »,
    « Retour ») ;
  - en-tête : le nom du profil reste visible, le thème est omis si la place
    manque.
- Sécurité : tout texte affiché (Markdown, métadonnées) est débarrassé des
  caractères de contrôle ; le HTML brut n'est jamais interprété.
- Vérifié dans un vrai pseudo-terminal : tableau de bord, parcours,
  recherche « droits », lecture et défilement, retour avec recherche
  conservée.

Limites connues :
- La progression (chapitre lu, déblocage selon les prérequis) arrive avec
  la tranche 5 ; les prérequis sont affichés mais pas imposés.
- Dans le lecteur, la suite d'un élément de liste replié n'est pas
  réindentée sous le texte de la puce.
- Les tableaux Markdown ne sont pas mis en forme.
- En mode compact, l'aide peut dépasser la hauteur disponible ; elle est
  alors coupée.
- La recherche porte sur titre, résumé, mots-clés, objectifs et
  compétences, pas sur le corps des chapitres.

### Tranche 4 — Exercices guidés (terminée)

- Domaine : `Exercise` (QCM simple ou multiple, commande à compléter,
  analyse de sortie, commande à rédiger), vérification des réponses,
  solution, `Outcome` (réussite, échec, solution affichée) et
  `ExerciseProgress` (tentatives, réussite, solution vue, moins d'indices
  utilisés lors d'une réussite — base des bonus de la tranche 5).
- Contenu : `exercises.json` par chapitre (format documenté), validation
  complète ; 12 exercices originaux pour le parcours « Linux — les bases ».
- Stockage : migration v2 `exercise_attempts` (profil, exercice, issue,
  indices, date ; la réponse saisie n'est pas conservée), suppression en
  cascade avec le profil. Vérifié en conditions réelles : une base créée
  par la tranche 2 est migrée au lancement, avec copie de sécurité
  préalable et données conservées.
- Application : port `AttemptRepository`, `ExerciseService` (enregistrer,
  progression).
- Interface :
  - depuis la lecture d'un chapitre, `e` ouvre ses exercices ;
  - QCM : flèches et Espace, ou chiffres, pour cocher ; état écrit `[x]`,
    bonnes réponses marquées `✓` une fois l'exercice terminé ;
  - réponses textuelles : les lettres sont du texte (pas de raccourci) ;
    mention permanente « la commande n'est pas exécutée » ;
  - Entrée valide (puis passe à l'exercice suivant après réussite), F2
    indice suivant, F3 solution (comptée comme tentative), PgSuiv/PgPréc
    exercice suivant/précédent, Échap retour au chapitre ;
  - réponse vide : message, rien n'est compté ;
  - sans profil actif : exercices utilisables, tentatives non enregistrées
    et signalées comme telles ;
  - tableau de bord, lecture et parcours affichent les exercices réussis.
- Vérifié dans un vrai pseudo-terminal sur une base migrée : QCM simple et
  multiple (réponse partielle refusée), commande complétée, commande
  rédigée avec invite et espaces superflus, compteurs à jour.

Limites connues :
- Vérification textuelle uniquement : l'ordre des options et les
  écritures équivalentes non listées par l'auteur sont refusés ; la
  vérification par exécution réelle viendra avec les laboratoires.
- Pas encore d'XP ni de niveaux (tranche 5) ; la progression est
  enregistrée pour être exploitée ensuite.
- Sur un petit terminal, la fin d'un exercice long (explication, indices)
  peut être coupée ; pas de défilement dans l'écran d'exercice.

### Tranche 5a — Progression (terminée)

- Domaine `progression` : tout est **calculé** à partir de la liste
  ordonnée des tentatives (aucune donnée dupliquée, règles modifiables en
  un seul endroit) :
  - XP : 10 / 20 / 30 selon le niveau du chapitre, accordé une seule fois
    (première réussite), +50 % sans indice, +25 % du premier coup, rien si
    la solution a été affichée avant la réussite, aucun XP retiré ;
  - niveaux : seuils 0, 100, 300, 600, 1000… (le niveau n demande n × 100 XP
    de plus que le précédent) ;
  - maîtrise par compétence, indépendante du niveau global : part des
    exercices réussis (sans solution vue avant) parmi ceux des chapitres
    qui travaillent la compétence ; non commencée, découverte (< 40 %),
    en progrès (< 80 %), maîtrisée ;
  - chapitres : verrouillé tant qu'un prérequis n'est pas terminé,
    disponible, terminé (tous ses exercices réussis ou solution vue) ;
    chapitre recommandé = premier chapitre disponible des parcours ;
  - série d'activité : jours consécutifs avec au moins une tentative,
    dans le fuseau de l'apprenant.
- Stockage : le dépôt renvoie les tentatives datées et ordonnées (pas de
  nouvelle migration).
- Interface :
  - **tableau de bord réel** : niveau et barre d'XP, exercices réussis,
    série, chapitre recommandé (Entrée l'ouvre), compétences, activité
    récente avec l'XP gagné ;
  - nouvel onglet **Progression** : bilan, maîtrise détaillée, rappel des
    règles d'XP, historique complet ;
  - **parcours** : état de chaque chapitre en texte ; un chapitre verrouillé
    ne s'ouvre pas depuis un parcours (message explicatif), la Bibliothèque
    reste en accès libre ;
  - exercice : XP obtenu affiché ;
  - messages repliés sur trois lignes au plus au lieu d'être tronqués ;
    en mode compact, l'en-tête n'affiche que l'écran courant et sa position.
- Vérifié dans un vrai pseudo-terminal sur une base réelle : 66 XP
  attendus et obtenus pour les 5 tentatives enregistrées, états des
  chapitres, verrouillage, recommandation, affichage compact.

Limites connues :
- Un chapitre sans exercice est considéré comme terminé (la lecture n'est
  pas encore suivie) : il ne bloque pas un parcours.
- Pas de succès/trophées ni d'objectifs quotidiens/hebdomadaires pour
  l'instant.
- En mode compact, l'activité récente du tableau de bord peut être coupée ;
  elle reste consultable dans l'onglet Progression.

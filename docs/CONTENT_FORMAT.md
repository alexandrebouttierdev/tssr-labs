# Format du contenu pédagogique (version 1)

Ce document décrit le format des cours embarqués dans TSSR Labs (répertoire
`content/`). Les scénarios de laboratoire `.tssrlab` ont leur propre format
(voir [`SCENARIO_FORMAT.md`](SCENARIO_FORMAT.md)).

## Organisation

```text
content/
├── catalog.json                 compétences et parcours
└── chapters/<identifiant>/
    ├── chapter.json             métadonnées du chapitre
    ├── chapter.md               corps du chapitre en Markdown
    └── exercises.json           exercices guidés (facultatif)
```

Le nom du répertoire d'un chapitre doit être égal à son identifiant.

## Identifiants

Minuscules ASCII, chiffres et tirets simples, 64 caractères au plus :
`linux-arborescence-fichiers`. Un identifiant est unique dans tout le
catalogue (compétences, chapitres et parcours confondus).

## `catalog.json`

```json
{
  "format_version": 1,
  "skills": [
    { "id": "linux-shell", "name": "Utiliser le shell Bash", "domain": "linux" }
  ],
  "paths": [
    {
      "id": "linux-bases",
      "title": "Linux — les bases",
      "summary": "…",
      "chapters": ["linux-terminal-premiers-pas", "linux-arborescence-fichiers"]
    }
  ]
}
```

Domaines : `linux`, `windows`, `reseau`, `systeme`, `securite`,
`virtualisation`, `depannage`, `automatisation`.

## `chapter.json`

| Champ | Type | Règle |
|---|---|---|
| `id` | texte | identifiant, égal au nom du répertoire |
| `version` | texte | version du contenu (ex. `1.0.0`) |
| `title`, `summary` | texte | non vides |
| `domain` | texte | un des domaines ci-dessus |
| `difficulty` | texte | `debutant`, `intermediaire` ou `avance` |
| `estimated_minutes` | entier | supérieur à 0 |
| `tags` | liste de textes | mots-clés pour la recherche |
| `skills` | liste d'identifiants | compétences déclarées dans `catalog.json` |
| `objectives` | liste de textes | objectifs pédagogiques |
| `prerequisites` | liste d'identifiants | chapitres existants, sans cycle |
| `sources` | liste | `title`, `url`, `verified_on` (`AAAA-MM-JJ` ou `null`) |
| `authors` | liste de textes | auteurs |
| `license` | texte | identifiant SPDX |

Tout champ inconnu est refusé.

## `chapter.md`

- Commence par un titre de niveau 1 (`# …`), omis à l'affichage car le titre
  du chapitre est déjà présenté.
- Markdown pris en charge : titres, paragraphes, listes (imbriquées),
  blocs de code, code en ligne, emphase, gras, citations, liens, séparateurs.
- Le HTML brut n'est pas interprété (affiché tel quel). Les tableaux
  Markdown ne sont pas mis en forme : préférer des listes.

## `exercises.json`

Liste d'exercices guidés du chapitre. Champs communs : `id` (identifiant
unique dans tout le catalogue), `type`, `title`, `prompt` (Markdown),
`hints` (liste, du plus général au plus précis, Markdown), `explanation`
(Markdown, affichée après réussite ou affichage de la solution).

| `type` | Champs propres | Vérification |
|---|---|---|
| `qcm` | `choices` (≥ 2, sans doublon), `correct` (indices à partir de 0, triés) | ensemble exact des propositions cochées |
| `completer` | `template` (contient exactement un `___`), `answers` | valeur saisie pour l'emplacement |
| `analyser` | `output` (sortie de terminal affichée), `answers` | texte court, **casse ignorée** |
| `rediger` | `answers` | commande complète, **casse respectée** |

Un champ propre à un autre type est refusé.

Les types `completer`, `analyser` et `rediger` sont vérifiés **par
comparaison de texte** : espaces superflus ignorés, invite `$ ` ou `# `
recopiée tolérée ; la commande n'est jamais exécutée et l'interface
l'indique. Lister dans `answers` toutes les formes équivalentes acceptées
(`mkdir -p x`, `mkdir --parents x`) : l'ordre des options n'est pas
normalisé.

```json
[
  {
    "id": "ex-mkdir-parents",
    "type": "rediger",
    "title": "Créer une arborescence",
    "prompt": "Créez `projets/site/images` en une seule commande.",
    "answers": ["mkdir -p projets/site/images", "mkdir --parents projets/site/images"],
    "hints": ["Une option permet de créer aussi les parents."],
    "explanation": "`mkdir -p` crée les répertoires parents manquants."
  }
]
```

## Validation

Au chargement (et dans les tests) :

- erreurs bloquantes : identifiant invalide ou en double, référence
  inconnue (compétence, prérequis, chapitre de parcours), cycle de
  prérequis, chapitre placé avant son prérequis dans un parcours, champ
  obligatoire vide, durée nulle, exercice incohérent avec son type,
  **caractère de contrôle** dans un texte
  (protection contre l'injection de séquences d'échappement dans le
  terminal) ;
- avertissement : source sans date de vérification (`verified_on: null`),
  affichée « non vérifiée » dans le lecteur.

## Règles éditoriales

- Contenus originaux, en français.
- Chaque chapitre cite des sources officielles. `verified_on` n'est
  renseigné qu'après vérification humaine de la source à cette date.
- Ne pas présenter un contenu comme conforme au référentiel officiel TSSR
  tant que la correspondance n'a pas été établie.

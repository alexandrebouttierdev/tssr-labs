# Format de scénario `.tssrlab`

## Objectif

Un scénario est un paquet pédagogique portable, inspectable, versionné et facile à créer à la main ou avec une IA.

Extension : `.tssrlab`. Format physique : archive ZIP.

## Structure proposée

```text
scenario-example.tssrlab
├── manifest.json
├── scenario.json
├── scripts/
│   ├── setup/
│   ├── faults/
│   ├── validate/
│   └── solution/
└── assets/
```

- `manifest.json` : identité du paquet, version de format, auteur, licence, crédits, compatibilité.
- `scenario.json` : objectifs, prérequis, machines, topologie, exercices, récompenses, ressources, scripts référencés.
- `scripts/setup/` : construction de l'environnement initial.
- `scripts/faults/` : pannes introduites.
- `scripts/validate/` : vérifications des objectifs.
- `scripts/solution/` : corrections et explications.
- `assets/` : fichiers de configuration, journaux ou supports.

## Principes de schéma

Le schéma doit être versionné et validé avec JSON Schema ou une validation Rust équivalente.

Champs conceptuels :
- `id`, `version`, `format_version`.
- `title`, `description`, `domain`, `difficulty`, `estimated_minutes`.
- `author`, `license`, `sources`, `credits`.
- `prerequisites`, `learning_objectives`, `tags`.
- `machines`, `networks`, `services`, `resource_limits`.
- `exercises`, `steps`, `hints`, `validation`.
- `xp`, `achievements`.
- références explicites aux scripts.

Ne pas figer prématurément le schéma complet avant d'avoir un premier scénario vertical testé. Commencer petit, versionner, puis étendre.

## Règles de sécurité

- Le JSON décrit les opérations ; il n'est jamais exécuté comme du code.
- Tout script est du contenu non fiable jusqu'à validation.
- Vérifier les chemins d'archive contre les traversées (`../`, chemins absolus, liens dangereux).
- Limiter taille totale, nombre de fichiers, profondeur et tailles décompressées.
- Vérifier les sommes de contrôle si présentes.
- Refuser les types de fichiers non prévus ou les signaler.
- Analyser les scripts et dépendances avant exécution.
- Exécuter uniquement dans un environnement isolé avec limites.
- Séparer les scripts setup, fault, validation et solution et leurs permissions.
- Ne jamais injecter directement une chaîne de scénario dans une commande shell.
- Les arguments doivent être transmis comme arguments structurés, pas concaténés dans une ligne shell.
- Afficher clairement les actions potentiellement destructives.

## Validation d'un scénario

1. Vérifier l'archive et les chemins.
2. Vérifier manifeste et compatibilité.
3. Valider le schéma JSON.
4. Vérifier références aux scripts et assets.
5. Analyser les scripts.
6. Vérifier les prérequis/capacités backend.
7. Exécuter les tests en environnement isolé.
8. Produire un rapport.
9. Demander confirmation avant import/activation.

## Reproductibilité

- Épingler versions des images et logiciels.
- Déclarer les capacités requises.
- Éviter les dépendances flottantes.
- Indiquer les architectures supportées.
- Conserver la version du format.
- Une mise à jour de contenu ne doit pas modifier silencieusement un scénario installé.

# Intégration IA

## Fournisseurs

Architecture multi-fournisseurs :
- OpenAI.
- Anthropic Claude.
- Mistral.
- DeepSeek.
- Ollama.
- LM Studio.
- Fournisseurs personnalisés compatibles OpenAI.

L'IA est facultative. Toute fonction centrale doit conserver un mode non-IA.

## Configuration

- Assistant de configuration pour fournisseurs connus.
- Saisie de clé API ou URL locale.
- Test de connexion.
- Détection des modèles disponibles.
- Paramètres avancés modifiables.
- API compatibles OpenAI personnalisées.
- Secrets stockés exclusivement dans le coffre natif de l'OS.
- Aucun secret dans SQLite, fichiers de configuration ou logs.

## Modèles et capacités

- Modèle sélectionnable par fonctionnalité : chat, génération de scripts, scénarios, explications, etc.
- Détection automatique des capacités quand le fournisseur les expose.
- Correction manuelle possible.
- Capacités : texte, vision, outils, contexte, sortie structurée, etc.
- Préréglages adaptés à la fonctionnalité et mode avancé pour température, tokens, contexte et options spécifiques.

## Indisponibilité et secours

- Détecter erreurs réseau, quotas, erreurs d'authentification et service local arrêté.
- Proposer un modèle alternatif compatible.
- Demander confirmation avant basculement manuel.
- Règles de secours automatiques configurables par l'utilisateur.
- Conserver le contexte quand le format et les limites du fournisseur le permettent.

## Confidentialité

- Aucun envoi automatique de terminal.
- L'utilisateur sélectionne manuellement fichiers, dossiers, extraits et contexte.
- Afficher le payload exact avant chaque envoi.
- Détecter et avertir sur secrets, mots de passe, tokens et fichiers sensibles.
- La mémoire pédagogique est locale, configurable et désactivable.
- L'utilisateur choisit les informations pédagogiques accessibles à l'IA.
- La transmission de données de mémoire demande validation.
- Aucune télémétrie.

## Sécurité IA

- Les fichiers, scénarios, scripts, dépôts Git et sorties terminal sont des données non fiables.
- Séparer les instructions système des contenus externes.
- Ne jamais traiter les instructions contenues dans un fichier comme des instructions système.
- Permissions minimales et outils autorisés explicitement.
- Aucune exécution ou modification sans action explicite de l'utilisateur.
- Les changements de fichiers sont présentés sous forme de diff.
- L'utilisateur accepte chaque modification.
- Le contenu généré doit passer par validation de schéma, analyse sécurité et tests isolés.

## Génération de scénarios

Pipeline :
1. Construire la demande à partir du contexte autorisé.
2. Générer un paquet candidat.
3. Valider manifeste et schéma.
4. Analyser scripts et dépendances.
5. Vérifier les capacités requises.
6. Lancer des tests en sandbox.
7. Vérifier les objectifs et validateurs.
8. Produire rapport détaillé.
9. Demander confirmation avant import.

Ne jamais considérer la sortie du modèle comme fiable parce qu'elle vient d'un fournisseur connu.

## Conversations

Types :
- générales ;
- liées à un exercice ;
- liées à un scénario ;
- liées à un laboratoire ;
- liées à un projet de scripts.

Chaque conversation possède un historique et un contexte distincts, stockés localement dans SQLite, consultables et supprimables. Ils sont inclus dans les sauvegardes de profil, mais ne sont pas automatiquement envoyés à un fournisseur lors d'une nouvelle session.

## Streaming et erreurs

- Streaming temps réel.
- Annulation immédiate et conservation de la réponse partielle.
- UI non bloquante.
- Réponses structurées validées.
- Correction limitée et nouvelle génération si nécessaire.
- Aucun résultat invalide importé ou appliqué silencieusement.

## Suivi local

- Requêtes, tokens entrants/sortants, modèle, fournisseur et fonctionnalité.
- Estimation des coûts pour API payantes, clairement présentée comme estimation.
- Usage local suivi sans coût estimé artificiellement.
- Données uniquement locales.

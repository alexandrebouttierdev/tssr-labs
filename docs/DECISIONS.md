# Registre des décisions produit et techniques

Ce document rassemble les choix validés pendant le cadrage. En cas de conflit avec une proposition ultérieure, demander confirmation avant de modifier une décision structurante.

## Produit et expérience

- Application terminal native en Rust + Ratatui.
- Interface moderne, sombre par défaut, professionnelle, sobre, panneaux lisibles, bordures discrètes et couleurs cohérentes.
- Navigation clavier prioritaire ; souris facultative.
- Tableau de bord central comme écran d'accueil.
- Interface adaptative à la taille du terminal ; référence pleine interface : 100 colonnes × 30 lignes. En dessous : mode compact, avertissement si nécessaire et aucune sortie hors écran.
- Compatibilité recherchée : Ghostty, Kitty, Alacritty, iTerm2, WezTerm, Windows Terminal, GNOME Terminal et terminaux ANSI compatibles.
- Thèmes prédéfinis, palettes daltonisme/contraste élevé, taille de texte ajustable, animations désactivables.
- Compatibilité lecteur d'écran dans la mesure permise par le terminal et l'OS.
- Raccourcis personnalisables avec détection des conflits et prise en compte AZERTY/QWERTY.
- Presse-papiers natif lorsque disponible, avec solution de repli.
- Langue initiale française, internationalisation prévue.

## Architecture et exécution

- Monolithe modulaire Rust, un exécutable.
- Couches : Domain, Application, Infrastructure, Presentation.
- Dépendances orientées vers le domaine ; celui-ci ne dépend ni de Ratatui, ni de SQLite, ni de Docker, ni d'un fournisseur IA.
- Tokio pour l'asynchrone ; tâches structurées, annulables et sans blocage de l'UI.
- Threads dédiés pour opérations bloquantes.
- SQLite pour les données applicatives locales, avec migrations versionnées.
- Détection automatique Docker/Podman, choix manuel possible.
- Abstraction commune Docker/Podman et adaptateurs séparés.
- Dépendances facultatives et détectées automatiquement ; les cours théoriques restent accessibles sans moteur de conteneurs.
- Binaries autonomes Linux/macOS/Windows.
- Tests automatisés étendus et CI sur les trois OS.

## Laboratoires

- Environnements réels, pas de simulation de commandes.
- Linux dans des conteneurs Docker/Podman.
- Bash et sh comme shells Linux principaux.
- Distributions initiales : Debian, Ubuntu Server, Rocky Linux.
- Images téléchargées à la demande, mises en cache et réutilisables.
- Versions d'images et logiciels épinglées pour la reproductibilité.
- Laboratoires Linux persistants : arrêt/reprise, modifications conservées, réinitialisation manuelle.
- PTY réel, ANSI, édition de ligne, historique, redimensionnement, signaux, Ctrl+C/Ctrl+D et applications interactives.
- Plusieurs terminaux simultanés par laboratoire, sous forme d'onglets, avec historique séparé.
- À la fermeture de l'application, fermer les sessions PTY et arrêter proprement les conteneurs ; conserver le système de fichiers du laboratoire. Les processus de l'ancienne session ne sont pas garantis.
- Arrêt automatique après inactivité configurable, avertissement préalable et possibilité de désactivation.
- Historique de commandes persistant par laboratoire, consultable et effaçable.
- Limites CPU/RAM/stockage configurables et surveillance selon les capacités réellement offertes par le backend.
- Réseau isolé par laboratoire ; communication entre machines du même labo, isolation entre labos, Internet/LAN désactivés par défaut.
- Topologies multi-machines et visualisation directement dans Ratatui.
- Adressage initial défini par scénario, modifiable par l'apprenant.
- Backend réseau interchangeable : réseau Docker/Podman au départ, backend namespaces/bridge/veth Linux si nécessaire.
- Catalogue de services initial : SSH, DNS, DHCP, Apache/Nginx, Samba, NFS, PostgreSQL/MySQL, UFW/nftables, systemd, Docker/Podman, Ansible, OpenLDAP.
- Scripts d'installation, de panne, de validation et de correction distincts.
- Environnements renforcés : non-root par défaut, capabilities réduites, seccomp/AppArmor/SELinux selon disponibilité, pas de montage du socket Docker hôte, volumes strictement contrôlés, limites ressources/temps et réseau désactivé par défaut.
- Docker/Podman ne constituent pas à eux seuls une frontière de sécurité absolue ; signaler les limites.

## Windows

- PowerShell 7 dans un environnement isolé quand possible.
- Exercices avancés Windows dans de vraies VM Windows.
- Détection d'hyperviseurs compatibles via abstraction : KVM/QEMU, Hyper-V, UTM/VMware selon API et OS.
- Import de VM existantes uniquement ; pas de téléchargement automatique d'OS Windows.
- Formats à prendre en charge si conversion possible : OVA/OVF, QCOW2, VMDK, VDI, VHD/VHDX, RAW.
- VMs persistantes, arrêt/reprise et réinitialisation manuelle.
- Toujours afficher les limites de compatibilité, les prérequis et les questions de licence.
- Ne jamais promettre une fonctionnalité non supportée par l'hôte.

## Snapshots et sauvegardes

- Snapshots automatiques avant chaque exercice et manuels à tout moment lorsque le backend le permet.
- Pour les conteneurs, un snapshot peut nécessiter sauvegarde de volumes/configuration et reconstruction ; ne pas prétendre capturer la mémoire/processus.
- Rétention intelligente configurable, snapshots épinglés protégés et avertissement avant nettoyage.
- Racine de stockage TSSR Labs configurable, organisée par profil/laboratoire/machine/snapshot.
- Ne pas déplacer arbitrairement les répertoires internes Docker/Podman/hyperviseur.
- Sauvegarde complète : profils, configurations, volumes Linux, disques VM, snapshots et métadonnées.
- Sauvegardes automatiques à fréquence configurable et manuelles, désactivables.
- Destinations multiples : disque local, USB, NAS/SMB/NFS.
- Archives compressées Zstandard, portables, vérifiées en intégrité et restaurables de façon complète ou sélective.
- Pas de chiffrement intégré des archives ; dépendance aux permissions et au chiffrement du disque/fichier hôte.
- Avant restauration : analyse d'intégrité, compatibilité, taille, aperçu des données remplacées, confirmation explicite.
- Sauvegarder l'état actuel avant remplacement et proposer un rollback en cas d'échec.
- Limites de rétention configurables, sauvegardes épinglées et avertissement avant suppression.

## Scénarios et partage

- Scénarios JSON + scripts personnalisés.
- Archive `.tssrlab` ZIP contenant manifeste, scénario, scripts et assets.
- Le JSON décrit ; les scripts exécutent. Le JSON ne doit jamais être exécuté comme du code.
- Éditeur intégré + édition directe des fichiers.
- Validation structurelle, sécurité et comportement avant publication/import.
- Import/export par fichiers et gestion de dépôts Git externes.
- Pas de catalogue communautaire intégré, pas de compte communautaire, pas de notation/commentaires.
- Dépôts Git multiples, publics/privés, clone/pull/commit/push/branches/diff et résolution assistée des conflits.
- Authentification Git SSH (clés existantes) et HTTPS (tokens).
- Secrets dans le coffre natif OS, jamais en clair dans SQLite/config/logs.
- Détection des scénarios en doublon entre dépôts : montrer origines, versions et différences ; demander le choix.
- Publication Git contrôlée : aperçu des fichiers/diffs, sélection, message de commit, commit local puis confirmation explicite avant push.
- Pull avec comparaison, fusion assistée, sauvegarde et confirmation avant application.
- Licence choisie par le créateur pour chaque scénario ; métadonnées auteur, licence, sources, crédits et version.

## Progression et contenu

- Parcours guidés + exploration libre.
- Contenu théorique + exercices guidés + labs + évaluations.
- Simulateur d'examen TSSR complet, chronométré, avec situations professionnelles, dépannage, score et rapport de compétences ; indiquer qu'il ne s'agit pas d'un examen officiel.
- Objectifs quotidiens/hebdomadaires facultatifs.
- Notifications internes uniquement, sans notification système, son ou communication en arrière-plan.
- Pas de télémétrie, ni rapport de crash automatique.
- Logs locaux détaillés, export manuel, secrets masqués.
- Stats détaillées consultables dans l'app, sans export dédié.

## Éditeur de scripts

- Éditeur avancé dans Ratatui : arborescence, multi-fichiers, coloration syntaxique Bash/PowerShell/JSON, numéros de ligne, indentation, recherche/remplacement, complétion de base.
- Analyse automatique : ShellCheck et PSScriptAnalyzer si disponibles ; erreurs, avertissements, suggestions avec emplacement.
- Exécution uniquement dans un laboratoire isolé, sortie live, interruption et traçage.
- Débogage selon les capacités natives : breakpoints, pas à pas, variables, pile d'appels lorsque réellement supportés.
- Projets locaux avec templates, recherche globale et dépendances.
- Dépendances installées dans le labo seulement, jamais sur l'hôte.
- Git intégré au workspace.
- Assistant IA dans un panneau latéral ; modifications proposées sous forme de diff et acceptées explicitement.

## IA

- Multi-fournisseurs : OpenAI, Anthropic Claude, Mistral, DeepSeek, Ollama, LM Studio et API personnalisées compatibles OpenAI.
- IA facultative ; le produit central fonctionne sans IA et hors ligne.
- Configuration assistée + avancée, test de connexion et découverte de modèles.
- Capacités détectées automatiquement si possible, corrigibles manuellement.
- Modèle configurable par fonctionnalité : chat, scripts, scénarios, explications, etc.
- Préréglages par fonctionnalité + paramètres avancés (température, tokens, contexte, options fournisseur).
- Indisponibilité : détecter, proposer un modèle compatible, demander confirmation ; règles de secours automatiques configurables.
- Suivi local : requêtes, tokens, consommation par modèle/fonctionnalité et estimation de coûts pour API payantes.
- Génération de scénarios : validation schéma, analyse sécurité, tests isolés et rapport ; import après confirmation.
- Prompt injection : séparation instructions/données, contenus externes non fiables, limites de permissions, contrôle des fichiers.
- Réponses invalides : validation, correction limitée ou nouvelle génération ; aucune application silencieuse.
- Conversations multiples : générales, exercices, scénarios, labs, projets ; contexte et historique distincts.
- Mémoire pédagogique configurable, locale ; choix des informations accessibles ; transmission après validation.
- L'utilisateur sélectionne manuellement les fichiers et contextes transmis ; affiche le payload exact avant chaque envoi.
- Détecter et avertir sur secrets/API keys/mots de passe/fichiers sensibles.
- Aucune transmission automatique du terminal.
- Pas de modification/exécution par l'IA sans action explicite.
- Historique des conversations stocké localement dans SQLite, par projet/conversation, consultable/supprimable et inclus dans sauvegardes.
- Réponses en streaming, génération interruptible, UI non bloquante.

## Git, mises à jour et confidentialité

- Git intégré au gestionnaire de dépôts et à l'éditeur.
- Mises à jour via GitHub Releases : stable/beta, notification changelog, téléchargement/installation après approbation.
- Vérification signature cryptographique et SHA-256 avant installation.
- Conservation de la version précédente et rollback, sauvegarde avant migration risquée.
- Mises à jour des contenus/scénarios après notification et confirmation, compatibilité et rollback si possible.
- Aucun compte distant, aucune télémétrie, aucun rapport automatique.
- Logs locaux avec masquage des secrets.
- Coffre natif : macOS Keychain, Windows Credential Manager, Linux Secret Service/équivalent. Si indisponible, avertir et ne pas stocker les secrets non protégés.

## Compléments validés au démarrage (2026-11-03)

Décisions prises lors de l'audit initial, en complément des points ci-dessus.

- Dépôt : branche principale `main` ; documents de cadrage versionnés dans le dépôt.
- Utilisateur dans les labos : côté hôte, conteneur non privilégié (`cap-drop ALL`, `no-new-privileges`, user namespaces/rootless si disponibles). Dans la machine, l'apprenant est un utilisateur non-root disposant de `sudo`, car les exercices TSSR exigent des actions d'administration. Les capabilities supplémentaires (ex. `NET_ADMIN` pour un pare-feu) proviennent d'une liste autorisée, sont déclarées par le scénario et affichées avant le démarrage.
- Images de labo : construites localement à partir de Dockerfiles embarqués et d'images de base épinglées par digest ; le réseau n'est ouvert que pendant ce build contrôlé par l'application, jamais dans le labo. Publication d'images préconstruites envisagée plus tard.
- Format des cours : texte en Markdown et métadonnées en JSON (cohérent avec les scénarios).
- Exercice « rédiger une commande » hors labo : comparaison textuelle normalisée, explicitement présentée comme telle (aucune exécution simulée).
- Premier labo sans systemd (support inégal selon Docker/Podman) ; systemd traité dans une tranche ultérieure.
- Taille de texte : non contrôlable par une application terminal ; remplacée par des modes de densité, la taille de police relevant de l'émulateur.
- Mises à jour : vérification manuelle par défaut, vérification au lancement activable ; aucune communication réseau implicite.
- Git : utilisation du client `git` du système (détecté comme dépendance facultative), appelé avec des arguments structurés, sans shell.
- Rust : édition 2024, toolchain épinglée dans `rust-toolchain.toml`.
- Migrations SQLite : exécuteur interne fondé sur `PRAGMA user_version` (une transaction par migration, copie de sécurité préalable, refus d'un schéma plus récent) plutôt que `rusqlite_migration`, dont la version disponible impose une version plus ancienne de `rusqlite`.
- Suppression d'un profil : confirmation par saisie exacte du nom et copie de sécurité de la base créée avant la suppression ; si la copie échoue, rien n'est supprimé.
- Contenu pédagogique embarqué : format v1 (`catalog.json`, `chapters/<id>/chapter.json` + `chapter.md`), lecture stricte, validation complète au chargement et en CI ; caractères de contrôle refusés dans tout texte de contenu.
- Sources de contenu : `verified_on` n'est renseigné qu'après vérification humaine ; sinon la source est affichée « non vérifiée ».
- Raccourcis clavier par contexte d'écran : conflits détectés seulement entre commandes pouvant être actives en même temps.

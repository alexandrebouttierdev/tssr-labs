# Laboratoires, PTY et isolation

## Architecture des laboratoires

Un laboratoire peut contenir une ou plusieurs machines et réseaux privés. Le scénario définit la topologie et l'état initial.

### Linux

- Conteneurs Docker/Podman.
- Distributions : Debian, Ubuntu Server, Rocky Linux.
- Bash et sh.
- Machines persistantes, arrêt/reprise.
- Images épinglées et téléchargées à la demande.
- Réseau privé par labo.
- Communication inter-machines dans le même labo.
- Pas d'accès LAN/Internet par défaut.

### Windows

- PowerShell 7 isolé lorsque possible.
- Pour les exercices dépendant de Windows Server, Active Directory, registre, GPO ou services natifs : VM Windows importée par l'utilisateur.
- Détecter les hyperviseurs et capacités réelles.
- Ne pas télécharger automatiquement Windows.
- Respecter les licences.

## PTY

Le terminal doit être un vrai pseudo-terminal :
- flux stdin/stdout/stderr ;
- séquences ANSI ;
- redimensionnement ;
- signaux et interruption ;
- édition et historique du shell ;
- programmes interactifs ;
- plusieurs sessions/onglets ;
- séparation machine et shell.

La couche UI ne doit pas réimplémenter artificiellement un shell.

## Cycle de vie

- Création depuis scénario.
- Initialisation.
- Introduction des pannes.
- Démarrage des sessions PTY.
- Validation.
- Arrêt et reprise.
- Réinitialisation explicite.
- Nettoyage explicite.

À la fermeture de l'application, fermer les sessions et arrêter proprement les conteneurs ; préserver les volumes/état du labo. Les processus de l'ancienne session peuvent être perdus.

## Réseau

Commencer par les réseaux natifs Docker/Podman. Prévoir un port backend permettant un moteur Linux plus avancé basé sur namespaces, bridge et veth.

Les scénarios déclarent leurs besoins réseau ; ils ne doivent pas dépendre directement d'une commande Docker spécifique.

## Sécurité

- Non-root par défaut.
- Retirer les Linux capabilities non nécessaires.
- Seccomp et profils AppArmor/SELinux selon disponibilité.
- Pas de socket Docker/Podman hôte monté dans les conteneurs d'exercice.
- Volumes explicitement déclarés et limités.
- Pas de montage arbitraire du système hôte.
- Réseau désactivé par défaut.
- Limites CPU, mémoire, stockage, temps et processus lorsque supportées.
- Scripts d'initialisation et de validation avec permissions distinctes.
- Détecter et signaler les limites de l'isolation selon l'OS et le moteur.

Docker/Podman ne doivent pas être présentés comme une isolation absolue contre un scénario hostile. Pour des contenus réellement non fiables, documenter les risques et envisager des frontières VM plus fortes.

## Ressources et arrêt

- Limites configurables par labo/machine.
- Surveillance en temps réel si backend le permet.
- Avertissements de consommation.
- Auto-stop configurable avec délai et avertissement.
- Inactivité calculée selon activité réelle du labo, pas uniquement saisie clavier.
- Nettoyage des ressources orphelines et diagnostic des échecs.

## Snapshots

- Snapshot avant exercice et manuel si supporté.
- Conteneurs : sauvegarde cohérente des volumes/configurations et reconstruction ; ne pas promettre un snapshot mémoire/processus.
- VM : snapshot natif si hyperviseur compatible.
- Rétention configurable et éléments épinglés.

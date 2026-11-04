# Arborescence et manipulation de fichiers

## Une seule arborescence

Sous Linux, tous les fichiers sont rangés dans une **arborescence unique** qui part de la racine `/`. Il n'y a pas de lettres de lecteur comme sous Windows : un disque supplémentaire, une clé USB ou un partage réseau sont **montés** dans un répertoire de cette arborescence.

## Les répertoires essentiels

La norme FHS (*Filesystem Hierarchy Standard*) décrit le rôle des principaux répertoires :

- `/etc` : fichiers de configuration du système et des services ;
- `/home` : répertoires personnels des utilisateurs (`/home/alice`) ;
- `/root` : répertoire personnel de l'administrateur root ;
- `/var` : données variables ; en particulier `/var/log` pour les journaux ;
- `/tmp` : fichiers temporaires, souvent vidés au redémarrage ;
- `/usr` : programmes et bibliothèques installés (`/usr/bin`, `/usr/lib`, `/usr/share`) ;
- `/bin` et `/sbin` : commandes de base ; sur les distributions récentes, ce sont souvent des liens vers `/usr/bin` et `/usr/sbin` ;
- `/dev` : fichiers représentant les périphériques ;
- `/proc` et `/sys` : informations fournies par le noyau, sans fichier réel sur le disque ;
- `/mnt` et `/media` : points de montage temporaires et supports amovibles ;
- `/opt` : logiciels additionnels installés hors des paquets de la distribution.

Connaître ces emplacements permet de savoir où chercher une configuration (`/etc`) ou un message d'erreur (`/var/log`).

## Chemins absolus et relatifs

- Un chemin **absolu** commence par `/` et part de la racine : `/var/log/syslog`.
- Un chemin **relatif** part du répertoire courant : depuis `/var`, le chemin `log/syslog` désigne le même fichier.
- `.` désigne le répertoire courant, `..` le répertoire parent et `~` le répertoire personnel.

```
cd /var
ls log
ls ../etc
ls ~/Documents
```

## Créer

```
mkdir projets
mkdir -p projets/site/images
touch notes.txt
```

- `mkdir` crée un répertoire ; `-p` crée aussi les répertoires parents manquants, sans erreur s'ils existent déjà.
- `touch` crée un fichier vide, ou met à jour sa date de modification s'il existe.

## Copier, déplacer, renommer

```
cp notes.txt notes.bak
cp -r projets projets-sauvegarde
mv notes.bak archives/
mv ancien-nom.txt nouveau-nom.txt
```

- `cp source destination` copie ; `-r` (récursif) est nécessaire pour copier un répertoire.
- `mv` déplace **et** renomme : renommer, c'est déplacer vers un nouveau nom.
- Si la destination existe déjà, elle est remplacée sans avertissement. L'option `-i` demande une confirmation.

## Supprimer avec prudence

```
rm fichier.txt
rmdir repertoire-vide
rm -r repertoire
```

- `rm` supprime des fichiers ; `rm -r` supprime un répertoire et tout son contenu ; `rmdir` ne supprime qu'un répertoire vide.
- Il n'y a **pas de corbeille** en ligne de commande : un fichier supprimé est perdu.
- Avant un `rm` avec un motif comme `*.log`, vérifiez d'abord ce qui sera concerné avec `ls *.log`.
- L'option `-i` demande une confirmation pour chaque fichier. Méfiez-vous de `-f`, qui supprime sans rien demander.

## Lire un fichier

```
cat /etc/hostname
less /var/log/syslog
head -n 20 /etc/passwd
tail -n 50 /var/log/syslog
tail -f /var/log/syslog
```

- `cat` affiche tout le fichier : adapté aux fichiers courts.
- `less` permet de parcourir un long fichier : flèches pour défiler, `/mot` pour chercher, `q` pour quitter.
- `head` et `tail` affichent le début et la fin ; `-n` précise le nombre de lignes.
- `tail -f` suit un fichier en temps réel : idéal pour observer un journal pendant un dépannage. Ctrl+C pour arrêter.

## Les motifs du shell

Le shell remplace certains caractères par la liste des fichiers correspondants **avant** de lancer la commande :

- `*` : n'importe quelle suite de caractères (`ls *.conf`) ;
- `?` : exactement un caractère (`ls fichier?.txt`) ;
- `[abc]` : un caractère parmi ceux listés (`ls rapport[123].pdf`).

Comme le remplacement est fait par le shell, la commande reçoit directement la liste des noms. Si aucun fichier ne correspond, Bash transmet le motif tel quel.

## À retenir

- Une seule arborescence à partir de `/` ; configuration dans `/etc`, journaux dans `/var/log`.
- Chemin absolu : depuis `/` ; chemin relatif : depuis le répertoire courant.
- `mkdir -p`, `cp -r`, `mv` pour créer, copier, déplacer et renommer.
- Pas de corbeille : vérifiez avec `ls` avant `rm`.
- `cat`, `less`, `head`, `tail -f` pour lire ; `*`, `?` et `[ ]` pour désigner plusieurs fichiers.

# Utilisateurs, groupes et permissions

Linux est un système multi-utilisateurs : chaque fichier appartient à un utilisateur et à un groupe, et chaque processus s'exécute sous une identité. Les permissions déterminent qui peut lire, modifier ou exécuter quoi.

## Qui suis-je ?

```
whoami
id
groups
```

- `whoami` affiche le nom de l'utilisateur courant ;
- `id` affiche son identifiant numérique **UID**, celui de son groupe principal **GID** et la liste de ses groupes ;
- `groups` liste les groupes de l'utilisateur.

Le système raisonne avec les numéros, pas avec les noms. L'utilisateur **root** a l'UID 0 : il n'est soumis à aucune restriction de permissions.

## Où sont stockés les comptes

- `/etc/passwd` contient une ligne par compte : nom, UID, GID principal, description, répertoire personnel et shell, séparés par `:`. Ce fichier est lisible par tous.
- `/etc/shadow` contient les empreintes des mots de passe et leur politique d'expiration. Il n'est lisible que par root.
- `/etc/group` décrit les groupes et leurs membres.

```
alice:x:1000:1000:Alice Martin:/home/alice:/bin/bash
```

Le `x` indique que le mot de passe est stocké dans `/etc/shadow`.

## Lire les permissions

`ls -l` affiche les permissions en tête de chaque ligne :

```
-rw-r----- 1 alice compta 2048 nov.  3 10:12 budget.ods
drwxr-x--- 2 alice compta 4096 nov.  3 10:15 rapports
```

- Le premier caractère indique le type : `-` fichier ordinaire, `d` répertoire, `l` lien symbolique.
- Les neuf suivants forment trois groupes de trois : le **propriétaire** (u), le **groupe** (g) et les **autres** (o).
- Dans chaque groupe : `r` lecture, `w` écriture, `x` exécution ; un tiret signifie que le droit est absent.

Ici, alice peut lire et modifier `budget.ods`, les membres du groupe `compta` peuvent seulement le lire, et les autres n'ont aucun droit.

Pour un **répertoire**, les droits ont un sens particulier :

- `r` permet de lister son contenu ;
- `w` permet d'y créer, renommer ou supprimer des fichiers ;
- `x` permet de le traverser, c'est-à-dire d'y entrer et d'accéder aux fichiers qu'il contient.

Sans `x` sur un répertoire, ses fichiers sont inaccessibles, même s'ils sont eux-mêmes lisibles.

## Modifier les permissions avec chmod

**Notation symbolique** : qui (`u`, `g`, `o`, `a` pour tous), opération (`+`, `-`, `=`), droits.

```
chmod u+x script.sh
chmod g-w rapport.txt
chmod o= secret.txt
chmod a+r public.html
```

**Notation octale** : chaque droit a une valeur (`r` = 4, `w` = 2, `x` = 1) ; on additionne pour chaque groupe.

```
chmod 644 page.html
chmod 755 script.sh
chmod 600 cle-privee
chmod 750 rapports
```

- `644` (`rw-r--r--`) : fichier lisible par tous, modifiable par le propriétaire ;
- `755` (`rwxr-xr-x`) : programme ou répertoire public ;
- `600` (`rw-------`) : fichier privé, par exemple une clé SSH ;
- `750` (`rwxr-x---`) : répertoire réservé au propriétaire et à son groupe.

## Changer de propriétaire

```
sudo chown bob rapport.txt
sudo chown bob:compta rapport.txt
sudo chgrp compta rapport.txt
sudo chown -R www-data:www-data /var/www/site
```

Changer le propriétaire d'un fichier est réservé à root ; `-R` applique la modification récursivement.

## Le masque par défaut : umask

Les nouveaux fichiers ne reçoivent pas tous les droits : le **masque** `umask` retire des droits à la création.

```
umask
```

Avec le masque courant `022`, un nouveau fichier est créé en `644` et un nouveau répertoire en `755` : le droit d'écriture est retiré au groupe et aux autres.

## Agir en administrateur

Travailler en permanence en root est dangereux : la moindre erreur de frappe peut endommager le système. On applique le **principe du moindre privilège** : utilisateur ordinaire au quotidien, droits d'administration seulement quand c'est nécessaire.

```
sudo apt update
sudo -l
```

- `sudo commande` exécute une seule commande avec les droits de root, après avoir demandé **votre** mot de passe ; chaque utilisation est journalisée.
- `sudo -l` liste ce que vous êtes autorisé à faire.
- Les autorisations sont définies dans `/etc/sudoers`, à modifier uniquement avec `visudo`, qui vérifie la syntaxe avant d'enregistrer.

Les droits spéciaux (setuid, setgid, sticky bit) et les listes de contrôle d'accès (ACL) feront l'objet d'un chapitre dédié.

## À retenir

- Chaque fichier a un propriétaire, un groupe et trois triplets de droits `rwx`.
- Sur un répertoire, `x` permet de le traverser.
- `chmod` en symbolique (`u+x`) ou en octal (`r`=4, `w`=2, `x`=1).
- `chown` et `chgrp` changent propriétaire et groupe (réservé à root).
- `umask 022` donne `644` pour les fichiers et `755` pour les répertoires.
- `sudo` plutôt qu'une session root permanente.

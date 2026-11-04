# Le terminal et le shell

Administrer un serveur Linux, c'est avant tout dialoguer avec lui en ligne de commande. Trois notions sont souvent confondues :

- le **terminal** (ou émulateur de terminal) est la fenêtre qui affiche le texte et transmet ce que vous tapez ;
- la **console** désigne historiquement l'écran et le clavier physiquement reliés à la machine ;
- le **shell** est le programme qui lit vos commandes, les interprète et lance les programmes correspondants. Sur la plupart des distributions, le shell par défaut est **Bash**.

Le terminal n'est donc qu'un intermédiaire : c'est le shell qui exécute réellement le travail.

## Lire l'invite de commande

Quand le shell attend une commande, il affiche une **invite** (en anglais *prompt*) :

```
alice@srv-web:~$
```

- `alice` : l'utilisateur connecté ;
- `srv-web` : le nom de la machine ;
- `~` : le répertoire courant (`~` désigne le répertoire personnel) ;
- `$` : vous êtes un utilisateur ordinaire. Un `#` indique que vous êtes **root**, l'administrateur : chaque commande a alors tous les droits, prudence.

L'apparence de l'invite varie selon la configuration, mais ces informations sont presque toujours présentes.

## Anatomie d'une commande

Une commande se compose d'un nom, d'options et d'arguments, séparés par des espaces :

```
ls -l -a /etc
```

- `ls` est la **commande** (lister le contenu d'un répertoire) ;
- `-l` et `-a` sont des **options** : elles modifient le comportement de la commande ;
- `/etc` est un **argument** : ce sur quoi la commande agit.

Quelques règles à connaître :

- Linux distingue les majuscules et les minuscules : `ls` fonctionne, `LS` non.
- Les options courtes commencent par un tiret et peuvent souvent être regroupées : `ls -la` équivaut à `ls -l -a`.
- Les options longues commencent par deux tirets : `ls --all`.
- Un argument contenant des espaces doit être placé entre guillemets : `ls "Mes documents"`.

## Trouver de l'aide

Personne ne retient toutes les options. Savoir chercher l'information est une compétence à part entière.

- `man ls` affiche le **manuel** de la commande. Dans le manuel, `/mot` recherche un mot, `n` passe à l'occurrence suivante et `q` quitte.
- `ls --help` affiche un résumé des options pour la plupart des commandes.
- `type cd` indique si une commande est un programme ou une **commande interne** du shell.
- `help cd` affiche l'aide des commandes internes de Bash, qui n'ont souvent pas de page de manuel dédiée.
- `apropos copie` (ou `man -k`) recherche les pages de manuel dont la description contient un mot.

## Se déplacer

Le shell travaille toujours dans un **répertoire courant**.

```
pwd
ls
ls -lh /var/log
cd /etc
cd ..
cd
cd -
```

- `pwd` affiche le répertoire courant ;
- `ls` liste son contenu ; `-l` donne le détail (droits, propriétaire, taille, date), `-h` rend les tailles lisibles et `-a` montre aussi les fichiers cachés, dont le nom commence par un point ;
- `cd /etc` change de répertoire ;
- `cd ..` remonte d'un niveau ;
- `cd` sans argument (ou `cd ~`) revient au répertoire personnel ;
- `cd -` revient au répertoire précédent.

## Gagner du temps

- **Tab** complète automatiquement un nom de commande, de fichier ou de répertoire. Deux appuis successifs affichent les possibilités quand il y en a plusieurs. Utilisez-la systématiquement : elle évite les fautes de frappe.
- **Flèche haut** rappelle les commandes précédentes ; `history` les liste toutes.
- **Ctrl+R** recherche dans l'historique : tapez un fragment, puis Entrée pour exécuter ou Échap pour éditer.
- **Ctrl+C** interrompt la commande en cours.
- **Ctrl+L** (ou `clear`) efface l'écran.
- **Ctrl+D** signale la fin de saisie ; dans un shell vide, il ferme la session.

## À retenir

- Le terminal affiche, le shell exécute.
- `$` : utilisateur ordinaire ; `#` : root.
- Une commande = nom + options + arguments ; la casse compte.
- `man`, `--help`, `help` et `type` sont vos premiers réflexes.
- `pwd`, `ls`, `cd` pour se repérer et se déplacer ; Tab et Ctrl+R pour aller vite.

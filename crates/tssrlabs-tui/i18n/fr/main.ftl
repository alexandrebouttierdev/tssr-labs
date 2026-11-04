# Textes de l'interface — français.
# Chaque identifiant doit être déclaré dans `src/i18n.rs` (vérifié par les tests).

## Application

app-title = TSSR Labs
app-subtitle = Plateforme pédagogique systèmes et réseaux

## Écrans

screen-home = Accueil
screen-profiles = Profils
screen-diagnostic = Diagnostic

## Accueil

home-version = Version { $version }
home-upcoming = Les exercices guidés arrivent dans la prochaine version.
home-greeting = Bonjour, { $name }
home-no-activity = Aucune activité enregistrée pour ce profil.
home-no-profile = Aucun profil actif.
home-no-profile-hint = Ouvrez l'écran Profils pour créer ou choisir un profil.
loading-profiles = Chargement des profils…

## Profils

profiles-empty = Aucun profil pour l'instant.
profiles-empty-hint = Appuyez sur { $key } pour créer votre premier profil.
profiles-col-name = Nom
profiles-col-created = Créé le
profiles-col-last-used = Dernière utilisation
profiles-col-state = État
profiles-active = actif
profiles-never = jamais
profiles-local-note = Profils locaux : aucun compte, aucune donnée envoyée sur le réseau.

## Fenêtres de dialogue

dialog-create-title = Nouveau profil
dialog-rename-title = Renommer « { $name } »
dialog-name-label = Nom du profil ({ $max } caractères au plus) :
dialog-delete-title = Supprimer « { $name } »
dialog-delete-warning = La progression, les notes et les préférences de ce profil seront supprimées.
dialog-delete-backup = Une copie de sécurité de la base est créée avant la suppression.
dialog-delete-prompt = Saisissez le nom du profil pour confirmer :
dialog-busy = Opération en cours…
dialog-hint = { $confirm } : valider · { $cancel } : annuler

## Messages

notice-info = Info : { $text }
notice-error = Erreur : { $text }
error-name-empty = le nom ne peut pas être vide.
error-name-too-long = le nom ne doit pas dépasser { $max } caractères.
error-name-control = le nom contient un caractère non imprimable.
error-name-taken = ce nom est déjà utilisé par un autre profil.
error-not-found = ce profil n'existe plus.
error-confirmation = la saisie ne correspond pas au nom du profil.
error-storage = problème de stockage ({ $detail }).
status-created = profil « { $name } » créé et activé.
status-renamed = profil renommé en « { $name } ».
status-selected = profil « { $name } » activé.
status-deleted = profil supprimé. Copie de sécurité : { $path }
status-prefs-error = préférences non enregistrées : { $detail }

## Commandes clavier

cmd-quit = Quitter
cmd-help = Aide
cmd-close = Fermer
cmd-next-screen = Écran suivant
cmd-prev-screen = Écran précédent
cmd-cycle-theme = Thème
cmd-up = Monter
cmd-down = Descendre
cmd-activate = Activer
cmd-new-profile = Nouveau
cmd-rename-profile = Renommer
cmd-delete-profile = Supprimer

## Noms des touches

key-esc = Échap
key-enter = Entrée
key-tab = Tab
key-backtab = Maj+Tab
key-space = Espace
key-up = ↑
key-down = ↓
key-left = ←
key-right = →
key-ctrl = Ctrl
key-alt = Alt
key-delete = Suppr

## Aide

help-title = Raccourcis clavier
help-close = { $key } : fermer
help-layout-note = Les lettres correspondent au caractère produit : elles fonctionnent à l'identique en AZERTY et en QWERTY.

## Thèmes

theme-dark = Sombre
theme-high-contrast = Contraste élevé
theme-colorblind = Daltonisme
theme-monochrome = Monochrome
status-theme = Thème : { $name }
status-profile = Profil : { $name }
status-no-profile = aucun profil actif

## Terminal trop petit

too-small-title = Terminal trop petit
too-small-size = Taille actuelle : { $width }×{ $height } — minimum : { $min_width }×{ $min_height }.
too-small-hint = Agrandissez la fenêtre ou réduisez la police du terminal.
too-small-quit = { $key } : quitter

## Diagnostic

diag-version = Version
diag-system = Système
diag-terminal = Terminal
diag-term = Variable TERM
diag-colors = Couleurs
diag-size = Taille
diag-theme = Thème
diag-data = Données
diag-config = Configuration
diag-logs = Journaux
diag-database = Base de données
diag-note = Ces informations restent sur cet ordinateur.
value-unknown = inconnu
colors-none = aucune (texte seul)
colors-basic = 16 couleurs
colors-256 = 256 couleurs
colors-truecolor = 24 bits
colors-forced = { $level } (imposé par TSSRLABS_COLORS)
layout-full = affichage complet
layout-compact = affichage compact
size-value = { $width }×{ $height } — { $mode }

## Contenu pédagogique

screen-paths = Parcours
screen-library = Bibliothèque
screen-reader = Chapitre
cmd-open = Ouvrir
cmd-back = Retour
cmd-focus-search = Rechercher
cmd-cycle-domain = Domaine
cmd-cycle-difficulty = Niveau
cmd-cycle-duration = Durée
cmd-clear-filters = Effacer les filtres
cmd-page-up = Page précédente
cmd-page-down = Page suivante
cmd-top = Début
cmd-bottom = Fin
key-page-up = PgPréc
key-page-down = PgSuiv
key-home = Début
key-end = Fin
domain-linux = Linux
domain-windows = Windows
domain-reseau = Réseaux
domain-systeme = Systèmes
domain-securite = Sécurité
domain-virtualisation = Virtualisation
domain-depannage = Dépannage
domain-automatisation = Automatisation
difficulty-debutant = Débutant
difficulty-intermediaire = Intermédiaire
difficulty-avance = Avancé
library-search = Recherche :
library-search-hint = { $key } pour saisir un mot-clé
library-search-done = { $enter } ou { $esc } : terminer la saisie
library-filter-domain = Domaine : { $value }
library-filter-difficulty = Niveau : { $value }
library-filter-duration = Durée : { $value }
library-all = tous
library-any-duration = toutes
library-max-minutes = ≤ { $minutes } min
library-results = { $count ->
    [0] Aucun chapitre
    [one] 1 chapitre
   *[other] { $count } chapitres
}
library-no-match = Aucun chapitre ne correspond à ces critères.
library-col-title = Titre
library-col-domain = Domaine
library-col-level = Niveau
library-col-duration = Durée
minutes = { $minutes } min
paths-summary = { $count ->
    [one] 1 chapitre
   *[other] { $count } chapitres
}
paths-empty = Aucun parcours disponible.
paths-prerequisites = Prérequis : { $list }
paths-no-prerequisite = Aucun prérequis.
reader-objectives = Objectifs
reader-prerequisites = Prérequis
reader-skills = Compétences travaillées
reader-sources = Sources
reader-verified = vérifiée le { $date }
reader-unverified = non vérifiée
reader-meta = { $domain } · { $difficulty } · environ { $minutes } min · version { $version }
reader-credits = Auteurs : { $authors } · Licence : { $license }
reader-position = Position : { $percent } %
reader-missing = Ce chapitre n'existe plus.
home-content = Contenu disponible : { $paths } parcours, { $chapters } chapitres.
home-start-hint = Ouvrez « Parcours » ou « Bibliothèque » pour commencer.
diag-content = Contenu
diag-content-value = { $paths } parcours, { $chapters } chapitres (format { $format }), { $warnings ->
    [0] aucun avertissement
    [one] 1 avertissement
   *[other] { $warnings } avertissements
}

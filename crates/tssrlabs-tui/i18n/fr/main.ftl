# Textes de l'interface — français.
# Chaque identifiant doit être déclaré dans `src/i18n.rs` (vérifié par les tests).

## Application

app-title = TSSR Labs
app-subtitle = Plateforme pédagogique systèmes et réseaux

## Écrans

screen-home = Accueil
screen-diagnostic = Diagnostic

## Accueil

home-version = Version { $version } — socle technique
home-upcoming = Les profils, les cours et le tableau de bord arrivent dans les prochaines versions.

## Commandes clavier

cmd-quit = Quitter
cmd-help = Aide
cmd-close = Fermer
cmd-next-screen = Écran suivant
cmd-prev-screen = Écran précédent
cmd-cycle-theme = Thème
cmd-up = Monter
cmd-down = Descendre

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
status-compact = compact

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

//! Raccourcis clavier : commandes, contextes, liaisons et conflits.
//!
//! Les liaisons portent sur le **caractère produit** par la touche (et non
//! sur sa position physique) : `q` ou `?` fonctionnent à l'identique en
//! AZERTY et en QWERTY. Les lettres sont insensibles à la casse (Verr. Maj).
//! Chaque commande essentielle possède aussi une touche non alphabétique
//! (F1, Tab, flèches) indépendante de la disposition.
//!
//! Chaque commande appartient à un [`Context`] : une même touche peut servir
//! dans deux écrans différents. Un conflit n'existe qu'entre commandes d'un
//! même contexte, ou entre une commande globale et une autre.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::i18n::{Localizer, msg};

/// Contexte d'application d'une commande.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    /// Partout.
    Global,
    /// Écran Profils.
    Profiles,
    /// Bibliothèque.
    Library,
    /// Lecture d'un chapitre.
    Reader,
    /// Exercice guidé.
    Exercise,
}

/// Commandes déclenchables au clavier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    /// Quitter l'application.
    Quit,
    /// Afficher ou masquer l'aide.
    ToggleHelp,
    /// Fermer / revenir.
    Close,
    /// Écran suivant.
    NextScreen,
    /// Écran précédent.
    PrevScreen,
    /// Thème suivant.
    CycleTheme,
    /// Élément précédent / défiler vers le haut.
    Up,
    /// Élément suivant / défiler vers le bas.
    Down,
    /// Activer ou ouvrir l'élément sélectionné.
    Activate,
    /// Créer un profil.
    NewProfile,
    /// Renommer le profil sélectionné.
    RenameProfile,
    /// Supprimer le profil sélectionné.
    DeleteProfile,
    /// Saisir une recherche.
    FocusSearch,
    /// Filtre de domaine suivant.
    CycleDomain,
    /// Filtre de niveau suivant.
    CycleDifficulty,
    /// Filtre de durée suivant.
    CycleDuration,
    /// Effacer recherche et filtres.
    ClearFilters,
    /// Page précédente.
    PageUp,
    /// Page suivante.
    PageDown,
    /// Début.
    Top,
    /// Fin.
    Bottom,
    /// Ouvrir les exercices du chapitre.
    OpenExercises,
    /// Afficher l'indice suivant.
    Hint,
    /// Afficher la solution.
    ShowSolution,
    /// Exercice suivant.
    NextExercise,
    /// Exercice précédent.
    PrevExercise,
    /// Cocher ou décocher la proposition sélectionnée.
    ToggleChoice,
}

impl Command {
    /// Ordre d'affichage dans l'aide.
    pub const ALL: [Self; 27] = [
        Self::NextScreen,
        Self::PrevScreen,
        Self::Up,
        Self::Down,
        Self::Activate,
        Self::Close,
        Self::CycleTheme,
        Self::ToggleHelp,
        Self::Quit,
        Self::NewProfile,
        Self::RenameProfile,
        Self::DeleteProfile,
        Self::FocusSearch,
        Self::CycleDomain,
        Self::CycleDifficulty,
        Self::CycleDuration,
        Self::ClearFilters,
        Self::PageUp,
        Self::PageDown,
        Self::Top,
        Self::Bottom,
        Self::OpenExercises,
        Self::ToggleChoice,
        Self::Hint,
        Self::ShowSolution,
        Self::NextExercise,
        Self::PrevExercise,
    ];

    /// Contexte de la commande.
    pub fn context(self) -> Context {
        match self {
            Self::NewProfile | Self::RenameProfile | Self::DeleteProfile => Context::Profiles,
            Self::FocusSearch
            | Self::CycleDomain
            | Self::CycleDifficulty
            | Self::CycleDuration
            | Self::ClearFilters => Context::Library,
            Self::PageUp | Self::PageDown | Self::Top | Self::Bottom | Self::OpenExercises => {
                Context::Reader
            }
            Self::Hint
            | Self::ShowSolution
            | Self::NextExercise
            | Self::PrevExercise
            | Self::ToggleChoice => Context::Exercise,
            _ => Context::Global,
        }
    }

    /// Identifiant du libellé localisé.
    pub fn label_id(self) -> &'static str {
        match self {
            Self::Quit => msg::CMD_QUIT,
            Self::ToggleHelp => msg::CMD_HELP,
            Self::Close => msg::CMD_CLOSE,
            Self::NextScreen => msg::CMD_NEXT_SCREEN,
            Self::PrevScreen => msg::CMD_PREV_SCREEN,
            Self::CycleTheme => msg::CMD_CYCLE_THEME,
            Self::Up => msg::CMD_UP,
            Self::Down => msg::CMD_DOWN,
            Self::Activate => msg::CMD_ACTIVATE,
            Self::NewProfile => msg::CMD_NEW_PROFILE,
            Self::RenameProfile => msg::CMD_RENAME_PROFILE,
            Self::DeleteProfile => msg::CMD_DELETE_PROFILE,
            Self::FocusSearch => msg::CMD_FOCUS_SEARCH,
            Self::CycleDomain => msg::CMD_CYCLE_DOMAIN,
            Self::CycleDifficulty => msg::CMD_CYCLE_DIFFICULTY,
            Self::CycleDuration => msg::CMD_CYCLE_DURATION,
            Self::ClearFilters => msg::CMD_CLEAR_FILTERS,
            Self::PageUp => msg::CMD_PAGE_UP,
            Self::PageDown => msg::CMD_PAGE_DOWN,
            Self::Top => msg::CMD_TOP,
            Self::Bottom => msg::CMD_BOTTOM,
            Self::OpenExercises => msg::CMD_OPEN_EXERCISES,
            Self::Hint => msg::CMD_HINT,
            Self::ShowSolution => msg::CMD_SHOW_SOLUTION,
            Self::NextExercise => msg::CMD_NEXT_EXERCISE,
            Self::PrevExercise => msg::CMD_PREV_EXERCISE,
            Self::ToggleChoice => msg::CMD_TOGGLE_CHOICE,
        }
    }
}

/// Combinaison de touches normalisée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Touche avec modificateurs (normalisée).
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        normalize(code, modifiers)
    }

    /// Touche sans modificateur.
    pub fn plain(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    /// Caractère avec Ctrl.
    pub fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Liaison correspondant à un événement clavier.
    pub fn from_event(key: &KeyEvent) -> Self {
        Self::new(key.code, key.modifiers)
    }

    /// Libellé lisible, localisé (ex. « Ctrl+C », « Échap », « F1 »).
    pub fn label(&self, i18n: &Localizer) -> String {
        let mut parts = Vec::new();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push(i18n.t(msg::KEY_CTRL));
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push(i18n.t(msg::KEY_ALT));
        }
        let key = match self.code {
            KeyCode::Char(' ') => i18n.t(msg::KEY_SPACE),
            KeyCode::Char(c) if !self.modifiers.is_empty() => c.to_uppercase().collect(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::F(n) => format!("F{n}"),
            KeyCode::Esc => i18n.t(msg::KEY_ESC),
            KeyCode::Enter => i18n.t(msg::KEY_ENTER),
            KeyCode::Tab => i18n.t(msg::KEY_TAB),
            KeyCode::BackTab => i18n.t(msg::KEY_BACKTAB),
            KeyCode::Up => i18n.t(msg::KEY_UP),
            KeyCode::Down => i18n.t(msg::KEY_DOWN),
            KeyCode::Left => i18n.t(msg::KEY_LEFT),
            KeyCode::Right => i18n.t(msg::KEY_RIGHT),
            KeyCode::Delete => i18n.t(msg::KEY_DELETE),
            KeyCode::PageUp => i18n.t(msg::KEY_PAGE_UP),
            KeyCode::PageDown => i18n.t(msg::KEY_PAGE_DOWN),
            KeyCode::Home => i18n.t(msg::KEY_HOME),
            KeyCode::End => i18n.t(msg::KEY_END),
            other => format!("{other:?}"),
        };
        parts.push(key);
        parts.join("+")
    }
}

fn normalize(code: KeyCode, modifiers: KeyModifiers) -> KeyBinding {
    // Seuls Ctrl et Alt sont significatifs ; Maj est déjà reflété par le
    // caractère produit (« ? », « Q ») ou par BackTab.
    let modifiers = modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT);
    let code = match code {
        KeyCode::Char(c) if c.is_alphabetic() => {
            KeyCode::Char(c.to_lowercase().next().unwrap_or(c))
        }
        other => other,
    };
    KeyBinding { code, modifiers }
}

/// Liaison partagée par des commandes qui peuvent être actives ensemble.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// Touche en conflit.
    pub binding: KeyBinding,
    /// Commandes concernées.
    pub commands: Vec<Command>,
}

/// Table des raccourcis.
#[derive(Debug, Clone)]
pub struct Keymap {
    bindings: Vec<(KeyBinding, Command)>,
}

impl Default for Keymap {
    fn default() -> Self {
        use KeyCode::{
            BackTab, Char, Delete, Down, End, Enter, Esc, F, Home, PageDown, PageUp, Tab, Up,
        };
        let plain = KeyBinding::plain;
        Self {
            bindings: vec![
                (plain(Char('q')), Command::Quit),
                (KeyBinding::ctrl('c'), Command::Quit),
                (plain(F(1)), Command::ToggleHelp),
                (plain(Char('?')), Command::ToggleHelp),
                (plain(Esc), Command::Close),
                (plain(Tab), Command::NextScreen),
                (plain(BackTab), Command::PrevScreen),
                (plain(Char('t')), Command::CycleTheme),
                (plain(Up), Command::Up),
                (plain(Char('k')), Command::Up),
                (plain(Down), Command::Down),
                (plain(Char('j')), Command::Down),
                (plain(Enter), Command::Activate),
                (plain(Char('n')), Command::NewProfile),
                (plain(Char('r')), Command::RenameProfile),
                (plain(Char('s')), Command::DeleteProfile),
                (plain(Delete), Command::DeleteProfile),
                (plain(Char('/')), Command::FocusSearch),
                (plain(Char('d')), Command::CycleDomain),
                (plain(Char('f')), Command::CycleDifficulty),
                (plain(Char('m')), Command::CycleDuration),
                (plain(Char('x')), Command::ClearFilters),
                (plain(PageUp), Command::PageUp),
                (plain(PageDown), Command::PageDown),
                (plain(Char(' ')), Command::PageDown),
                (plain(Home), Command::Top),
                (plain(End), Command::Bottom),
                (plain(Char('e')), Command::OpenExercises),
                (plain(Char(' ')), Command::ToggleChoice),
                (plain(F(2)), Command::Hint),
                (plain(F(3)), Command::ShowSolution),
                (plain(PageDown), Command::NextExercise),
                (plain(PageUp), Command::PrevExercise),
            ],
        }
    }
}

fn may_overlap(a: Context, b: Context) -> bool {
    a == b || a == Context::Global || b == Context::Global
}

impl Keymap {
    /// Commande associée à un événement clavier dans un contexte (appuis
    /// uniquement : sous Windows, les relâchements sont aussi signalés).
    pub fn command_for(&self, key: &KeyEvent, context: Context) -> Option<Command> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        let binding = KeyBinding::from_event(key);
        self.bindings
            .iter()
            .filter(|(_, c)| c.context() == Context::Global || c.context() == context)
            .find(|(b, _)| *b == binding)
            .map(|(_, command)| *command)
    }

    /// Liaisons d'une commande, dans l'ordre de déclaration.
    pub fn bindings_for(&self, command: Command) -> impl Iterator<Item = KeyBinding> + '_ {
        self.bindings
            .iter()
            .filter(move |(_, c)| *c == command)
            .map(|(b, _)| *b)
    }

    /// Libellé de la première liaison d'une commande, s'il y en a une.
    pub fn primary_label(&self, command: Command, i18n: &Localizer) -> Option<String> {
        self.bindings_for(command).next().map(|b| b.label(i18n))
    }

    /// Ajoute une liaison. Les conflits éventuels sont signalés par
    /// [`Self::conflicts`] et doivent être résolus avant enregistrement.
    pub fn bind(&mut self, binding: KeyBinding, command: Command) {
        if !self.bindings.contains(&(binding, command)) {
            self.bindings.push((binding, command));
        }
    }

    /// Touches liées à plusieurs commandes pouvant être actives en même temps.
    pub fn conflicts(&self) -> Vec<Conflict> {
        let mut conflicts: Vec<Conflict> = Vec::new();
        for (i, (binding, a)) in self.bindings.iter().enumerate() {
            for (other, b) in &self.bindings[i + 1..] {
                if other != binding || a == b || !may_overlap(a.context(), b.context()) {
                    continue;
                }
                match conflicts.iter_mut().find(|c| c.binding == *binding) {
                    Some(conflict) => {
                        for command in [*a, *b] {
                            if !conflict.commands.contains(&command) {
                                conflict.commands.push(command);
                            }
                        }
                    }
                    None => conflicts.push(Conflict {
                        binding: *binding,
                        commands: vec![*a, *b],
                    }),
                }
            }
        }
        conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use crossterm::event::KeyEventState;

    fn event(code: KeyCode, modifiers: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind,
            state: KeyEventState::NONE,
        }
    }

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        event(code, modifiers, KeyEventKind::Press)
    }

    fn global(keymap: &Keymap, code: KeyCode, modifiers: KeyModifiers) -> Option<Command> {
        keymap.command_for(&press(code, modifiers), Context::Global)
    }

    #[test]
    fn default_keymap_has_no_conflict() {
        assert_eq!(Keymap::default().conflicts(), vec![]);
    }

    #[test]
    fn every_command_has_a_binding() {
        let keymap = Keymap::default();
        for command in Command::ALL {
            assert!(keymap.bindings_for(command).next().is_some(), "{command:?}");
        }
    }

    #[test]
    fn letters_ignore_case_and_shift() {
        let keymap = Keymap::default();
        assert_eq!(
            global(&keymap, KeyCode::Char('Q'), KeyModifiers::SHIFT),
            Some(Command::Quit)
        );
        assert_eq!(
            global(&keymap, KeyCode::Char('C'), KeyModifiers::CONTROL),
            Some(Command::Quit)
        );
    }

    #[test]
    fn shifted_symbols_match_produced_character() {
        // AZERTY : « ? » s'obtient avec Maj ; seul le caractère compte.
        let keymap = Keymap::default();
        assert_eq!(
            global(&keymap, KeyCode::Char('?'), KeyModifiers::SHIFT),
            Some(Command::ToggleHelp)
        );
        assert_eq!(
            global(&keymap, KeyCode::BackTab, KeyModifiers::SHIFT),
            Some(Command::PrevScreen)
        );
    }

    #[test]
    fn releases_and_plain_c_are_ignored() {
        let keymap = Keymap::default();
        assert_eq!(
            keymap.command_for(
                &event(
                    KeyCode::Char('q'),
                    KeyModifiers::NONE,
                    KeyEventKind::Release
                ),
                Context::Global
            ),
            None
        );
        assert_eq!(
            global(&keymap, KeyCode::Char('c'), KeyModifiers::NONE),
            None
        );
    }

    #[test]
    fn screen_commands_only_apply_in_their_context() {
        let keymap = Keymap::default();
        let n = press(KeyCode::Char('n'), KeyModifiers::NONE);
        assert_eq!(
            keymap.command_for(&n, Context::Profiles),
            Some(Command::NewProfile)
        );
        assert_eq!(keymap.command_for(&n, Context::Library), None);
        let d = press(KeyCode::Char('d'), KeyModifiers::NONE);
        assert_eq!(
            keymap.command_for(&d, Context::Library),
            Some(Command::CycleDomain)
        );
        assert_eq!(keymap.command_for(&d, Context::Profiles), None);
        // Les commandes globales restent disponibles partout.
        let q = press(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(keymap.command_for(&q, Context::Reader), Some(Command::Quit));
    }

    #[test]
    fn same_key_in_distinct_screens_is_not_a_conflict() {
        let mut keymap = Keymap::default();
        keymap.bind(KeyBinding::plain(KeyCode::Char('n')), Command::CycleDomain);
        assert_eq!(keymap.conflicts(), vec![]);
    }

    #[test]
    fn conflicting_binding_is_reported() {
        let mut keymap = Keymap::default();
        keymap.bind(KeyBinding::plain(KeyCode::Char('Q')), Command::CycleDomain);
        let conflicts = keymap.conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].binding, KeyBinding::plain(KeyCode::Char('q')));
        assert_eq!(
            conflicts[0].commands,
            vec![Command::Quit, Command::CycleDomain]
        );
    }

    #[test]
    fn labels_are_localized() {
        let i18n = Localizer::new(Language::French);
        assert_eq!(KeyBinding::ctrl('c').label(&i18n), "Ctrl+C");
        assert_eq!(KeyBinding::plain(KeyCode::Esc).label(&i18n), "Échap");
        assert_eq!(KeyBinding::plain(KeyCode::F(1)).label(&i18n), "F1");
        assert_eq!(KeyBinding::plain(KeyCode::Char('?')).label(&i18n), "?");
        assert_eq!(KeyBinding::plain(KeyCode::PageDown).label(&i18n), "PgSuiv");
    }
}

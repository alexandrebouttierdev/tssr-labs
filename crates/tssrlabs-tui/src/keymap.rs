//! Raccourcis clavier : commandes, liaisons et détection des conflits.
//!
//! Les liaisons portent sur le **caractère produit** par la touche (et non
//! sur sa position physique) : `q` ou `?` fonctionnent à l'identique en
//! AZERTY et en QWERTY. Les lettres sont insensibles à la casse (Verr. Maj).
//! Chaque commande essentielle possède aussi une touche non alphabétique
//! (F1, Tab, flèches) indépendante de la disposition.
//!
//! La personnalisation par l'utilisateur sera persistée avec les préférences
//! (tranche 2) ; [`Keymap::bind`] et [`Keymap::conflicts`] la préparent.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::i18n::{Localizer, msg};

/// Commandes déclenchables au clavier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    /// Quitter l'application.
    Quit,
    /// Afficher ou masquer l'aide.
    ToggleHelp,
    /// Fermer la fenêtre superposée active.
    Close,
    /// Écran suivant.
    NextScreen,
    /// Écran précédent.
    PrevScreen,
    /// Thème suivant.
    CycleTheme,
    /// Élément précédent.
    Up,
    /// Élément suivant.
    Down,
}

impl Command {
    /// Ordre d'affichage dans l'aide.
    pub const ALL: [Self; 8] = [
        Self::NextScreen,
        Self::PrevScreen,
        Self::Up,
        Self::Down,
        Self::CycleTheme,
        Self::ToggleHelp,
        Self::Close,
        Self::Quit,
    ];

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

/// Liaison partagée par plusieurs commandes.
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
        use KeyCode::{BackTab, Char, Down, Esc, F, Tab, Up};
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
            ],
        }
    }
}

impl Keymap {
    /// Commande associée à un événement clavier (appuis uniquement : sous
    /// Windows, les relâchements sont aussi signalés).
    pub fn command_for(&self, key: &KeyEvent) -> Option<Command> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        let binding = KeyBinding::from_event(key);
        self.bindings
            .iter()
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

    /// Touches liées à plusieurs commandes différentes.
    pub fn conflicts(&self) -> Vec<Conflict> {
        let mut conflicts: Vec<Conflict> = Vec::new();
        for (binding, command) in &self.bindings {
            match conflicts.iter_mut().find(|c| c.binding == *binding) {
                Some(conflict) if !conflict.commands.contains(command) => {
                    conflict.commands.push(*command);
                }
                Some(_) => {}
                None => conflicts.push(Conflict {
                    binding: *binding,
                    commands: vec![*command],
                }),
            }
        }
        conflicts.retain(|c| c.commands.len() > 1);
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
            keymap.command_for(&press(KeyCode::Char('Q'), KeyModifiers::SHIFT)),
            Some(Command::Quit)
        );
        assert_eq!(
            keymap.command_for(&press(KeyCode::Char('C'), KeyModifiers::CONTROL)),
            Some(Command::Quit)
        );
    }

    #[test]
    fn shifted_symbols_match_produced_character() {
        // AZERTY : « ? » s'obtient avec Maj ; seul le caractère compte.
        let keymap = Keymap::default();
        assert_eq!(
            keymap.command_for(&press(KeyCode::Char('?'), KeyModifiers::SHIFT)),
            Some(Command::ToggleHelp)
        );
        assert_eq!(
            keymap.command_for(&press(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(Command::PrevScreen)
        );
    }

    #[test]
    fn releases_and_plain_c_are_ignored() {
        let keymap = Keymap::default();
        assert_eq!(
            keymap.command_for(&event(
                KeyCode::Char('q'),
                KeyModifiers::NONE,
                KeyEventKind::Release
            )),
            None
        );
        assert_eq!(
            keymap.command_for(&press(KeyCode::Char('c'), KeyModifiers::NONE)),
            None
        );
    }

    #[test]
    fn conflicting_binding_is_reported() {
        let mut keymap = Keymap::default();
        keymap.bind(KeyBinding::plain(KeyCode::Char('Q')), Command::CycleTheme);
        let conflicts = keymap.conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].binding, KeyBinding::plain(KeyCode::Char('q')));
        assert_eq!(
            conflicts[0].commands,
            vec![Command::Quit, Command::CycleTheme]
        );
    }

    #[test]
    fn labels_are_localized() {
        let i18n = Localizer::new(Language::French);
        assert_eq!(KeyBinding::ctrl('c').label(&i18n), "Ctrl+C");
        assert_eq!(KeyBinding::plain(KeyCode::Esc).label(&i18n), "Échap");
        assert_eq!(KeyBinding::plain(KeyCode::F(1)).label(&i18n), "F1");
        assert_eq!(KeyBinding::plain(KeyCode::Char('?')).label(&i18n), "?");
    }
}

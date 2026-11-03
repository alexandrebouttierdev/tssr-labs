//! État de l'interface et traduction des événements en actions.

use std::path::PathBuf;

use crossterm::event::Event;
use tssrlabs_domain::host::HostInfo;

use crate::i18n::{Language, Localizer, msg};
use crate::keymap::{Command, Keymap};
use crate::screens::diagnostic;
use crate::theme::{Theme, ThemeKind};

/// Informations de contexte fournies par la racine de composition.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Version de l'application.
    pub version: &'static str,
    /// Hôte constaté au démarrage.
    pub host: HostInfo,
    /// Répertoire des données.
    pub data_dir: PathBuf,
    /// Répertoire de configuration.
    pub config_dir: PathBuf,
    /// Répertoire des journaux.
    pub log_dir: PathBuf,
}

/// Écrans disponibles. Seuls les écrans réellement implémentés figurent ici.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Accueil.
    Home,
    /// Diagnostic de l'hôte.
    Diagnostic,
}

impl Screen {
    /// Ordre de navigation.
    pub const ALL: [Self; 2] = [Self::Home, Self::Diagnostic];

    fn offset(self, delta: isize) -> Self {
        let len = Self::ALL.len() as isize;
        let index = Self::ALL.iter().position(|s| *s == self).unwrap_or(0) as isize;
        Self::ALL[(index + delta).rem_euclid(len) as usize]
    }

    /// Identifiant du titre localisé.
    pub fn title_id(self) -> &'static str {
        match self {
            Self::Home => msg::SCREEN_HOME,
            Self::Diagnostic => msg::SCREEN_DIAGNOSTIC,
        }
    }
}

/// Intention produite par un événement ; seule [`App::update`] modifie l'état.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Commande clavier.
    Command(Command),
    /// Redessiner (redimensionnement, retour au premier plan…).
    Redraw,
}

/// État de l'interface.
#[derive(Debug)]
pub struct App {
    running: bool,
    screen: Screen,
    help_open: bool,
    theme_kind: ThemeKind,
    diagnostic_row: usize,
    env: Environment,
    keymap: Keymap,
    i18n: Localizer,
}

impl App {
    /// Crée l'état initial.
    pub fn new(env: Environment) -> Self {
        Self {
            running: true,
            screen: Screen::Home,
            help_open: false,
            theme_kind: ThemeKind::default(),
            diagnostic_row: 0,
            env,
            keymap: Keymap::default(),
            i18n: Localizer::new(Language::default()),
        }
    }

    /// Indique si la boucle d'événements doit continuer.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Écran affiché.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// Indique si l'aide est affichée.
    pub fn help_open(&self) -> bool {
        self.help_open
    }

    /// Ligne sélectionnée dans le diagnostic.
    pub fn diagnostic_row(&self) -> usize {
        self.diagnostic_row
    }

    /// Contexte d'exécution.
    pub fn env(&self) -> &Environment {
        &self.env
    }

    /// Raccourcis clavier.
    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    /// Traducteur.
    pub fn i18n(&self) -> &Localizer {
        &self.i18n
    }

    /// Thème effectif, adapté aux capacités du terminal.
    pub fn theme(&self) -> Theme {
        Theme::resolve(self.theme_kind, self.env.host.terminal.color)
    }

    /// Indique si le changement de thème a un effet (terminal en couleurs).
    pub fn can_change_theme(&self) -> bool {
        self.env.host.terminal.color.has_colors()
    }

    /// Traduit un événement terminal en action. Ne modifie pas l'état.
    pub fn handle_event(&self, event: &Event) -> Option<Action> {
        match event {
            Event::Key(key) => self.keymap.command_for(key).map(Action::Command),
            Event::Resize(..) | Event::FocusGained => Some(Action::Redraw),
            _ => None,
        }
    }

    /// Applique une action à l'état.
    pub fn update(&mut self, action: Action) {
        let Action::Command(command) = action else {
            return;
        };
        match command {
            Command::Quit => self.running = false,
            Command::ToggleHelp => self.help_open = !self.help_open,
            Command::Close => self.help_open = false,
            Command::NextScreen => self.switch_screen(1),
            Command::PrevScreen => self.switch_screen(-1),
            Command::CycleTheme => {
                if self.can_change_theme() {
                    self.theme_kind = self.theme_kind.next();
                    tracing::debug!(theme = ?self.theme_kind, "thème modifié");
                }
            }
            Command::Up if self.screen == Screen::Diagnostic => {
                self.diagnostic_row = self.diagnostic_row.saturating_sub(1);
            }
            Command::Down if self.screen == Screen::Diagnostic => {
                self.diagnostic_row = (self.diagnostic_row + 1).min(diagnostic::ROW_COUNT - 1);
            }
            Command::Up | Command::Down => {}
        }
    }

    fn switch_screen(&mut self, delta: isize) {
        self.help_open = false;
        self.screen = self.screen.offset(delta);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use tssrlabs_domain::host::{ColorSupport, DetectionSource, TerminalInfo};

    pub(crate) fn test_env(color: ColorSupport) -> Environment {
        Environment {
            version: "9.9.9",
            host: HostInfo {
                os: "linux".into(),
                arch: "x86_64".into(),
                terminal: TerminalInfo {
                    term: Some("xterm-256color".into()),
                    program: Some("WezTerm".into()),
                    color,
                    color_source: DetectionSource::Detected,
                },
            },
            data_dir: "/home/apprenant/.local/share/tssrlabs".into(),
            config_dir: "/home/apprenant/.config/tssrlabs".into(),
            log_dir: "/home/apprenant/.local/share/tssrlabs/logs".into(),
        }
    }

    fn app() -> App {
        App::new(test_env(ColorSupport::TrueColor))
    }

    fn press(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn send(app: &mut App, code: KeyCode) {
        if let Some(action) = app.handle_event(&press(code)) {
            app.update(action);
        }
    }

    #[test]
    fn q_quits() {
        let mut app = app();
        send(&mut app, KeyCode::Char('q'));
        assert!(!app.is_running());
    }

    #[test]
    fn tab_cycles_screens_both_ways() {
        let mut app = app();
        send(&mut app, KeyCode::Tab);
        assert_eq!(app.screen(), Screen::Diagnostic);
        send(&mut app, KeyCode::Tab);
        assert_eq!(app.screen(), Screen::Home);
        send(&mut app, KeyCode::BackTab);
        assert_eq!(app.screen(), Screen::Diagnostic);
    }

    #[test]
    fn help_toggles_and_closes_with_escape() {
        let mut app = app();
        send(&mut app, KeyCode::F(1));
        assert!(app.help_open());
        send(&mut app, KeyCode::Esc);
        assert!(!app.help_open());
        send(&mut app, KeyCode::Char('?'));
        send(&mut app, KeyCode::Char('?'));
        assert!(!app.help_open());
        // Échap sans fenêtre ouverte ne quitte pas.
        send(&mut app, KeyCode::Esc);
        assert!(app.is_running());
    }

    #[test]
    fn theme_cycles_only_when_colors_are_available() {
        let mut app = app();
        send(&mut app, KeyCode::Char('t'));
        assert_eq!(app.theme().kind, ThemeKind::HighContrast);

        let mut mono = App::new(test_env(ColorSupport::None));
        assert!(!mono.can_change_theme());
        send(&mut mono, KeyCode::Char('t'));
        assert_eq!(mono.theme().kind, ThemeKind::Monochrome);
    }

    #[test]
    fn diagnostic_selection_is_clamped() {
        let mut app = app();
        send(&mut app, KeyCode::Down); // sans effet hors diagnostic
        assert_eq!(app.diagnostic_row(), 0);
        send(&mut app, KeyCode::Tab);
        send(&mut app, KeyCode::Up);
        assert_eq!(app.diagnostic_row(), 0);
        for _ in 0..100 {
            send(&mut app, KeyCode::Char('j'));
        }
        assert_eq!(app.diagnostic_row(), diagnostic::ROW_COUNT - 1);
    }

    #[test]
    fn resize_requests_redraw() {
        assert_eq!(
            app().handle_event(&Event::Resize(80, 24)),
            Some(Action::Redraw)
        );
    }
}

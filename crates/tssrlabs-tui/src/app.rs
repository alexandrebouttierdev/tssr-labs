//! État de l'interface et traduction des événements en actions.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Intention produite par un événement ; seule [`App::update`] modifie l'état.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quitter l'application proprement.
    Quit,
    /// Redessiner (redimensionnement, retour au premier plan…).
    Redraw,
}

/// État de l'interface.
#[derive(Debug, Clone)]
pub struct App {
    running: bool,
    version: &'static str,
}

impl App {
    /// Crée l'état initial.
    pub fn new(version: &'static str) -> Self {
        Self {
            running: true,
            version,
        }
    }

    /// Indique si la boucle d'événements doit continuer.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Version affichée de l'application.
    pub fn version(&self) -> &'static str {
        self.version
    }

    /// Traduit un événement terminal en action. Ne modifie pas l'état.
    pub fn handle_event(&self, event: &Event) -> Option<Action> {
        match event {
            Event::Key(key) => Self::handle_key(key),
            Event::Resize(..) | Event::FocusGained => Some(Action::Redraw),
            _ => None,
        }
    }

    fn handle_key(key: &KeyEvent) -> Option<Action> {
        // Sous Windows, crossterm signale aussi les relâchements de touche.
        if key.kind != KeyEventKind::Press {
            return None;
        }
        match key.code {
            KeyCode::Char('q' | 'Q') | KeyCode::Esc => Some(Action::Quit),
            // En mode brut, Ctrl+C n'émet pas SIGINT : on le traite ici.
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }

    /// Applique une action à l'état.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::Redraw => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventState;

    fn key(code: KeyCode, modifiers: KeyModifiers, kind: KeyEventKind) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind,
            state: KeyEventState::NONE,
        })
    }

    fn press(code: KeyCode) -> Event {
        key(code, KeyModifiers::NONE, KeyEventKind::Press)
    }

    #[test]
    fn quit_keys_produce_quit_action() {
        let app = App::new("0.0.0");
        for event in [
            press(KeyCode::Char('q')),
            press(KeyCode::Char('Q')),
            press(KeyCode::Esc),
            key(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                KeyEventKind::Press,
            ),
        ] {
            assert_eq!(app.handle_event(&event), Some(Action::Quit), "{event:?}");
        }
    }

    #[test]
    fn key_release_and_other_keys_are_ignored() {
        let app = App::new("0.0.0");
        assert_eq!(
            app.handle_event(&key(
                KeyCode::Char('q'),
                KeyModifiers::NONE,
                KeyEventKind::Release
            )),
            None
        );
        assert_eq!(app.handle_event(&press(KeyCode::Char('c'))), None);
        assert_eq!(app.handle_event(&press(KeyCode::Enter)), None);
    }

    #[test]
    fn resize_requests_redraw() {
        let app = App::new("0.0.0");
        assert_eq!(
            app.handle_event(&Event::Resize(80, 24)),
            Some(Action::Redraw)
        );
    }

    #[test]
    fn quit_action_stops_the_app() {
        let mut app = App::new("0.0.0");
        app.update(Action::Redraw);
        assert!(app.is_running());
        app.update(Action::Quit);
        assert!(!app.is_running());
    }
}

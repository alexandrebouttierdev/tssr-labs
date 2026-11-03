//! Rendu de l'état de l'interface : choix du mode d'affichage, cadre commun
//! et écran courant. Le rendu est pur : il ne modifie pas l'état.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Padding};

use crate::app::{App, Screen};
use crate::layout::LayoutMode;
use crate::{screens, widgets};

/// Dessine l'écran courant.
pub fn render(app: &App, frame: &mut Frame) {
    let theme = app.theme();
    let area = frame.area();
    frame.render_widget(Block::new().style(theme.base), area);

    let mode = LayoutMode::for_size(area.width, area.height);
    if mode == LayoutMode::TooSmall {
        widgets::too_small::render(app, &theme, frame, area);
        return;
    }

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);
    widgets::chrome::header(app, &theme, mode, frame, header);
    widgets::chrome::footer(app, &theme, mode, frame, footer);

    let content = if mode == LayoutMode::Full {
        let title = app.i18n().t(app.screen().title_id());
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(theme.border)
            .title(Line::styled(format!(" {title} "), theme.title))
            .padding(Padding::new(2, 2, 1, 0));
        let inner = block.inner(body);
        frame.render_widget(block, body);
        inner
    } else {
        // Compact : pas de cadre, une marge minimale.
        body.inner(Margin::new(1, 0))
    };

    match app.screen() {
        Screen::Home => screens::home::render(app, &theme, frame, content),
        Screen::Diagnostic => screens::diagnostic::render(app, &theme, frame, content, area),
    }

    if app.help_open() {
        widgets::help::render(app, &theme, frame, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Action;
    use crate::app::tests::test_env;
    use crate::keymap::Command;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use tssrlabs_domain::host::ColorSupport;

    fn rendered(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(app, frame)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    fn app() -> App {
        App::new(test_env(ColorSupport::TrueColor))
    }

    #[test]
    fn full_layout_shows_tabs_theme_and_shortcuts() {
        let text = rendered(&app(), 100, 30);
        for expected in [
            "TSSR Labs",
            "Accueil",
            "Diagnostic",
            "Thème : Sombre",
            "Quitter",
            "Version 9.9.9",
        ] {
            assert!(text.contains(expected), "« {expected} » absent :\n{text}");
        }
    }

    #[test]
    fn compact_layout_keeps_essential_actions() {
        let text = rendered(&app(), 70, 20);
        for expected in ["compact", "Aide", "Quitter", "Accueil"] {
            assert!(text.contains(expected), "« {expected} » absent :\n{text}");
        }
        assert!(!text.contains("Thème : "), "{text}");
    }

    #[test]
    fn too_small_terminal_explains_instead_of_truncating() {
        let text = rendered(&app(), 50, 12);
        assert!(text.contains("Terminal trop petit"), "{text}");
        assert!(text.contains("50×12"), "{text}");
        assert!(text.contains("60×16"), "{text}");
    }

    #[test]
    fn diagnostic_screen_lists_host_information() {
        let mut app = app();
        app.update(Action::Command(Command::NextScreen));
        let text = rendered(&app, 100, 30);
        for expected in ["Système", "linux (x86_64)", "24 bits", "100×30", "Journaux"] {
            assert!(text.contains(expected), "« {expected} » absent :\n{text}");
        }
    }

    #[test]
    fn help_overlay_lists_shortcuts() {
        let mut app = app();
        app.update(Action::Command(Command::ToggleHelp));
        let text = rendered(&app, 100, 30);
        for expected in ["Raccourcis clavier", "Ctrl+C", "Échap", "AZERTY"] {
            assert!(text.contains(expected), "« {expected} » absent :\n{text}");
        }
    }

    #[test]
    fn theme_shortcut_is_hidden_without_colors() {
        let app = App::new(test_env(ColorSupport::None));
        let text = rendered(&app, 100, 30);
        assert!(
            !text.contains("t Thème"),
            "raccourci de thème affiché :\n{text}"
        );
        assert!(text.contains("Thème : Monochrome"), "{text}");
    }

    #[test]
    fn every_screen_and_size_renders_without_panic() {
        let mut app = app();
        for _ in 0..2 {
            for help in [false, true] {
                if help != app.help_open() {
                    app.update(Action::Command(Command::ToggleHelp));
                }
                for (w, h) in [
                    (1, 1),
                    (10, 3),
                    (59, 15),
                    (60, 16),
                    (80, 24),
                    (100, 30),
                    (250, 80),
                ] {
                    let _ = rendered(&app, w, h);
                }
            }
            app.update(Action::Command(Command::NextScreen));
        }
    }
}

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

    let notice_height = u16::from(app.notice().is_some());
    let [header, body, notice, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(notice_height),
        Constraint::Length(1),
    ])
    .areas(area);
    widgets::chrome::header(app, &theme, mode, frame, header);
    widgets::chrome::notice(app, &theme, frame, notice);
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
        Screen::Paths => screens::paths::render(app, &theme, mode, frame, content),
        Screen::Library => screens::library::render(app, &theme, mode, frame, content),
        Screen::Reader => screens::reader::render(app, &theme, frame, content),
        Screen::Profiles => screens::profiles::render(app, &theme, mode, frame, content),
        Screen::Diagnostic => screens::diagnostic::render(app, &theme, frame, content, area),
    }

    if app.help_open() {
        widgets::help::render(app, &theme, frame, area);
    } else {
        widgets::dialog::render(app, &theme, frame, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Screen;
    use crate::app::tests::{go_to, loaded_app, test_catalog, test_env};
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
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

    fn send(app: &mut App, code: KeyCode) {
        app.handle_terminal_event(&Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }));
    }

    fn assert_contains(text: &str, expected: &[&str]) {
        for e in expected {
            assert!(text.contains(e), "« {e} » absent :\n{text}");
        }
    }

    #[test]
    fn loading_state_is_explicit() {
        let app = App::new(test_env(ColorSupport::TrueColor), test_catalog());
        assert_contains(&rendered(&app, 100, 30), &["Chargement des profils"]);
    }

    #[test]
    fn dashboard_greets_active_profile() {
        let app = loaded_app(&["Alice"], Some(0));
        assert_contains(
            &rendered(&app, 100, 30),
            &[
                "Accueil",
                "Profils",
                "Diagnostic",
                "Bonjour, Alice",
                "Aucune activité",
                "Profil : Alice",
                "Quitter",
            ],
        );
    }

    #[test]
    fn first_launch_shows_creation_dialog() {
        let app = loaded_app(&[], None);
        let text = rendered(&app, 100, 30);
        assert_contains(
            &text,
            &[
                "Nouveau profil",
                "Nom du profil",
                "Entrée : valider",
                "aucun profil actif",
            ],
        );
    }

    #[test]
    fn profiles_screen_lists_profiles_with_textual_state() {
        let mut app = loaded_app(&["Alice", "Bob"], Some(0));
        go_to(&mut app, Screen::Profiles);
        let text = rendered(&app, 100, 30);
        assert_contains(
            &text,
            &[
                "Nom",
                "État",
                "Alice",
                "actif",
                "Bob",
                "jamais",
                "Créé le",
                "Supprimer",
                "Profils locaux",
            ],
        );
        // Compact : colonnes et raccourcis secondaires masqués.
        let compact = rendered(&app, 70, 20);
        assert_contains(&compact, &["Alice", "Nouveau"]);
        assert!(!compact.contains("Créé le"), "{compact}");
    }

    #[test]
    fn footer_always_keeps_help_and_quit() {
        let mut app = loaded_app(&["Alice"], Some(0));
        go_to(&mut app, Screen::Profiles);
        for width in [60, 80, 100] {
            let text = rendered(&app, width, 30);
            let footer = text.lines().last().unwrap_or_default();
            assert!(
                footer.contains("Aide") && footer.contains("Quitter"),
                "{width} : {footer}"
            );
        }
    }

    #[test]
    fn delete_dialog_warns_and_mentions_backup() {
        let mut app = loaded_app(&["Alice"], Some(0));
        go_to(&mut app, Screen::Profiles);
        send(&mut app, KeyCode::Char('s'));
        assert_contains(
            &rendered(&app, 100, 30),
            &[
                "Supprimer « Alice »",
                "copie de sécurité",
                "Saisissez le nom",
            ],
        );
    }

    #[test]
    fn dialog_error_is_labelled_not_only_colored() {
        let mut app = loaded_app(&[], None);
        send(&mut app, KeyCode::Enter);
        assert_contains(
            &rendered(&app, 100, 30),
            &["Erreur : le nom ne peut pas être vide."],
        );
    }

    #[test]
    fn too_small_terminal_explains_instead_of_truncating() {
        let app = loaded_app(&["Alice"], Some(0));
        assert_contains(
            &rendered(&app, 50, 12),
            &["Terminal trop petit", "50×12", "60×16"],
        );
    }

    #[test]
    fn diagnostic_screen_lists_host_information() {
        let mut app = loaded_app(&["Alice"], Some(0));
        send(&mut app, KeyCode::BackTab);
        assert_contains(
            &rendered(&app, 100, 30),
            &[
                "Système",
                "linux (x86_64)",
                "24 bits",
                "100×30",
                "Base de données",
            ],
        );
    }

    #[test]
    fn help_overlay_lists_shortcuts_of_current_screen() {
        let mut app = loaded_app(&["Alice"], Some(0));
        send(&mut app, KeyCode::F(1));
        let home = rendered(&app, 100, 30);
        assert_contains(&home, &["Raccourcis clavier", "Ctrl+C", "Échap", "AZERTY"]);
        assert!(!home.contains("Suppr"), "{home}");

        send(&mut app, KeyCode::Esc);
        go_to(&mut app, Screen::Profiles);
        send(&mut app, KeyCode::F(1));
        assert_contains(&rendered(&app, 100, 30), &["s, Suppr", "Nouveau"]);

        send(&mut app, KeyCode::Esc);
        go_to(&mut app, Screen::Library);
        send(&mut app, KeyCode::F(1));
        assert_contains(
            &rendered(&app, 100, 30),
            &["Rechercher", "Domaine", "Effacer les filtres"],
        );
    }

    #[test]
    fn paths_library_and_reader_render_real_content() {
        let mut app = loaded_app(&["Alice"], Some(0));
        go_to(&mut app, Screen::Paths);
        assert_contains(
            &rendered(&app, 100, 30),
            &[
                "Linux — les bases",
                "2 chapitres",
                "60 min",
                "1. Le terminal",
                "2. Les permissions",
                "Aucun prérequis",
            ],
        );
        go_to(&mut app, Screen::Library);
        let library = rendered(&app, 100, 30);
        assert_contains(
            &library,
            &[
                "Recherche",
                "/ pour saisir",
                "Domaine : tous",
                "2 chapitres",
                "Intermédiaire",
                "Entrée Ouvrir",
            ],
        );
        send(&mut app, KeyCode::Enter);
        let reader = rendered(&app, 100, 30);
        assert_contains(
            &reader,
            &[
                "Le terminal",
                "Linux · Débutant · environ 20 min",
                "Objectifs",
                "Position : 0 %",
                "Échap Retour",
            ],
        );
        // Fin du chapitre : sources et statut de vérification.
        send(&mut app, KeyCode::End);
        let end = rendered(&app, 100, 30);
        assert_contains(
            &end,
            &[
                "Sources",
                "non vérifiée",
                "Licence : MIT",
                "Position : 100 %",
            ],
        );
    }

    #[test]
    fn theme_shortcut_is_hidden_without_colors() {
        let app = App::new(test_env(ColorSupport::None), test_catalog());
        let text = rendered(&app, 100, 30);
        assert!(!text.contains("t Thème"), "{text}");
        assert!(!text.contains("Thème : Sombre"), "{text}");
    }

    #[test]
    fn every_state_and_size_renders_without_panic() {
        let mut apps = vec![
            App::new(test_env(ColorSupport::TrueColor), test_catalog()),
            loaded_app(&[], None),
            loaded_app(&["Alice", "Bob avec un nom long"], Some(1)),
        ];
        let mut app = loaded_app(&["Alice"], Some(0));
        go_to(&mut app, Screen::Profiles);
        send(&mut app, KeyCode::Char('s'));
        apps.push(app);
        for app in &mut apps {
            for _ in 0..3 {
                for (w, h) in [
                    (1, 1),
                    (10, 3),
                    (59, 15),
                    (60, 16),
                    (80, 24),
                    (100, 30),
                    (250, 80),
                ] {
                    let _ = rendered(app, w, h);
                }
                send(app, KeyCode::Tab);
            }
        }
    }
}

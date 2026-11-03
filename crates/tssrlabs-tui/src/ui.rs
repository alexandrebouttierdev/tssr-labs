//! Rendu de l'état de l'interface.
//!
//! Écran provisoire de la tranche 0 : il présente honnêtement l'état du
//! produit. Le tableau de bord réel arrive avec les tranches suivantes.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};

use crate::app::App;

/// Dessine l'écran courant.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().dark_gray())
        .title(Line::from(" TSSR Labs ").bold());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [body, footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);
    frame.render_widget(welcome(app), centered_vertically(body, 6));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("q", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw(" / "),
            Span::styled("Échap", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw(" : quitter"),
        ]))
        .alignment(Alignment::Center)
        .dark_gray(),
        footer,
    );
}

fn welcome(app: &App) -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from("TSSR Labs").bold(),
        Line::from("Plateforme pédagogique systèmes et réseaux"),
        Line::from(""),
        Line::from(format!(
            "Version {} — socle technique (tranche 0)",
            app.version()
        ))
        .dark_gray(),
        Line::from(""),
        Line::from("Profils, cours et tableau de bord arrivent dans les prochaines tranches.")
            .dark_gray(),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
}

/// Centre verticalement un bloc de `height` lignes ; si la zone est trop
/// petite, renvoie la zone entière (le contenu est alors coupé par Ratatui,
/// jamais dessiné hors écran).
fn centered_vertically(area: Rect, height: u16) -> Rect {
    let [_, middle, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .areas(area);
    middle
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered_text(width: u16, height: u16) -> String {
        let app = App::new("9.9.9");
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| render(&app, frame)).unwrap();
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

    #[test]
    fn reference_size_shows_title_version_and_quit_hint() {
        let text = rendered_text(100, 30);
        assert!(text.contains("TSSR Labs"), "{text}");
        assert!(text.contains("Version 9.9.9"), "{text}");
        assert!(text.contains("quitter"), "{text}");
    }

    #[test]
    fn tiny_terminal_does_not_panic() {
        for (w, h) in [(1, 1), (10, 3), (40, 8)] {
            let _ = rendered_text(w, h);
        }
    }
}

//! Message affiché quand le terminal est trop petit pour un affichage sûr.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::i18n::msg;
use crate::keymap::Command;
use crate::layout::{MIN_HEIGHT, MIN_WIDTH};
use crate::theme::Theme;
use crate::widgets::centered;

/// Explique la taille minimale au lieu de tronquer l'interface.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let quit = app
        .keymap()
        .primary_label(Command::Quit, i18n)
        .unwrap_or_default();
    let lines = vec![
        Line::styled(i18n.t(msg::TOO_SMALL_TITLE), theme.title),
        Line::styled(
            i18n.t_args(
                msg::TOO_SMALL_SIZE,
                &[
                    ("width", area.width.into()),
                    ("height", area.height.into()),
                    ("min_width", MIN_WIDTH.into()),
                    ("min_height", MIN_HEIGHT.into()),
                ],
            ),
            theme.base,
        ),
        Line::styled(i18n.t(msg::TOO_SMALL_HINT), theme.muted),
        Line::styled(
            i18n.t_args(msg::TOO_SMALL_QUIT, &[("key", quit.into())]),
            theme.accent,
        ),
    ];
    // Centré verticalement ; quelques lignes de marge pour le retour à la ligne.
    let area = centered(area, area.width, 8);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

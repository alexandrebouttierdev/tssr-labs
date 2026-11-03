//! Écran d'accueil provisoire.
//!
//! Il présente honnêtement l'état du produit ; le tableau de bord réel
//! (niveau, compétences, activités) le remplacera avec les profils et la
//! progression.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::i18n::msg;
use crate::theme::Theme;

/// Dessine l'accueil.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let lines = vec![
        Line::styled(i18n.t(msg::APP_TITLE), theme.title),
        Line::styled(i18n.t(msg::APP_SUBTITLE), theme.base),
        Line::from(""),
        Line::styled(
            i18n.t_args(msg::HOME_VERSION, &[("version", app.env().version.into())]),
            theme.muted,
        ),
        Line::from(""),
        Line::styled(i18n.t(msg::HOME_UPCOMING), theme.muted),
    ];
    let height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let [_, middle, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height + 1),
        Constraint::Fill(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        middle,
    );
}

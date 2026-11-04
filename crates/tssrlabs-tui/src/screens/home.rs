//! Tableau de bord.
//!
//! Il n'affiche que des informations réelles : profil actif et absence
//! d'activité tant que les cours ne sont pas disponibles. Niveau,
//! compétences et recommandations arriveront avec la progression.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::i18n::msg;
use crate::theme::Theme;

/// Dessine le tableau de bord.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let version = Line::styled(
        i18n.t_args(msg::HOME_VERSION, &[("version", app.env().version.into())]),
        theme.muted,
    );
    let lines = match (app.profiles(), app.active_profile()) {
        (None, _) => vec![Line::styled(i18n.t(msg::LOADING_PROFILES), theme.muted)],
        (Some(_), Some(profile)) => vec![
            Line::styled(
                i18n.t_args(
                    msg::HOME_GREETING,
                    &[("name", profile.name.to_string().into())],
                ),
                theme.title,
            ),
            Line::styled(i18n.t(msg::APP_SUBTITLE), theme.muted),
            Line::from(""),
            Line::styled(i18n.t(msg::HOME_NO_ACTIVITY), theme.base),
            Line::styled(i18n.t(msg::HOME_UPCOMING), theme.muted),
            Line::from(""),
            version,
        ],
        (Some(_), None) => vec![
            Line::styled(i18n.t(msg::HOME_NO_PROFILE), theme.title),
            Line::styled(i18n.t(msg::HOME_NO_PROFILE_HINT), theme.base),
            Line::from(""),
            version,
        ],
    };
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

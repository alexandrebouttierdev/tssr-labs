//! Écran des profils : liste, activation, création, renommage, suppression.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{HighlightSpacing, Paragraph, Row, Table, TableState, Wrap};

use crate::app::App;
use crate::i18n::msg;
use crate::keymap::Command;
use crate::layout::LayoutMode;
use crate::text::format_local;
use crate::theme::Theme;

const HIGHLIGHT: &str = "› ";

/// Dessine l'écran des profils.
pub fn render(app: &App, theme: &Theme, mode: LayoutMode, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let Some(snapshot) = app.profiles() else {
        frame.render_widget(
            Paragraph::new(Line::styled(i18n.t(msg::LOADING_PROFILES), theme.muted)),
            area,
        );
        return;
    };

    let [list_area, note_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::styled(i18n.t(msg::PROFILES_LOCAL_NOTE), theme.muted)),
        note_area,
    );

    if snapshot.profiles.is_empty() {
        let key = app
            .keymap()
            .primary_label(Command::NewProfile, i18n)
            .unwrap_or_default();
        let lines = vec![
            Line::styled(i18n.t(msg::PROFILES_EMPTY), theme.title),
            Line::styled(
                i18n.t_args(msg::PROFILES_EMPTY_HINT, &[("key", key.into())]),
                theme.base,
            ),
        ];
        frame.render_widget(
            Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            list_area,
        );
        return;
    }

    let active_id = app.active_profile().map(|p| p.id);
    let active_label = i18n.t(msg::PROFILES_ACTIVE);
    let never = i18n.t(msg::PROFILES_NEVER);
    let full = mode == LayoutMode::Full;

    let rows = snapshot.profiles.iter().map(|profile| {
        // L'état actif est écrit en toutes lettres, pas seulement coloré.
        let state = if Some(profile.id) == active_id {
            Line::styled(active_label.clone(), theme.success)
        } else {
            Line::from("")
        };
        let last_used = profile
            .last_used_at
            .map_or_else(|| never.clone(), format_local);
        let mut cells = vec![
            Line::styled(profile.name.to_string(), theme.base),
            state,
            Line::styled(last_used, theme.muted),
        ];
        if full {
            cells.push(Line::styled(format_local(profile.created_at), theme.muted));
        }
        Row::new(cells)
    });

    let mut header = vec![
        i18n.t(msg::PROFILES_COL_NAME),
        i18n.t(msg::PROFILES_COL_STATE),
        i18n.t(msg::PROFILES_COL_LAST_USED),
    ];
    let mut widths = vec![
        Constraint::Fill(1),
        Constraint::Length(8),
        Constraint::Length(21),
    ];
    if full {
        header.push(i18n.t(msg::PROFILES_COL_CREATED));
        widths.push(Constraint::Length(17));
    }

    let table = Table::new(rows, widths)
        .header(Row::new(header).style(theme.title).bottom_margin(1))
        .row_highlight_style(theme.selection)
        .highlight_symbol(HIGHLIGHT)
        .highlight_spacing(HighlightSpacing::Always)
        .column_spacing(2);
    let mut state = TableState::default().with_selected(Some(app.profile_row()));
    frame.render_stateful_widget(table, list_area, &mut state);
}

//! Fenêtre d'aide : liste des raccourcis.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Row, Table, Wrap};

use crate::app::App;
use crate::i18n::msg;
use crate::keymap::Command;
use crate::theme::Theme;
use crate::widgets::centered;

/// Dessine l'aide par-dessus l'écran courant.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let rows: Vec<Row> = Command::ALL
        .into_iter()
        .filter(|c| *c != Command::CycleTheme || app.can_change_theme())
        .map(|command| {
            let keys: Vec<String> = app
                .keymap()
                .bindings_for(command)
                .map(|b| b.label(i18n))
                .collect();
            Row::new(vec![
                Line::styled(keys.join(", "), theme.accent),
                Line::styled(i18n.t(command.label_id()), theme.base),
            ])
        })
        .collect();
    let row_count = u16::try_from(rows.len()).unwrap_or(u16::MAX);

    let close = app
        .keymap()
        .primary_label(Command::Close, i18n)
        .unwrap_or_default();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.accent)
        .style(theme.base)
        .padding(Padding::horizontal(1))
        .title(Line::styled(
            format!(" {} ", i18n.t(msg::HELP_TITLE)),
            theme.title,
        ))
        .title_bottom(Line::styled(
            format!(
                " {} ",
                i18n.t_args(msg::HELP_CLOSE, &[("key", close.into())])
            ),
            theme.muted,
        ));

    // Lignes + séparation + note (2 lignes) + bordures.
    let popup = centered(area, 64, row_count + 5);
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);

    let [table_area, note_area] =
        Layout::vertical([Constraint::Length(row_count + 1), Constraint::Min(0)]).areas(inner);
    frame.render_widget(
        Table::new(rows, [Constraint::Length(18), Constraint::Fill(1)]),
        table_area,
    );
    frame.render_widget(
        Paragraph::new(Line::styled(i18n.t(msg::HELP_LAYOUT_NOTE), theme.muted))
            .wrap(Wrap { trim: true }),
        note_area,
    );
}

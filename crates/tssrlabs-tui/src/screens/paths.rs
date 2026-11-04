//! Parcours guidés : chapitres dans l'ordre recommandé.
//!
//! Le suivi de progression et le déblocage arriveront avec la progression
//! (XP, compétences) ; l'écran présente pour l'instant l'ordre et les
//! prérequis.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{HighlightSpacing, Paragraph, Row, Table, TableState, Wrap};

use crate::app::App;
use crate::i18n::msg;
use crate::layout::LayoutMode;
use crate::screens::difficulty_label_id;
use crate::theme::Theme;

const HIGHLIGHT: &str = "› ";

/// Dessine les parcours.
pub fn render(app: &App, theme: &Theme, mode: LayoutMode, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let catalog = app.catalog();
    if catalog.paths().is_empty() {
        frame.render_widget(
            Paragraph::new(Line::styled(i18n.t(msg::PATHS_EMPTY), theme.muted)),
            area,
        );
        return;
    }

    let details_height = if mode == LayoutMode::Full { 5 } else { 0 };
    let [list_area, details_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(details_height)]).areas(area);

    let mut rows = Vec::new();
    let mut entry_rows = Vec::new();
    for path in catalog.paths() {
        let chapters: Vec<_> = path
            .chapters
            .iter()
            .filter_map(|id| catalog.chapter(id))
            .collect();
        let minutes: u32 = chapters
            .iter()
            .map(|c| u32::from(c.estimated_minutes))
            .sum();
        let summary = i18n.t_args(msg::PATHS_SUMMARY, &[("count", chapters.len().into())]);
        let total = i18n.t_args(msg::MINUTES, &[("minutes", minutes.into())]);
        if !rows.is_empty() {
            rows.push(Row::new(vec![Line::from("")]));
        }
        rows.push(Row::new(vec![
            Line::styled(path.title.clone(), theme.title),
            Line::styled(summary, theme.muted),
            Line::styled(total, theme.muted),
        ]));
        for (index, chapter) in chapters.iter().enumerate() {
            entry_rows.push(rows.len());
            rows.push(Row::new(vec![
                Line::from(vec![
                    Span::styled(format!("  {}. ", index + 1), theme.accent),
                    Span::styled(chapter.title.clone(), theme.base),
                ]),
                Line::styled(i18n.t(difficulty_label_id(chapter.difficulty)), theme.muted),
                Line::styled(
                    i18n.t_args(
                        msg::MINUTES,
                        &[("minutes", chapter.estimated_minutes.into())],
                    ),
                    theme.muted,
                ),
            ]));
        }
    }

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Length(14),
            Constraint::Length(8),
        ],
    )
    .row_highlight_style(theme.selection)
    .highlight_symbol(HIGHLIGHT)
    .highlight_spacing(HighlightSpacing::Always)
    .column_spacing(2);
    let selected = entry_rows.get(app.paths_row()).copied();
    let mut state = TableState::default().with_selected(selected);
    frame.render_stateful_widget(table, list_area, &mut state);

    if details_height == 0 {
        return;
    }
    let entries = app.path_entries();
    let Some((_, chapter)) = entries.get(app.paths_row()) else {
        return;
    };
    let prerequisites: Vec<String> = chapter
        .prerequisites
        .iter()
        .filter_map(|id| catalog.chapter(id))
        .map(|c| c.title.clone())
        .collect();
    let prerequisites = if prerequisites.is_empty() {
        i18n.t(msg::PATHS_NO_PREREQUISITE)
    } else {
        i18n.t_args(
            msg::PATHS_PREREQUISITES,
            &[("list", prerequisites.join(", ").into())],
        )
    };
    let details = vec![
        Line::styled("─".repeat(usize::from(details_area.width)), theme.border),
        Line::styled(chapter.summary.clone(), theme.base),
        Line::styled(prerequisites, theme.muted),
    ];
    frame.render_widget(
        Paragraph::new(details).wrap(Wrap { trim: true }),
        details_area,
    );
}

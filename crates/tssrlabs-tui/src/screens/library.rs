//! Bibliothèque : exploration libre avec recherche et filtres.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{HighlightSpacing, Paragraph, Row, Table, TableState};

use crate::app::App;
use crate::i18n::msg;
use crate::keymap::Command;
use crate::layout::LayoutMode;
use crate::screens::{difficulty_label_id, domain_label_id};
use crate::theme::Theme;

const HIGHLIGHT: &str = "› ";

/// Dessine la bibliothèque.
pub fn render(app: &App, theme: &Theme, mode: LayoutMode, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let library = app.library();
    let query = &library.query;
    let full = mode == LayoutMode::Full;

    let [search_area, filters_area, count_area, list_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .areas(area);

    // Ligne de recherche.
    let label = format!("{} ", i18n.t(msg::LIBRARY_SEARCH));
    let label_width = u16::try_from(label.chars().count()).unwrap_or(u16::MAX);
    let [label_area, input_area] =
        Layout::horizontal([Constraint::Length(label_width), Constraint::Min(0)])
            .areas(search_area);
    frame.render_widget(Paragraph::new(Line::styled(label, theme.title)), label_area);
    if library.searching || !query.text.is_empty() {
        let style = if library.searching {
            theme.selection
        } else {
            theme.base
        };
        library.input.render(
            frame,
            input_area,
            style,
            library.searching && !app.help_open(),
        );
    } else {
        let key = app
            .keymap()
            .primary_label(Command::FocusSearch, i18n)
            .unwrap_or_default();
        frame.render_widget(
            Paragraph::new(Line::styled(
                i18n.t_args(msg::LIBRARY_SEARCH_HINT, &[("key", key.into())]),
                theme.muted,
            )),
            input_area,
        );
    }

    // Filtres : valeur courante et touche associée, en texte.
    let all = i18n.t(msg::LIBRARY_ALL);
    let domain = query
        .domain
        .map_or_else(|| all.clone(), |d| i18n.t(domain_label_id(d)));
    let difficulty = query
        .difficulty
        .map_or_else(|| all.clone(), |d| i18n.t(difficulty_label_id(d)));
    let duration = query.max_minutes.map_or_else(
        || i18n.t(msg::LIBRARY_ANY_DURATION),
        |m| i18n.t_args(msg::LIBRARY_MAX_MINUTES, &[("minutes", m.into())]),
    );
    let filter = |id: &str, value: String, command: Command| -> Vec<Span<'static>> {
        let key = app
            .keymap()
            .primary_label(command, i18n)
            .unwrap_or_default();
        vec![
            Span::styled(format!("[{key}] "), theme.accent),
            Span::styled(i18n.t_args(id, &[("value", value.into())]), theme.base),
            Span::raw("   "),
        ]
    };
    let mut spans = filter(msg::LIBRARY_FILTER_DOMAIN, domain, Command::CycleDomain);
    spans.extend(filter(
        msg::LIBRARY_FILTER_DIFFICULTY,
        difficulty,
        Command::CycleDifficulty,
    ));
    spans.extend(filter(
        msg::LIBRARY_FILTER_DURATION,
        duration,
        Command::CycleDuration,
    ));
    frame.render_widget(Paragraph::new(Line::from(spans)), filters_area);

    let results = app.library_results();
    let count = i18n.t_args(msg::LIBRARY_RESULTS, &[("count", results.len().into())]);
    frame.render_widget(Paragraph::new(Line::styled(count, theme.muted)), count_area);

    if results.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::styled(i18n.t(msg::LIBRARY_NO_MATCH), theme.base)),
            list_area,
        );
        return;
    }

    let rows = results.iter().map(|chapter| {
        let minutes = i18n.t_args(
            msg::MINUTES,
            &[("minutes", chapter.estimated_minutes.into())],
        );
        let mut cells = vec![Line::styled(chapter.title.clone(), theme.base)];
        if full {
            cells.push(Line::styled(
                i18n.t(domain_label_id(chapter.domain)),
                theme.muted,
            ));
            cells.push(Line::styled(
                i18n.t(difficulty_label_id(chapter.difficulty)),
                theme.muted,
            ));
        }
        cells.push(Line::styled(minutes, theme.muted));
        Row::new(cells)
    });
    let mut header = vec![i18n.t(msg::LIBRARY_COL_TITLE)];
    let mut widths = vec![Constraint::Fill(1)];
    if full {
        header.extend([
            i18n.t(msg::LIBRARY_COL_DOMAIN),
            i18n.t(msg::LIBRARY_COL_LEVEL),
        ]);
        widths.extend([Constraint::Length(15), Constraint::Length(14)]);
    }
    header.push(i18n.t(msg::LIBRARY_COL_DURATION));
    widths.push(Constraint::Length(8));

    let table = Table::new(rows, widths)
        .header(Row::new(header).style(theme.title).bottom_margin(1))
        .row_highlight_style(theme.selection)
        .highlight_symbol(HIGHLIGHT)
        .highlight_spacing(HighlightSpacing::Always)
        .column_spacing(2);
    let selected = (!library.searching).then_some(library.row);
    let mut state = TableState::default().with_selected(selected);
    frame.render_stateful_widget(table, list_area, &mut state);
}

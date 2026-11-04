//! Lecture d'un chapitre.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use tssrlabs_domain::content::Chapter;

use crate::app::App;
use crate::i18n::msg;
use crate::markdown;
use crate::screens::{difficulty_label_id, domain_label_id};
use crate::text::sanitize;
use crate::theme::Theme;

/// Titre de section suivi d'une liste à puces (rien si la liste est vide).
fn section(lines: &mut Vec<Line<'static>>, theme: &Theme, title: &str, items: Vec<String>) {
    if items.is_empty() {
        return;
    }
    lines.push(Line::styled(title.to_owned(), theme.accent));
    for item in items {
        lines.push(Line::from(vec![
            Span::styled("• ", theme.accent),
            Span::styled(sanitize(&item), theme.base),
        ]));
    }
    lines.push(Line::from(""));
}

/// Lignes complètes d'un chapitre : en-tête, objectifs, corps, sources.
pub fn chapter_lines(app: &App, theme: &Theme, chapter: &Chapter) -> Vec<Line<'static>> {
    let i18n = app.i18n();
    let catalog = app.catalog();
    let mut lines = vec![
        Line::styled(sanitize(&chapter.title), theme.title),
        Line::styled(
            i18n.t_args(
                msg::READER_META,
                &[
                    ("domain", i18n.t(domain_label_id(chapter.domain)).into()),
                    (
                        "difficulty",
                        i18n.t(difficulty_label_id(chapter.difficulty)).into(),
                    ),
                    ("minutes", chapter.estimated_minutes.into()),
                    ("version", sanitize(&chapter.version).into()),
                ],
            ),
            theme.muted,
        ),
        Line::from(""),
        Line::styled(sanitize(&chapter.summary), theme.base),
        Line::from(""),
    ];

    section(
        &mut lines,
        theme,
        &i18n.t(msg::READER_OBJECTIVES),
        chapter.objectives.clone(),
    );
    section(
        &mut lines,
        theme,
        &i18n.t(msg::READER_PREREQUISITES),
        chapter
            .prerequisites
            .iter()
            .filter_map(|id| catalog.chapter(id))
            .map(|c| c.title.clone())
            .collect(),
    );
    section(
        &mut lines,
        theme,
        &i18n.t(msg::READER_SKILLS),
        chapter
            .skills
            .iter()
            .filter_map(|id| catalog.skill(id))
            .map(|s| s.name.clone())
            .collect(),
    );

    lines.push(Line::styled("─".repeat(40), theme.border));
    lines.push(Line::from(""));
    // Le titre est déjà affiché en tête : on omet le titre principal du corps.
    let body = match chapter.body.strip_prefix("# ") {
        Some(rest) => rest.split_once('\n').map_or("", |(_, after)| after),
        None => chapter.body.as_str(),
    };
    lines.extend(markdown::render(body, theme));
    lines.push(Line::from(""));
    lines.push(Line::styled("─".repeat(40), theme.border));
    lines.push(Line::from(""));

    let sources = chapter
        .sources
        .iter()
        .map(|source| {
            let status = match source.verified_on {
                Some(date) => i18n.t_args(
                    msg::READER_VERIFIED,
                    &[("date", date.strftime("%d/%m/%Y").to_string().into())],
                ),
                None => i18n.t(msg::READER_UNVERIFIED),
            };
            format!("{} — {} ({status})", source.title, source.url)
        })
        .collect();
    section(&mut lines, theme, &i18n.t(msg::READER_SOURCES), sources);
    lines.push(Line::styled(
        i18n.t_args(
            msg::READER_CREDITS,
            &[
                ("authors", sanitize(&chapter.authors.join(", ")).into()),
                ("license", sanitize(&chapter.license).into()),
            ],
        ),
        theme.muted,
    ));
    lines
}

/// Dessine la lecture en cours. Met à jour les bornes de défilement
/// (qui dépendent de la largeur disponible).
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let Some(reader) = app.reader() else {
        return;
    };
    let Some(chapter) = app.catalog().chapter(&reader.chapter) else {
        frame.render_widget(
            Paragraph::new(Line::styled(i18n.t(msg::READER_MISSING), theme.danger)),
            area,
        );
        return;
    };

    let [text_area, position_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    let [text_area, _, scrollbar_area] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(text_area);

    let paragraph = Paragraph::new(chapter_lines(app, theme, chapter)).wrap(Wrap { trim: false });
    let total = u16::try_from(paragraph.line_count(text_area.width)).unwrap_or(u16::MAX);
    let max_scroll = total.saturating_sub(text_area.height);
    reader.max_scroll.set(max_scroll);
    reader.page.set(text_area.height);
    let scroll = reader.scroll.min(max_scroll);

    frame.render_widget(paragraph.scroll((scroll, 0)), text_area);

    let mut scrollbar = ScrollbarState::new(usize::from(max_scroll))
        .position(usize::from(scroll))
        .viewport_content_length(usize::from(text_area.height));
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .style(theme.muted),
        scrollbar_area,
        &mut scrollbar,
    );

    let percent = if max_scroll == 0 {
        100
    } else {
        u32::from(scroll) * 100 / u32::from(max_scroll)
    };
    frame.render_widget(
        Paragraph::new(Line::styled(
            i18n.t_args(msg::READER_POSITION, &[("percent", percent.into())]),
            theme.muted,
        ))
        .alignment(Alignment::Right),
        position_area,
    );
}

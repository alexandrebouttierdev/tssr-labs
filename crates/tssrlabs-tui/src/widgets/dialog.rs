//! Fenêtres de dialogue : saisie d'un nom, confirmation de suppression.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};

use crate::app::{App, Modal, NamePurpose};
use crate::i18n::msg;
use crate::theme::Theme;
use crate::widgets::centered;
use crate::widgets::input::TextInput;

/// Dessine la fenêtre de dialogue ouverte, s'il y en a une.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let (title, intro, input, error, danger) = match app.modal() {
        Some(Modal::Name {
            purpose,
            input,
            error,
        }) => {
            let title = match purpose {
                NamePurpose::Create => i18n.t(msg::DIALOG_CREATE_TITLE),
                NamePurpose::Rename(profile) => i18n.t_args(
                    msg::DIALOG_RENAME_TITLE,
                    &[("name", profile.name.to_string().into())],
                ),
            };
            let label = i18n.t_args(
                msg::DIALOG_NAME_LABEL,
                &[("max", tssrlabs_domain::profile::MAX_NAME_CHARS.into())],
            );
            (
                title,
                vec![Line::styled(label, theme.base)],
                input,
                error,
                false,
            )
        }
        Some(Modal::ConfirmDelete {
            profile,
            input,
            error,
        }) => (
            i18n.t_args(
                msg::DIALOG_DELETE_TITLE,
                &[("name", profile.name.to_string().into())],
            ),
            vec![
                Line::styled(i18n.t(msg::DIALOG_DELETE_WARNING), theme.warning),
                Line::styled(i18n.t(msg::DIALOG_DELETE_BACKUP), theme.muted),
                Line::from(""),
                Line::styled(i18n.t(msg::DIALOG_DELETE_PROMPT), theme.base),
            ],
            input,
            error,
            true,
        ),
        _ => return,
    };
    draw(
        app,
        theme,
        frame,
        area,
        &DialogContent {
            title,
            intro,
            input,
            error: error.as_deref(),
            danger,
        },
    );
}

struct DialogContent<'a> {
    title: String,
    intro: Vec<Line<'static>>,
    input: &'a TextInput,
    error: Option<&'a str>,
    danger: bool,
}

fn draw(app: &App, theme: &Theme, frame: &mut Frame, area: Rect, content: &DialogContent<'_>) {
    let i18n = app.i18n();
    let width = area.width.saturating_sub(4).min(64);
    let inner_width = width.saturating_sub(4).max(1);
    // Hauteur du texte d'introduction une fois replié.
    let intro_height: u16 = content
        .intro
        .iter()
        .map(|line| {
            let w = u16::try_from(line.width()).unwrap_or(u16::MAX);
            w.div_ceil(inner_width).max(1)
        })
        .sum();
    // Bordures (2) + intro + champ (1) + ligne d'état (2) + espacement (1).
    let height = intro_height + 6;
    let popup = centered(area, width, height);

    let hint = i18n.t_args(
        msg::DIALOG_HINT,
        &[
            ("confirm", i18n.t(msg::KEY_ENTER).into()),
            ("cancel", i18n.t(msg::KEY_ESC).into()),
        ],
    );
    let border = if content.danger {
        theme.danger
    } else {
        theme.accent
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(border)
        .style(theme.base)
        .padding(Padding::horizontal(1))
        .title(Line::styled(format!(" {} ", content.title), theme.title))
        .title_bottom(Line::styled(format!(" {hint} "), theme.muted));
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);

    let [intro_area, _, input_area, status_area] = Layout::vertical([
        Constraint::Length(intro_height),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);
    frame.render_widget(
        Paragraph::new(content.intro.clone()).wrap(Wrap { trim: true }),
        intro_area,
    );
    content
        .input
        .render(frame, input_area, theme.selection, !app.is_busy());

    let status = if app.is_busy() {
        Some(Line::styled(i18n.t(msg::DIALOG_BUSY), theme.muted))
    } else {
        content.error.map(|error| {
            Line::styled(
                i18n.t_args(msg::NOTICE_ERROR, &[("text", error.to_owned().into())]),
                theme.danger,
            )
        })
    };
    if let Some(status) = status {
        frame.render_widget(
            Paragraph::new(status).wrap(Wrap { trim: true }),
            status_area,
        );
    }
}

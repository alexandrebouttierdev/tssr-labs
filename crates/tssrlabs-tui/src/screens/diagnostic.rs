//! Écran de diagnostic : hôte, terminal et emplacements des données.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{HighlightSpacing, Paragraph, Row, Table, TableState};
use tssrlabs_domain::host::{ColorSupport, DetectionSource};
use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::i18n::msg;
use crate::layout::LayoutMode;
use crate::text::ellipsize_middle;
use crate::theme::{Theme, theme_label_id};

/// Nombre de lignes du diagnostic.
pub const ROW_COUNT: usize = 12;

const HIGHLIGHT: &str = "› ";

/// Lignes (libellé, valeur) du diagnostic pour une taille de terminal.
pub fn rows(app: &App, terminal_size: (u16, u16)) -> [(String, String); ROW_COUNT] {
    let i18n = app.i18n();
    let env = app.env();
    let terminal = &env.host.terminal;
    let unknown = || i18n.t(msg::VALUE_UNKNOWN);

    let level = i18n.t(match terminal.color {
        ColorSupport::None => msg::COLORS_NONE,
        ColorSupport::Basic => msg::COLORS_BASIC,
        ColorSupport::Ansi256 => msg::COLORS_256,
        ColorSupport::TrueColor => msg::COLORS_TRUECOLOR,
    });
    let colors = match terminal.color_source {
        DetectionSource::Detected => level,
        DetectionSource::UserOverride => {
            i18n.t_args(msg::COLORS_FORCED, &[("level", level.into())])
        }
    };
    let (width, height) = terminal_size;
    let mode = i18n.t(match LayoutMode::for_size(width, height) {
        LayoutMode::Full => msg::LAYOUT_FULL,
        _ => msg::LAYOUT_COMPACT,
    });

    [
        (msg::DIAG_VERSION, env.version.to_owned()),
        (
            msg::DIAG_SYSTEM,
            format!("{} ({})", env.host.os, env.host.arch),
        ),
        (
            msg::DIAG_TERMINAL,
            terminal.program.clone().unwrap_or_else(unknown),
        ),
        (
            msg::DIAG_TERM,
            terminal.term.clone().unwrap_or_else(unknown),
        ),
        (msg::DIAG_COLORS, colors),
        (
            msg::DIAG_SIZE,
            i18n.t_args(
                msg::SIZE_VALUE,
                &[
                    ("width", width.into()),
                    ("height", height.into()),
                    ("mode", mode.into()),
                ],
            ),
        ),
        (msg::DIAG_THEME, i18n.t(theme_label_id(app.theme().kind))),
        (msg::DIAG_DATA, env.data_dir.display().to_string()),
        (msg::DIAG_CONFIG, env.config_dir.display().to_string()),
        (msg::DIAG_LOGS, env.log_dir.display().to_string()),
        (msg::DIAG_DATABASE, env.database_file.display().to_string()),
        (
            msg::DIAG_CONTENT,
            i18n.t_args(
                msg::DIAG_CONTENT_VALUE,
                &[
                    ("paths", app.catalog().paths().len().into()),
                    ("chapters", app.catalog().chapters().len().into()),
                    ("format", env.content_format.into()),
                    ("warnings", app.catalog().warnings().len().into()),
                ],
            ),
        ),
    ]
    .map(|(label, value)| (i18n.t(label), value))
}

/// Dessine le diagnostic dans `area` ; `terminal` est la taille totale.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect, terminal: Rect) {
    let rows = rows(app, (terminal.width, terminal.height));
    let label_width = rows.iter().map(|(l, _)| l.width()).max().unwrap_or(0) + 2;
    let value_width = usize::from(area.width).saturating_sub(label_width + HIGHLIGHT.width() + 1);

    let table_rows = rows.iter().map(|(label, value)| {
        Row::new(vec![
            Line::styled(label.clone(), theme.muted),
            Line::styled(ellipsize_middle(value, value_width), theme.base),
        ])
    });
    let table = Table::new(
        table_rows,
        [
            Constraint::Length(u16::try_from(label_width).unwrap_or(u16::MAX)),
            Constraint::Fill(1),
        ],
    )
    .row_highlight_style(theme.selection)
    .highlight_symbol(HIGHLIGHT)
    .highlight_spacing(HighlightSpacing::Always);

    let show_note = usize::from(area.height) > ROW_COUNT + 1;
    let [table_area, note_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(if show_note { 1 } else { 0 }),
    ])
    .areas(area);

    let mut state = TableState::default().with_selected(Some(app.diagnostic_row()));
    frame.render_stateful_widget(table, table_area, &mut state);
    if show_note {
        frame.render_widget(
            Paragraph::new(Line::styled(app.i18n().t(msg::DIAG_NOTE), theme.muted)),
            note_area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::{test_catalog, test_env};

    #[test]
    fn rows_describe_host_and_paths() {
        let app = App::new(test_env(ColorSupport::Ansi256), test_catalog());
        let rows = rows(&app, (120, 40));
        let text: Vec<String> = rows.iter().map(|(l, v)| format!("{l}={v}")).collect();
        assert!(
            text.contains(&"Système=linux (x86_64)".to_owned()),
            "{text:?}"
        );
        assert!(
            text.contains(&"Couleurs=256 couleurs".to_owned()),
            "{text:?}"
        );
        assert!(
            text.contains(&"Taille=120×40 — affichage complet".to_owned()),
            "{text:?}"
        );
        assert!(
            text.iter()
                .any(|r| r.starts_with("Journaux=/home/apprenant")),
            "{text:?}"
        );
    }
}

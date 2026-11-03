//! En-tête (titre, onglets, état) et pied de page (raccourcis visibles).

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::{App, Screen};
use crate::i18n::msg;
use crate::keymap::Command;
use crate::layout::LayoutMode;
use crate::theme::Theme;

/// Barre d'en-tête : nom de l'application, écrans, état d'affichage.
pub fn header(app: &App, theme: &Theme, mode: LayoutMode, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    frame.render_widget(Block::new().style(theme.bar), area);

    let mut spans = vec![Span::styled(
        format!(" {} ", i18n.t(msg::APP_TITLE)),
        theme.accent,
    )];
    for screen in Screen::ALL {
        let style = if screen == app.screen() {
            theme.tab_active
        } else {
            theme.muted
        };
        // Espaces hors du style : le soulignement ne couvre que le nom.
        spans.push(Span::raw("   "));
        spans.push(Span::styled(i18n.t(screen.title_id()), style));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);

    let status = match mode {
        LayoutMode::Full => i18n.t_args(
            msg::STATUS_THEME,
            &[("name", i18n.t(theme.kind.label_id()).into())],
        ),
        _ => i18n.t(msg::STATUS_COMPACT),
    };
    frame.render_widget(
        Paragraph::new(Line::styled(format!("{status} "), theme.muted)).alignment(Alignment::Right),
        area,
    );
}

/// Pied de page : raccourcis disponibles dans le contexte courant.
pub fn footer(app: &App, theme: &Theme, mode: LayoutMode, frame: &mut Frame, area: Rect) {
    frame.render_widget(Block::new().style(theme.bar), area);
    let mut commands = vec![Command::NextScreen];
    if mode == LayoutMode::Full {
        if app.screen() == Screen::Diagnostic {
            commands.extend([Command::Up, Command::Down]);
        }
        if app.can_change_theme() {
            commands.push(Command::CycleTheme);
        }
    }
    if app.help_open() {
        commands.push(Command::Close);
    }
    commands.extend([Command::ToggleHelp, Command::Quit]);
    frame.render_widget(Paragraph::new(hints(app, theme, &commands)), area);
}

/// Ligne « touche libellé » pour une liste de commandes.
pub fn hints(app: &App, theme: &Theme, commands: &[Command]) -> Line<'static> {
    let i18n = app.i18n();
    let mut spans = vec![Span::raw(" ")];
    for command in commands {
        let Some(key) = app.keymap().primary_label(*command, i18n) else {
            continue;
        };
        spans.push(Span::styled(key, theme.accent));
        spans.push(Span::styled(
            format!(" {}   ", i18n.t(command.label_id())),
            theme.muted,
        ));
    }
    Line::from(spans)
}

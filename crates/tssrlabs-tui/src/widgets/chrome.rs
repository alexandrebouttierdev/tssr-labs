//! En-tête (titre, onglets, état), ligne de message et pied de page
//! (raccourcis visibles).

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::{App, Notice, Screen};
use crate::i18n::msg;
use crate::keymap::Command;
use crate::layout::LayoutMode;
use crate::text::ellipsize_middle;
use crate::theme::{Theme, theme_label_id};

/// Barre d'en-tête : nom de l'application, écrans, profil et thème.
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
    let tabs = Line::from(spans);
    let tabs_width = tabs.width();
    frame.render_widget(Paragraph::new(tabs), area);

    let profile = match app.active_profile() {
        Some(profile) => i18n.t_args(
            msg::STATUS_PROFILE,
            &[("name", profile.name.to_string().into())],
        ),
        None => i18n.t(msg::STATUS_NO_PROFILE),
    };
    let status = if mode == LayoutMode::Full {
        let theme_name = i18n.t_args(
            msg::STATUS_THEME,
            &[("name", i18n.t(theme_label_id(theme.kind)).into())],
        );
        format!("{profile} · {theme_name} ")
    } else {
        format!("{profile} ")
    };
    // Le statut ne doit jamais recouvrir les onglets.
    let room = usize::from(area.width).saturating_sub(tabs_width + 2);
    frame.render_widget(
        Paragraph::new(Line::styled(ellipsize_middle(&status, room), theme.muted))
            .alignment(Alignment::Right),
        area,
    );
}

/// Ligne de message (information ou erreur), avec préfixe textuel.
pub fn notice(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let (text, style) = match app.notice() {
        Some(Notice::Info(text)) => (
            i18n.t_args(msg::NOTICE_INFO, &[("text", text.clone().into())]),
            theme.success,
        ),
        Some(Notice::Error(text)) => (
            i18n.t_args(msg::NOTICE_ERROR, &[("text", text.clone().into())]),
            theme.danger,
        ),
        None => return,
    };
    let text = ellipsize_middle(&text, usize::from(area.width).saturating_sub(2));
    frame.render_widget(
        Paragraph::new(Line::styled(format!(" {text}"), style)),
        area,
    );
}

/// Pied de page : raccourcis disponibles dans le contexte courant.
pub fn footer(app: &App, theme: &Theme, mode: LayoutMode, frame: &mut Frame, area: Rect) {
    frame.render_widget(Block::new().style(theme.bar), area);
    let i18n = app.i18n();

    // Fenêtre de dialogue : seules Entrée et Échap s'appliquent.
    if app.modal().is_some() && !app.help_open() {
        let hint = i18n.t_args(
            msg::DIALOG_HINT,
            &[
                ("confirm", i18n.t(msg::KEY_ENTER).into()),
                ("cancel", i18n.t(msg::KEY_ESC).into()),
            ],
        );
        frame.render_widget(
            Paragraph::new(Line::styled(format!(" {hint}"), theme.muted)),
            area,
        );
        return;
    }

    let full = mode == LayoutMode::Full;
    let mut commands = vec![Command::NextScreen];
    match app.screen() {
        Screen::Profiles if app.profiles().is_some() && !app.is_busy() => {
            let has_profiles = app.profiles().is_some_and(|s| !s.profiles.is_empty());
            if has_profiles {
                commands.push(Command::Activate);
            }
            commands.push(Command::NewProfile);
            if has_profiles && full {
                commands.extend([Command::RenameProfile, Command::DeleteProfile]);
            }
        }
        Screen::Diagnostic if full => commands.extend([Command::Up, Command::Down]),
        _ => {}
    }
    if full && app.can_change_theme() {
        commands.push(Command::CycleTheme);
    }
    if app.help_open() {
        commands.push(Command::Close);
    }
    commands.extend([Command::ToggleHelp, Command::Quit]);

    // Si la ligne déborde, on retire d'abord les raccourcis secondaires les
    // plus à droite ; l'aide et « Quitter » restent toujours visibles.
    let essential = [Command::ToggleHelp, Command::Quit, Command::Close];
    let mut line = hints(app, theme, &commands);
    while line.width() > usize::from(area.width) {
        let Some(index) = commands.iter().rposition(|c| !essential.contains(c)) else {
            break;
        };
        commands.remove(index);
        line = hints(app, theme, &commands);
    }
    frame.render_widget(Paragraph::new(line), area);
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
            format!(" {}  ", i18n.t(command.label_id())),
            theme.muted,
        ));
    }
    Line::from(spans)
}

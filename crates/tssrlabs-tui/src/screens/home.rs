//! Tableau de bord : niveau, série, recommandation, compétences et
//! activité récente du profil actif. Uniquement des informations réelles,
//! calculées à partir des tentatives enregistrées.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;
use crate::i18n::msg;
use crate::keymap::Command;
use crate::layout::LayoutMode;
use crate::screens::progression::{
    attempt_xp, exercise_title, level_lines, outcome_label_id, skill_line, streak_line,
};
use crate::text::{format_local, sanitize};
use crate::theme::Theme;

/// Nombre de tentatives affichées dans « Activité récente ».
const RECENT: usize = 4;

/// Dessine le tableau de bord.
pub fn render(app: &App, theme: &Theme, mode: LayoutMode, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let Some(profile) = app.active_profile() else {
        let lines = match app.profiles() {
            None => vec![Line::styled(i18n.t(msg::LOADING_PROFILES), theme.muted)],
            Some(_) => vec![
                Line::styled(i18n.t(msg::HOME_NO_PROFILE), theme.title),
                Line::styled(i18n.t(msg::HOME_NO_PROFILE_HINT), theme.base),
            ],
        };
        frame.render_widget(
            Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            area,
        );
        return;
    };

    let report = app.report();
    let mut left = vec![
        Line::styled(
            i18n.t_args(
                msg::HOME_GREETING,
                &[("name", profile.name.to_string().into())],
            ),
            theme.title,
        ),
        Line::from(""),
    ];
    left.extend(level_lines(app, theme, 10));
    left.push(Line::styled(
        i18n.t_args(
            msg::HOME_EXERCISES,
            &[
                ("solved", report.solved_count().into()),
                ("total", app.catalog().exercises().len().into()),
            ],
        ),
        theme.base,
    ));
    left.push(streak_line(app, theme));
    left.push(Line::from(""));
    match report.recommended(app.catalog()) {
        Some(chapter) => {
            let key = app
                .keymap()
                .primary_label(Command::Activate, i18n)
                .unwrap_or_default();
            left.push(Line::styled(
                i18n.t_args(
                    msg::HOME_RECOMMENDED,
                    &[("title", sanitize(&chapter.title).into())],
                ),
                theme.accent,
            ));
            left.push(Line::styled(
                i18n.t_args(msg::HOME_RECOMMENDED_HINT, &[("key", key.into())]),
                theme.muted,
            ));
        }
        None => left.push(Line::styled(i18n.t(msg::HOME_ALL_DONE), theme.success)),
    }
    left.push(Line::from(""));
    left.push(Line::styled(i18n.t(msg::HOME_UPCOMING), theme.muted));

    let mut right = vec![Line::styled(i18n.t(msg::HOME_SKILLS), theme.accent)];
    // Largeur restant pour le nom : colonne moins barre, pourcentage et degré.
    let column = if mode == LayoutMode::Full {
        area.width * 55 / 100
    } else {
        area.width
    };
    let name_width = usize::from(column).saturating_sub(32).clamp(10, 40);
    right.extend(
        report
            .skills
            .iter()
            .map(|s| skill_line(app, theme, s, name_width, false)),
    );
    right.push(Line::from(""));
    right.push(Line::styled(i18n.t(msg::HOME_RECENT), theme.accent));
    let recent: Vec<_> = app.attempts().iter().rev().take(RECENT).collect();
    if recent.is_empty() {
        right.push(Line::styled(i18n.t(msg::HOME_NO_RECENT), theme.muted));
    }
    for attempt in recent {
        let xp = attempt_xp(app, attempt)
            .map(|xp| format!(" {}", i18n.t_args(msg::HISTORY_XP, &[("xp", xp.into())])))
            .unwrap_or_default();
        right.push(Line::from(vec![
            Span::styled(format!("{}  ", format_local(attempt.at)), theme.muted),
            Span::styled(exercise_title(app, attempt), theme.base),
            Span::styled(
                format!(" · {}", i18n.t(outcome_label_id(attempt.outcome))),
                theme.muted,
            ),
            Span::styled(xp, theme.accent),
        ]));
    }

    if mode == LayoutMode::Full {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .areas(area);
        frame.render_widget(Paragraph::new(left).wrap(Wrap { trim: true }), left_area);
        frame.render_widget(Paragraph::new(right).wrap(Wrap { trim: true }), right_area);
    } else {
        left.push(Line::from(""));
        left.extend(right);
        frame.render_widget(Paragraph::new(left).wrap(Wrap { trim: true }), area);
    }
}

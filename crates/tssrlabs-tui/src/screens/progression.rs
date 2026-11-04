//! Écran « Progression » et éléments partagés avec le tableau de bord :
//! niveau, maîtrise des compétences, série d'activité et historique.

use jiff::Zoned;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{HighlightSpacing, Paragraph, Row, Table, TableState, Wrap};
use tssrlabs_domain::exercise::Outcome;
use tssrlabs_domain::progression::{AttemptRecord, Mastery, SkillMastery, streak};

use crate::app::App;
use crate::i18n::msg;
use crate::text::{bar, format_local, sanitize};
use crate::theme::Theme;

const HIGHLIGHT: &str = "› ";

/// Libellé d'un degré de maîtrise.
pub fn mastery_label_id(mastery: Mastery) -> &'static str {
    match mastery {
        Mastery::NotStarted => msg::MASTERY_NOT_STARTED,
        Mastery::Discovering => msg::MASTERY_DISCOVERING,
        Mastery::Progressing => msg::MASTERY_PROGRESSING,
        Mastery::Mastered => msg::MASTERY_MASTERED,
    }
}

/// Libellé d'une issue de tentative.
pub fn outcome_label_id(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Correct => msg::OUTCOME_CORRECT,
        Outcome::Incorrect => msg::OUTCOME_INCORRECT,
        Outcome::Revealed => msg::OUTCOME_REVEALED,
    }
}

/// Ligne « Niveau 2 · ███░░ 40 / 200 XP vers le niveau 3 ».
pub fn level_lines(app: &App, theme: &Theme, bar_width: usize) -> Vec<Line<'static>> {
    let i18n = app.i18n();
    let report = app.report();
    let level = report.level();
    let percent = (level.xp_into * 100)
        .checked_div(level.xp_span)
        .unwrap_or(100);
    vec![
        Line::styled(
            i18n.t_args(
                msg::HOME_LEVEL,
                &[
                    ("level", level.number.into()),
                    ("xp", report.total_xp.into()),
                ],
            ),
            theme.title,
        ),
        Line::from(vec![
            Span::styled(bar(percent, bar_width), theme.accent),
            Span::raw(" "),
            Span::styled(
                i18n.t_args(
                    msg::LEVEL_BAR,
                    &[
                        ("into", level.xp_into.into()),
                        ("span", level.xp_span.into()),
                        ("next", (level.number + 1).into()),
                    ],
                ),
                theme.muted,
            ),
        ]),
    ]
}

/// Ligne de maîtrise d'une compétence : nom, barre, pourcentage et degré.
pub fn skill_line(
    app: &App,
    theme: &Theme,
    skill: &SkillMastery,
    name_width: usize,
    detailed: bool,
) -> Line<'static> {
    let i18n = app.i18n();
    let name = app
        .catalog()
        .skill(&skill.skill)
        .map_or_else(|| skill.skill.to_string(), |s| s.name.clone());
    let name = crate::text::ellipsize_end(&sanitize(&name), name_width);
    let padding = name_width.saturating_sub(unicode_width::UnicodeWidthStr::width(name.as_str()));
    let style = match skill.mastery() {
        Mastery::Mastered => theme.success,
        Mastery::NotStarted => theme.muted,
        _ => theme.accent,
    };
    Line::from(vec![
        Span::styled(format!("{name}{} ", " ".repeat(padding)), theme.base),
        Span::styled(bar(skill.percent(), if detailed { 10 } else { 8 }), style),
        Span::styled(
            if detailed {
                format!(
                    " {:>3} % · {} ({}/{})",
                    skill.percent(),
                    i18n.t(mastery_label_id(skill.mastery())),
                    skill.solved,
                    skill.total
                )
            } else {
                format!(
                    " {:>3} % · {}",
                    skill.percent(),
                    i18n.t(mastery_label_id(skill.mastery()))
                )
            },
            theme.muted,
        ),
    ])
}

/// Série d'activité du profil actif, dans le fuseau du système.
pub fn current_streak(app: &App) -> u32 {
    let tz = TimeZone::system();
    let days: Vec<Date> = app
        .attempts()
        .iter()
        .map(|a| a.at.to_zoned(tz.clone()).date())
        .collect();
    streak(&days, Zoned::now().with_time_zone(tz).date())
}

/// Ligne de série d'activité.
pub fn streak_line(app: &App, theme: &Theme) -> Line<'static> {
    Line::styled(
        app.i18n()
            .t_args(msg::HOME_STREAK, &[("days", current_streak(app).into())]),
        theme.muted,
    )
}

/// XP rapporté par une tentative (celle de la première réussite).
pub fn attempt_xp(app: &App, attempt: &AttemptRecord) -> Option<u32> {
    let award = app.report().awards.get(&attempt.exercise)?;
    (attempt.outcome == Outcome::Correct && award.at == attempt.at).then(|| award.total())
}

/// Titre d'un exercice (ou son identifiant s'il n'existe plus).
pub fn exercise_title(app: &App, attempt: &AttemptRecord) -> String {
    app.catalog()
        .exercises()
        .iter()
        .find(|e| e.id == attempt.exercise)
        .map_or_else(|| attempt.exercise.to_string(), |e| sanitize(&e.title))
}

/// Dessine l'écran de progression.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    if app.active_profile().is_none() {
        frame.render_widget(
            Paragraph::new(Line::styled(
                i18n.t(msg::PROGRESSION_NO_PROFILE),
                theme.muted,
            ))
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    let report = app.report();
    let skills_height = u16::try_from(report.skills.len()).unwrap_or(u16::MAX) + 1;
    let [summary_area, skills_area, rules_area, history_area] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(skills_height + 1),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .areas(area);

    let mut summary = level_lines(app, theme, 20);
    summary.push(Line::styled(
        i18n.t_args(
            msg::PROGRESSION_TOTALS,
            &[
                ("xp", report.total_xp.into()),
                ("solved", report.solved_count().into()),
                ("total", app.catalog().exercises().len().into()),
            ],
        ),
        theme.base,
    ));
    summary.push(streak_line(app, theme));
    frame.render_widget(Paragraph::new(summary), summary_area);

    let name_width = report
        .skills
        .iter()
        .filter_map(|s| app.catalog().skill(&s.skill))
        .map(|s| unicode_width::UnicodeWidthStr::width(s.name.as_str()))
        .max()
        .unwrap_or(10)
        .min(usize::from(area.width / 2));
    let mut skills = vec![Line::styled(i18n.t(msg::HOME_SKILLS), theme.accent)];
    skills.extend(
        report
            .skills
            .iter()
            .map(|s| skill_line(app, theme, s, name_width, true)),
    );
    frame.render_widget(Paragraph::new(skills), skills_area);

    frame.render_widget(
        Paragraph::new(Line::styled(i18n.t(msg::PROGRESSION_RULES), theme.muted))
            .wrap(Wrap { trim: true }),
        rules_area,
    );

    // Historique, le plus récent en premier.
    let attempts: Vec<&AttemptRecord> = app.attempts().iter().rev().collect();
    let [title_area, table_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(history_area);
    frame.render_widget(
        Paragraph::new(Line::styled(
            i18n.t_args(
                msg::PROGRESSION_HISTORY,
                &[("count", attempts.len().into())],
            ),
            theme.accent,
        )),
        title_area,
    );
    if attempts.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::styled(i18n.t(msg::HOME_NO_RECENT), theme.muted)),
            table_area,
        );
        return;
    }
    let rows = attempts.iter().map(|attempt| {
        let xp = attempt_xp(app, attempt)
            .map(|xp| i18n.t_args(msg::HISTORY_XP, &[("xp", xp.into())]))
            .unwrap_or_default();
        let style = match attempt.outcome {
            Outcome::Correct => theme.success,
            Outcome::Incorrect => theme.danger,
            Outcome::Revealed => theme.warning,
        };
        Row::new(vec![
            Line::styled(format_local(attempt.at), theme.muted),
            Line::styled(exercise_title(app, attempt), theme.base),
            Line::styled(i18n.t(outcome_label_id(attempt.outcome)), style),
            Line::styled(xp, theme.accent),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Fill(1),
            Constraint::Length(18),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec![
            i18n.t(msg::PROGRESSION_COL_DATE),
            i18n.t(msg::PROGRESSION_COL_EXERCISE),
            i18n.t(msg::PROGRESSION_COL_RESULT),
            i18n.t(msg::PROGRESSION_COL_XP),
        ])
        .style(theme.title),
    )
    .row_highlight_style(theme.selection)
    .highlight_symbol(HIGHLIGHT)
    .highlight_spacing(HighlightSpacing::Always)
    .column_spacing(2);
    let mut state = TableState::default().with_selected(Some(app.progression_row()));
    frame.render_stateful_widget(table, table_area, &mut state);
}

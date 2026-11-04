//! Écran d'exercice guidé.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use tssrlabs_domain::exercise::{BLANK, Exercise, ExerciseKind};

use crate::app::App;
use crate::app::exercise_types::{AttemptStatus, ExerciseState};
use crate::i18n::msg;
use crate::keymap::Command;
use crate::markdown;
use crate::text::sanitize;
use crate::theme::Theme;

/// Empile des blocs de haut en bas ; ce qui dépasse la zone est coupé.
struct Flow<'f, 'a> {
    frame: &'f mut Frame<'a>,
    area: Rect,
    y: u16,
}

impl Flow<'_, '_> {
    fn remaining(&self) -> u16 {
        (self.area.y + self.area.height).saturating_sub(self.y)
    }

    fn next_rect(&mut self, height: u16) -> Option<Rect> {
        let height = height.min(self.remaining());
        if height == 0 {
            return None;
        }
        let rect = Rect::new(self.area.x, self.y, self.area.width, height);
        self.y += height;
        Some(rect)
    }

    fn lines(&mut self, lines: Vec<Line<'static>>) {
        if lines.is_empty() {
            return;
        }
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        let height = u16::try_from(paragraph.line_count(self.area.width)).unwrap_or(u16::MAX);
        if let Some(rect) = self.next_rect(height) {
            self.frame.render_widget(paragraph, rect);
        }
    }

    fn line(&mut self, line: Line<'static>) {
        self.lines(vec![line]);
    }

    fn gap(&mut self) {
        self.next_rect(1);
    }
}

fn kind_label_id(kind: &ExerciseKind) -> &'static str {
    match kind {
        ExerciseKind::MultipleChoice { .. } => msg::EXERCISE_KIND_QCM,
        ExerciseKind::CompleteCommand { .. } => msg::EXERCISE_KIND_COMPLETE,
        ExerciseKind::AnalyzeOutput { .. } => msg::EXERCISE_KIND_ANALYZE,
        ExerciseKind::WriteCommand { .. } => msg::EXERCISE_KIND_WRITE,
    }
}

fn code_line(text: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("  │ ", theme.muted),
        Span::styled(sanitize(text), theme.accent),
    ])
}

/// Dessine l'exercice en cours.
pub fn render(app: &App, theme: &Theme, frame: &mut Frame, area: Rect) {
    let i18n = app.i18n();
    let (Some((exercise, total)), Some(state)) = (app.current_exercise(), app.exercise_state())
    else {
        frame.render_widget(
            Paragraph::new(Line::styled(i18n.t(msg::EXERCISE_NONE), theme.muted)),
            area,
        );
        return;
    };
    let mut flow = Flow {
        frame,
        area,
        y: area.y,
    };

    flow.line(Line::styled(
        i18n.t_args(
            msg::EXERCISE_HEADER,
            &[
                ("index", (state.index + 1).into()),
                ("total", total.into()),
                ("kind", i18n.t(kind_label_id(&exercise.kind)).into()),
            ],
        ),
        theme.muted,
    ));
    flow.line(Line::styled(sanitize(&exercise.title), theme.title));
    flow.gap();
    flow.lines(markdown::render(&exercise.prompt, theme));
    flow.gap();

    answer_area(app, theme, &mut flow, exercise, state);
    feedback(app, theme, &mut flow, exercise, state, total);
}

fn answer_area(
    app: &App,
    theme: &Theme,
    flow: &mut Flow<'_, '_>,
    exercise: &Exercise,
    state: &ExerciseState,
) {
    let i18n = app.i18n();
    match &exercise.kind {
        ExerciseKind::MultipleChoice { choices, correct } => {
            let finished = state.status.is_finished();
            let lines = choices
                .iter()
                .enumerate()
                .map(|(index, choice)| {
                    let checked = state.chosen.contains(&index);
                    // État en texte : [x] coché, ✓ bonne réponse une fois terminé.
                    let mark = if checked { "[x]" } else { "[ ]" };
                    let good = if finished && correct.contains(&index) {
                        " ✓"
                    } else {
                        ""
                    };
                    let style = if index == state.cursor && !finished {
                        theme.selection
                    } else {
                        theme.base
                    };
                    let pointer = if index == state.cursor && !finished {
                        "› "
                    } else {
                        "  "
                    };
                    Line::from(vec![
                        Span::styled(pointer, theme.accent),
                        Span::styled(format!("{mark} {}. {}", index + 1, sanitize(choice)), style),
                        Span::styled(good, theme.success),
                    ])
                })
                .collect();
            flow.lines(lines);
            let count = if correct.len() > 1 {
                msg::EXERCISE_MULTIPLE
            } else {
                msg::EXERCISE_SINGLE
            };
            let keys = |command| {
                app.keymap()
                    .primary_label(command, i18n)
                    .unwrap_or_default()
            };
            flow.line(Line::styled(
                format!(
                    "{} {}",
                    i18n.t(count),
                    i18n.t_args(
                        msg::EXERCISE_CHOICE_HINT,
                        &[
                            ("toggle", keys(Command::ToggleChoice).into()),
                            ("validate", keys(Command::Activate).into()),
                        ],
                    )
                ),
                theme.muted,
            ));
        }
        kind => {
            match kind {
                ExerciseKind::AnalyzeOutput { output, .. } => {
                    flow.lines(output.lines().map(|l| code_line(l, theme)).collect());
                    flow.gap();
                }
                ExerciseKind::CompleteCommand { template, .. } => {
                    flow.line(code_line(&template.replace(BLANK, "____"), theme));
                    flow.gap();
                }
                _ => {}
            }
            input_row(app, theme, flow, state);
            flow.gap();
            flow.lines(vec![Line::styled(
                i18n.t(msg::EXERCISE_TEXT_CHECK),
                theme.muted,
            )]);
        }
    }
}

fn input_row(app: &App, theme: &Theme, flow: &mut Flow<'_, '_>, state: &ExerciseState) {
    let label = format!("{} ", app.i18n().t(msg::EXERCISE_ANSWER));
    let label_width = u16::try_from(label.chars().count()).unwrap_or(u16::MAX);
    let Some(row) = flow.next_rect(1) else {
        return;
    };
    let label_rect = Rect::new(row.x, row.y, label_width.min(row.width), 1);
    flow.frame
        .render_widget(Paragraph::new(Line::styled(label, theme.title)), label_rect);
    let input_rect = Rect::new(
        row.x + label_rect.width,
        row.y,
        row.width.saturating_sub(label_rect.width),
        1,
    );
    let finished = state.status.is_finished();
    let style = if finished {
        theme.base
    } else {
        theme.selection
    };
    state
        .input
        .render(flow.frame, input_rect, style, !finished && !app.help_open());
}

fn feedback(
    app: &App,
    theme: &Theme,
    flow: &mut Flow<'_, '_>,
    exercise: &Exercise,
    state: &ExerciseState,
    total: usize,
) {
    let i18n = app.i18n();
    let shown = usize::from(state.hints_shown);
    if shown > 0 {
        flow.gap();
        // Indices et explications sont rédigés en Markdown (code en ligne).
        let mut lines: Vec<Line<'static>> = exercise
            .hints
            .iter()
            .take(shown)
            .enumerate()
            .flat_map(|(i, hint)| {
                markdown::render(
                    &i18n.t_args(
                        msg::EXERCISE_HINT,
                        &[("index", (i + 1).into()), ("text", hint.clone().into())],
                    ),
                    theme,
                )
            })
            .collect();
        if shown >= exercise.hints.len() {
            lines.push(Line::styled(
                i18n.t(msg::EXERCISE_NO_MORE_HINT),
                theme.muted,
            ));
        }
        flow.lines(lines);
    }

    let status: Option<(String, Style)> = match state.status {
        AttemptStatus::Pending => None,
        AttemptStatus::Empty => Some((i18n.t(msg::EXERCISE_EMPTY), theme.warning)),
        AttemptStatus::Incorrect => Some((i18n.t(msg::EXERCISE_INCORRECT), theme.danger)),
        AttemptStatus::Correct => Some((i18n.t(msg::EXERCISE_CORRECT), theme.success)),
        AttemptStatus::Revealed => Some((
            i18n.t_args(
                msg::EXERCISE_REVEALED,
                &[("solution", sanitize(&exercise.solution()).into())],
            ),
            theme.warning,
        )),
    };
    if let Some((text, style)) = status {
        flow.gap();
        flow.lines(vec![Line::styled(text, style)]);
    }
    if state.status.is_finished() {
        flow.lines(markdown::render(
            &i18n.t_args(
                msg::EXERCISE_EXPLANATION,
                &[("text", exercise.explanation.clone().into())],
            ),
            theme,
        ));
        if state.index + 1 < total {
            let key = app
                .keymap()
                .primary_label(Command::Activate, i18n)
                .unwrap_or_default();
            flow.line(Line::styled(
                i18n.t_args(msg::EXERCISE_NEXT_HINT, &[("key", key.into())]),
                theme.accent,
            ));
        }
    }

    flow.gap();
    let progress = match app.active_profile() {
        Some(_) => {
            let progress = app
                .progress()
                .get(&exercise.id)
                .copied()
                .unwrap_or_default();
            i18n.t_args(
                msg::EXERCISE_PROGRESS,
                &[
                    ("attempts", progress.attempts.into()),
                    ("solved", if progress.solved { "yes" } else { "no" }.into()),
                ],
            )
        }
        None => i18n.t(msg::EXERCISE_NOT_RECORDED),
    };
    flow.line(Line::styled(progress, theme.muted));
}

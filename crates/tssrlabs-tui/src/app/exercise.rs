//! État et commandes de l'écran d'exercice.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tssrlabs_domain::content::ContentId;
use tssrlabs_domain::exercise::{Answer, Exercise, ExerciseKind, Outcome, Verdict};

use super::{App, Screen};
use crate::effects::Effect;
use crate::keymap::Command;
use crate::widgets::input::TextInput;

/// État de la réponse à l'exercice affiché.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStatus {
    /// Pas encore validé.
    Pending,
    /// Validation d'une réponse vide (rien n'est compté).
    Empty,
    /// Dernière réponse incorrecte.
    Incorrect,
    /// Réussi.
    Correct,
    /// Solution affichée.
    Revealed,
}

impl AttemptStatus {
    /// L'exercice est terminé (réussi ou solution affichée).
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Correct | Self::Revealed)
    }
}

/// État de l'écran d'exercice.
#[derive(Debug, Clone)]
pub struct ExerciseState {
    /// Chapitre dont on fait les exercices.
    pub chapter: ContentId,
    /// Position dans la liste des exercices du chapitre.
    pub index: usize,
    /// Proposition mise en évidence (QCM).
    pub cursor: usize,
    /// Propositions cochées (QCM), triées.
    pub chosen: Vec<usize>,
    /// Réponse saisie (exercices textuels).
    pub input: TextInput,
    /// Nombre d'indices affichés.
    pub hints_shown: u8,
    /// État de la réponse.
    pub status: AttemptStatus,
}

impl ExerciseState {
    fn new(chapter: ContentId, index: usize) -> Self {
        Self {
            chapter,
            index,
            cursor: 0,
            chosen: Vec::new(),
            input: TextInput::default(),
            hints_shown: 0,
            status: AttemptStatus::Pending,
        }
    }

    fn toggle(&mut self, choice: usize) {
        match self.chosen.binary_search(&choice) {
            Ok(position) => {
                self.chosen.remove(position);
            }
            Err(position) => self.chosen.insert(position, choice),
        }
    }
}

impl App {
    /// Exercice en cours.
    pub fn exercise_state(&self) -> Option<&ExerciseState> {
        self.exercise.as_ref()
    }

    /// Exercice affiché et nombre d'exercices du chapitre.
    pub fn current_exercise(&self) -> Option<(&Exercise, usize)> {
        let state = self.exercise.as_ref()?;
        let exercises = self.catalog.exercises_for(&state.chapter);
        exercises.get(state.index).map(|e| (*e, exercises.len()))
    }

    /// Ouvre le premier exercice du chapitre lu.
    pub(super) fn open_exercises(&mut self) -> bool {
        let Some(chapter) = self.reader.as_ref().map(|r| r.chapter.clone()) else {
            return false;
        };
        if self.catalog.exercises_for(&chapter).is_empty() {
            return false;
        }
        self.exercise = Some(ExerciseState::new(chapter, 0));
        self.screen = Screen::Exercise;
        true
    }

    /// Touche sur l'écran d'exercice, avant les raccourcis : la saisie
    /// d'une réponse textuelle a priorité (les lettres sont du texte).
    /// Renvoie `true` si la touche a été consommée.
    pub(super) fn on_exercise_key(&mut self, key: &KeyEvent) -> bool {
        let Some((exercise, _)) = self.current_exercise() else {
            return false;
        };
        let is_text = exercise.kind.is_text();
        let choices = match &exercise.kind {
            ExerciseKind::MultipleChoice { choices, .. } => choices.len(),
            _ => 0,
        };
        let Some(state) = self.exercise.as_mut() else {
            return false;
        };
        if state.status.is_finished() {
            return false;
        }
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        if is_text {
            let editing = matches!(
                key.code,
                KeyCode::Char(_)
                    | KeyCode::Backspace
                    | KeyCode::Delete
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Home
                    | KeyCode::End
            );
            if plain && editing && state.input.handle(key) {
                state.status = AttemptStatus::Pending;
                return true;
            }
            return false;
        }
        // QCM : un chiffre coche directement la proposition correspondante.
        if let KeyCode::Char(c) = key.code
            && plain
            && let Some(digit) = c.to_digit(10)
        {
            let index = digit as usize;
            if (1..=choices).contains(&index) {
                state.cursor = index - 1;
                state.toggle(index - 1);
                state.status = AttemptStatus::Pending;
            }
            return true;
        }
        false
    }

    /// Commandes de l'écran d'exercice ; renvoie `true` si traitée.
    pub(super) fn on_exercise_command(&mut self, command: Command) -> bool {
        let Some((exercise, total)) = self.current_exercise() else {
            return false;
        };
        let exercise = exercise.clone();
        let choices = match &exercise.kind {
            ExerciseKind::MultipleChoice { choices, .. } => choices.len(),
            _ => 0,
        };
        let Some(state) = self.exercise.as_mut() else {
            return false;
        };
        match command {
            Command::Activate if state.status.is_finished() => self.change_exercise(1, total),
            Command::Activate => self.submit(&exercise),
            Command::Up if choices > 0 => state.cursor = state.cursor.saturating_sub(1),
            Command::Down if choices > 0 => {
                state.cursor = (state.cursor + 1).min(choices.saturating_sub(1));
            }
            Command::ToggleChoice if choices > 0 && !state.status.is_finished() => {
                let cursor = state.cursor;
                state.toggle(cursor);
                state.status = AttemptStatus::Pending;
            }
            Command::Hint => {
                let available = u8::try_from(exercise.hints.len()).unwrap_or(u8::MAX);
                state.hints_shown = (state.hints_shown + 1).min(available);
            }
            Command::ShowSolution if !state.status.is_finished() => {
                state.status = AttemptStatus::Revealed;
                let hints = state.hints_shown;
                self.record(&exercise, Outcome::Revealed, hints);
            }
            Command::NextExercise => self.change_exercise(1, total),
            Command::PrevExercise => self.change_exercise(-1, total),
            Command::Close => {
                self.exercise = None;
                self.screen = Screen::Reader;
            }
            Command::NextScreen | Command::PrevScreen => {
                self.exercise = None;
                let return_to = self.reader.take().map_or(Screen::Library, |r| r.return_to);
                let delta = if command == Command::NextScreen {
                    1
                } else {
                    -1
                };
                self.screen = return_to.offset(delta);
            }
            _ => return false,
        }
        true
    }

    fn change_exercise(&mut self, delta: isize, total: usize) {
        let Some(state) = self.exercise.as_mut() else {
            return;
        };
        let target = state.index as isize + delta;
        if (0..total as isize).contains(&target) {
            *state = ExerciseState::new(state.chapter.clone(), target as usize);
        }
    }

    fn submit(&mut self, exercise: &Exercise) {
        let Some(state) = self.exercise.as_mut() else {
            return;
        };
        let answer = if exercise.kind.is_text() {
            Answer::Text(state.input.value())
        } else {
            Answer::Choices(state.chosen.clone())
        };
        let hints = state.hints_shown;
        let outcome = match exercise.check(&answer) {
            Verdict::Empty => {
                state.status = AttemptStatus::Empty;
                return;
            }
            Verdict::Correct => {
                state.status = AttemptStatus::Correct;
                Outcome::Correct
            }
            Verdict::Incorrect => {
                state.status = AttemptStatus::Incorrect;
                Outcome::Incorrect
            }
        };
        self.record(exercise, outcome, hints);
    }

    /// Enregistre la tentative pour le profil actif (sinon, rien n'est
    /// conservé et l'écran l'indique).
    fn record(&mut self, exercise: &Exercise, outcome: Outcome, hints_used: u8) {
        if let Some(profile) = self.active_profile().map(|p| p.id) {
            self.effects.push(Effect::RecordAttempt {
                profile,
                exercise: exercise.id.clone(),
                outcome,
                hints_used,
            });
        }
    }
}

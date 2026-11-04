//! Exercices guidés : types, vérification des réponses et progression.
//!
//! Les exercices « commande à compléter », « commande à rédiger » et
//! « analyse de sortie » sont vérifiés **par comparaison de texte** : aucune
//! commande n'est exécutée. L'interface doit l'indiquer clairement ; la
//! vérification par exécution réelle relève des laboratoires.

use std::collections::HashSet;

use crate::content::{ContentId, ContentIssue};

/// Marqueur de l'emplacement à compléter dans un modèle de commande.
pub const BLANK: &str = "___";

/// Nature d'un exercice et données nécessaires à sa vérification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExerciseKind {
    /// Question à choix multiples (une ou plusieurs bonnes réponses).
    MultipleChoice {
        /// Propositions.
        choices: Vec<String>,
        /// Indices des bonnes propositions (triés, sans doublon).
        correct: Vec<usize>,
    },
    /// Compléter l'emplacement [`BLANK`] d'une commande.
    CompleteCommand {
        /// Modèle contenant exactement un [`BLANK`].
        template: String,
        /// Valeurs acceptées pour l'emplacement.
        answers: Vec<String>,
    },
    /// Lire une sortie de terminal et répondre par un texte court.
    AnalyzeOutput {
        /// Sortie affichée.
        output: String,
        /// Réponses acceptées (casse ignorée).
        answers: Vec<String>,
    },
    /// Rédiger une commande complète.
    WriteCommand {
        /// Commandes acceptées.
        answers: Vec<String>,
    },
}

impl ExerciseKind {
    /// Identifiant stable du type (format de contenu).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MultipleChoice { .. } => "qcm",
            Self::CompleteCommand { .. } => "completer",
            Self::AnalyzeOutput { .. } => "analyser",
            Self::WriteCommand { .. } => "rediger",
        }
    }

    /// Indique si la réponse est un texte libre (vérification textuelle).
    pub fn is_text(&self) -> bool {
        !matches!(self, Self::MultipleChoice { .. })
    }
}

/// Exercice rattaché à un chapitre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exercise {
    /// Identifiant (unique dans tout le catalogue).
    pub id: ContentId,
    /// Chapitre de rattachement.
    pub chapter: ContentId,
    /// Titre court.
    pub title: String,
    /// Consigne.
    pub prompt: String,
    /// Type et données de vérification.
    pub kind: ExerciseKind,
    /// Indices, du plus général au plus précis.
    pub hints: Vec<String>,
    /// Explication affichée après réussite ou affichage de la solution.
    pub explanation: String,
}

/// Réponse d'un apprenant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// Propositions cochées (indices).
    Choices(Vec<usize>),
    /// Texte saisi.
    Text(String),
}

/// Résultat d'une vérification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Réponse correcte.
    Correct,
    /// Réponse incorrecte.
    Incorrect,
    /// Réponse vide : rien n'est compté.
    Empty,
}

/// Issue d'une tentative enregistrée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Réussite.
    Correct,
    /// Échec.
    Incorrect,
    /// Solution affichée.
    Revealed,
}

impl Outcome {
    /// Identifiant stable (stockage).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Correct => "correct",
            Self::Incorrect => "incorrect",
            Self::Revealed => "revealed",
        }
    }

    /// Lit un identifiant stocké.
    pub fn parse(value: &str) -> Option<Self> {
        [Self::Correct, Self::Incorrect, Self::Revealed]
            .into_iter()
            .find(|o| o.as_str() == value)
    }
}

/// Progression d'un profil sur un exercice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExerciseProgress {
    /// Nombre de tentatives (y compris l'affichage de la solution).
    pub attempts: u32,
    /// Réussi au moins une fois.
    pub solved: bool,
    /// Solution affichée au moins une fois.
    pub revealed: bool,
    /// Plus petit nombre d'indices utilisés lors d'une réussite.
    pub fewest_hints: Option<u8>,
}

impl ExerciseProgress {
    /// Met à jour la progression avec une nouvelle tentative.
    pub fn record(&mut self, outcome: Outcome, hints_used: u8) {
        self.attempts = self.attempts.saturating_add(1);
        match outcome {
            Outcome::Correct => {
                self.solved = true;
                self.fewest_hints =
                    Some(self.fewest_hints.map_or(hints_used, |h| h.min(hints_used)));
            }
            Outcome::Revealed => self.revealed = true,
            Outcome::Incorrect => {}
        }
    }
}

/// Normalise une saisie : espaces superflus retirés et réduits à un seul.
pub fn normalize_spaces(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalise une commande saisie : espaces, et invite `$ ` ou `# `
/// éventuellement recopiée en tête.
pub fn normalize_command(text: &str) -> String {
    let text = normalize_spaces(text);
    match text.strip_prefix("$ ").or_else(|| text.strip_prefix("# ")) {
        Some(rest) => rest.to_owned(),
        None => text,
    }
}

impl Exercise {
    /// Vérifie une réponse. Les commandes sont sensibles à la casse (comme
    /// sous Linux) ; les réponses d'analyse ne le sont pas.
    pub fn check(&self, answer: &Answer) -> Verdict {
        match (&self.kind, answer) {
            (ExerciseKind::MultipleChoice { correct, .. }, Answer::Choices(selected)) => {
                if selected.is_empty() {
                    return Verdict::Empty;
                }
                let mut selected = selected.clone();
                selected.sort_unstable();
                selected.dedup();
                verdict(selected == *correct)
            }
            (ExerciseKind::CompleteCommand { answers, .. }, Answer::Text(text))
            | (ExerciseKind::WriteCommand { answers }, Answer::Text(text)) => {
                let given = normalize_command(text);
                if given.is_empty() {
                    return Verdict::Empty;
                }
                verdict(answers.iter().any(|a| normalize_command(a) == given))
            }
            (ExerciseKind::AnalyzeOutput { answers, .. }, Answer::Text(text)) => {
                let given = normalize_spaces(text).to_lowercase();
                if given.is_empty() {
                    return Verdict::Empty;
                }
                verdict(
                    answers
                        .iter()
                        .any(|a| normalize_spaces(a).to_lowercase() == given),
                )
            }
            _ => Verdict::Incorrect,
        }
    }

    /// Solution affichable (première réponse acceptée, ou propositions
    /// correctes pour un QCM).
    pub fn solution(&self) -> String {
        match &self.kind {
            ExerciseKind::MultipleChoice { choices, correct } => correct
                .iter()
                .filter_map(|i| choices.get(*i))
                .cloned()
                .collect::<Vec<_>>()
                .join(" ; "),
            ExerciseKind::CompleteCommand { template, answers } => answers
                .first()
                .map(|a| template.replacen(BLANK, a, 1))
                .unwrap_or_default(),
            ExerciseKind::AnalyzeOutput { answers, .. }
            | ExerciseKind::WriteCommand { answers } => {
                answers.first().cloned().unwrap_or_default()
            }
        }
    }
}

fn verdict(correct: bool) -> Verdict {
    if correct {
        Verdict::Correct
    } else {
        Verdict::Incorrect
    }
}

fn has_control(text: &str, multiline: bool) -> bool {
    text.chars()
        .any(|c| c.is_control() && !(multiline && (c == '\n' || c == '\t')))
}

/// Valide des exercices : identifiants uniques (y compris face aux autres
/// identifiants du catalogue), chapitre existant, données cohérentes avec
/// le type et absence de caractères de contrôle.
pub fn validate(
    exercises: &[Exercise],
    chapter_ids: &HashSet<&ContentId>,
    other_ids: &HashSet<&ContentId>,
) -> Vec<ContentIssue> {
    let mut issues = Vec::new();
    let mut seen = HashSet::new();
    for exercise in exercises {
        let id = exercise.id.to_string();
        if !seen.insert(&exercise.id) || other_ids.contains(&exercise.id) {
            issues.push(ContentIssue::DuplicateId(id.clone()));
        }
        if !chapter_ids.contains(&exercise.chapter) {
            issues.push(ContentIssue::UnknownExerciseChapter {
                exercise: id.clone(),
                chapter: exercise.chapter.to_string(),
            });
        }
        let text =
            |issues: &mut Vec<ContentIssue>, field: &'static str, value: &str, multiline: bool| {
                if value.trim().is_empty() {
                    issues.push(ContentIssue::EmptyField {
                        id: id.clone(),
                        field,
                    });
                } else if has_control(value, multiline) {
                    issues.push(ContentIssue::ControlCharacter {
                        id: id.clone(),
                        field,
                    });
                }
            };
        let invalid = |issues: &mut Vec<ContentIssue>, reason: &'static str| {
            issues.push(ContentIssue::InvalidExercise {
                exercise: id.clone(),
                reason,
            });
        };
        text(&mut issues, "title", &exercise.title, false);
        text(&mut issues, "prompt", &exercise.prompt, true);
        text(&mut issues, "explanation", &exercise.explanation, true);
        for hint in &exercise.hints {
            text(&mut issues, "hints", hint, false);
        }
        match &exercise.kind {
            ExerciseKind::MultipleChoice { choices, correct } => {
                if choices.len() < 2 {
                    invalid(&mut issues, "au moins deux propositions sont nécessaires");
                }
                if correct.is_empty() || correct.iter().any(|i| *i >= choices.len()) {
                    invalid(&mut issues, "bonnes réponses absentes ou hors limites");
                }
                if correct.windows(2).any(|w| w[0] >= w[1]) {
                    invalid(&mut issues, "bonnes réponses non triées ou en double");
                }
                let unique: HashSet<&String> = choices.iter().collect();
                if unique.len() != choices.len() {
                    invalid(&mut issues, "propositions en double");
                }
                for choice in choices {
                    text(&mut issues, "choices", choice, false);
                }
            }
            ExerciseKind::CompleteCommand { template, answers } => {
                if template.matches(BLANK).count() != 1 {
                    invalid(&mut issues, "le modèle doit contenir exactement un ___");
                }
                text(&mut issues, "template", template, false);
                if answers.is_empty() {
                    invalid(&mut issues, "aucune réponse acceptée");
                }
                for answer in answers {
                    text(&mut issues, "answers", answer, false);
                }
            }
            ExerciseKind::AnalyzeOutput { output, answers } => {
                text(&mut issues, "output", output, true);
                if answers.is_empty() {
                    invalid(&mut issues, "aucune réponse acceptée");
                }
                for answer in answers {
                    text(&mut issues, "answers", answer, false);
                }
            }
            ExerciseKind::WriteCommand { answers } => {
                if answers.is_empty() {
                    invalid(&mut issues, "aucune réponse acceptée");
                }
                for answer in answers {
                    text(&mut issues, "answers", answer, false);
                }
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(v: &str) -> ContentId {
        ContentId::parse(v).unwrap()
    }

    fn exercise(kind: ExerciseKind) -> Exercise {
        Exercise {
            id: id("ex"),
            chapter: id("chapitre"),
            title: "Titre".into(),
            prompt: "Consigne".into(),
            kind,
            hints: vec!["Indice".into()],
            explanation: "Explication".into(),
        }
    }

    fn text(t: &str) -> Answer {
        Answer::Text(t.into())
    }

    #[test]
    fn multiple_choice_requires_exact_set() {
        let ex = exercise(ExerciseKind::MultipleChoice {
            choices: vec!["a".into(), "b".into(), "c".into()],
            correct: vec![0, 2],
        });
        assert_eq!(ex.check(&Answer::Choices(vec![2, 0])), Verdict::Correct);
        assert_eq!(ex.check(&Answer::Choices(vec![0])), Verdict::Incorrect);
        assert_eq!(
            ex.check(&Answer::Choices(vec![0, 1, 2])),
            Verdict::Incorrect
        );
        assert_eq!(ex.check(&Answer::Choices(vec![])), Verdict::Empty);
        assert_eq!(ex.solution(), "a ; c");
    }

    #[test]
    fn commands_ignore_extra_spaces_and_prompt_but_not_case() {
        let ex = exercise(ExerciseKind::WriteCommand {
            answers: vec!["mkdir -p a/b".into(), "mkdir --parents a/b".into()],
        });
        assert_eq!(ex.check(&text("  mkdir   -p a/b ")), Verdict::Correct);
        assert_eq!(ex.check(&text("$ mkdir --parents a/b")), Verdict::Correct);
        assert_eq!(ex.check(&text("MKDIR -p a/b")), Verdict::Incorrect);
        assert_eq!(ex.check(&text("   ")), Verdict::Empty);
    }

    #[test]
    fn complete_command_checks_the_blank() {
        let ex = exercise(ExerciseKind::CompleteCommand {
            template: "tail ___ /var/log/syslog".into(),
            answers: vec!["-f".into(), "--follow".into()],
        });
        assert_eq!(ex.check(&text("-f")), Verdict::Correct);
        assert_eq!(ex.check(&text("-n")), Verdict::Incorrect);
        assert_eq!(ex.solution(), "tail -f /var/log/syslog");
    }

    #[test]
    fn analysis_answers_ignore_case() {
        let ex = exercise(ExerciseKind::AnalyzeOutput {
            output: "-rw-r----- 1 alice compta".into(),
            answers: vec!["compta".into()],
        });
        assert_eq!(ex.check(&text(" COMPTA ")), Verdict::Correct);
        assert_eq!(ex.check(&text("alice")), Verdict::Incorrect);
    }

    #[test]
    fn mismatched_answer_type_is_incorrect() {
        let ex = exercise(ExerciseKind::WriteCommand {
            answers: vec!["ls".into()],
        });
        assert_eq!(ex.check(&Answer::Choices(vec![0])), Verdict::Incorrect);
    }

    #[test]
    fn progress_accumulates_attempts() {
        let mut progress = ExerciseProgress::default();
        progress.record(Outcome::Incorrect, 0);
        progress.record(Outcome::Correct, 2);
        progress.record(Outcome::Correct, 1);
        progress.record(Outcome::Revealed, 0);
        assert_eq!(
            progress,
            ExerciseProgress {
                attempts: 4,
                solved: true,
                revealed: true,
                fewest_hints: Some(1),
            }
        );
    }

    #[test]
    fn outcome_identifiers_round_trip() {
        for o in [Outcome::Correct, Outcome::Incorrect, Outcome::Revealed] {
            assert_eq!(Outcome::parse(o.as_str()), Some(o));
        }
        assert_eq!(Outcome::parse("autre"), None);
    }

    #[test]
    fn validation_reports_inconsistent_exercises() {
        let chapter = id("chapitre");
        let chapters: HashSet<&ContentId> = [&chapter].into_iter().collect();
        let others = HashSet::new();

        let bad_qcm = exercise(ExerciseKind::MultipleChoice {
            choices: vec!["a".into(), "a".into()],
            correct: vec![3],
        });
        let mut bad_template = exercise(ExerciseKind::CompleteCommand {
            template: "tail /var/log/syslog".into(),
            answers: vec![],
        });
        bad_template.id = id("ex-2");
        bad_template.chapter = id("inconnu");
        bad_template.prompt = "Consigne\u{1b}[2J".into();

        let issues = validate(&[bad_qcm, bad_template], &chapters, &others);
        let invalid = issues
            .iter()
            .filter(|i| matches!(i, ContentIssue::InvalidExercise { .. }))
            .count();
        assert_eq!(invalid, 4, "{issues:?}");
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, ContentIssue::UnknownExerciseChapter { .. }))
        );
        assert!(issues.iter().any(|i| matches!(
            i,
            ContentIssue::ControlCharacter {
                field: "prompt",
                ..
            }
        )));
    }

    #[test]
    fn validation_detects_id_collisions() {
        let chapter = id("chapitre");
        let chapters: HashSet<&ContentId> = [&chapter].into_iter().collect();
        let ex = exercise(ExerciseKind::WriteCommand {
            answers: vec!["ls".into()],
        });
        let others: HashSet<&ContentId> = [&chapter].into_iter().collect();
        let mut clash = ex.clone();
        clash.id = id("chapitre");
        let issues = validate(&[ex.clone(), ex, clash], &chapters, &others);
        assert_eq!(
            issues
                .iter()
                .filter(|i| matches!(i, ContentIssue::DuplicateId(_)))
                .count(),
            2
        );
    }
}

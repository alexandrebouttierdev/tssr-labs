//! Progression : XP, niveaux, maîtrise des compétences, déblocage des
//! chapitres et séries d'activité.
//!
//! Tout est calculé à partir de la liste **ordonnée** des tentatives d'un
//! profil : rien n'est stocké en double, et une modification des règles
//! s'applique rétroactivement de façon cohérente.
//!
//! Règles d'XP (voir aussi `docs/DECISIONS.md`) :
//! - base selon le niveau du chapitre : 10 (débutant), 20 (intermédiaire),
//!   30 (avancé) ;
//! - l'XP n'est accordé qu'à la **première** réussite d'un exercice ;
//! - bonus « sans indice » : +50 % de la base ;
//! - bonus « du premier coup » (aucun échec avant) : +25 % de la base ;
//! - solution affichée avant la réussite : aucun XP pour cet exercice ;
//! - les échecs ne retirent jamais d'XP.

use std::collections::{HashMap, HashSet};

use jiff::Timestamp;
use jiff::civil::Date;

use crate::content::{Catalog, Chapter, ContentId, Difficulty};
use crate::exercise::{ExerciseProgress, Outcome};

/// Bonus « sans indice », en pourcentage de la base.
pub const NO_HINT_BONUS_PERCENT: u32 = 50;
/// Bonus « du premier coup », en pourcentage de la base.
pub const FIRST_TRY_BONUS_PERCENT: u32 = 25;
/// Pas des niveaux : passer du niveau n au niveau n+1 demande `n × LEVEL_STEP` XP.
pub const LEVEL_STEP: u32 = 100;

/// Tentative enregistrée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptRecord {
    /// Exercice.
    pub exercise: ContentId,
    /// Issue.
    pub outcome: Outcome,
    /// Indices utilisés.
    pub hints_used: u8,
    /// Date.
    pub at: Timestamp,
}

/// XP de base d'un exercice selon le niveau de son chapitre.
pub fn base_xp(difficulty: Difficulty) -> u32 {
    match difficulty {
        Difficulty::Beginner => 10,
        Difficulty::Intermediate => 20,
        Difficulty::Advanced => 30,
    }
}

/// XP obtenu pour un exercice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XpAward {
    /// Base.
    pub base: u32,
    /// Bonus sans indice.
    pub no_hint_bonus: u32,
    /// Bonus du premier coup.
    pub first_try_bonus: u32,
    /// Date de la réussite qui a rapporté l'XP.
    pub at: Timestamp,
}

impl XpAward {
    /// Total.
    pub fn total(&self) -> u32 {
        self.base + self.no_hint_bonus + self.first_try_bonus
    }
}

/// XP d'un exercice à partir de ses tentatives, dans l'ordre chronologique.
/// `None` si l'exercice n'est pas réussi, ou si la solution a été affichée
/// avant la première réussite.
pub fn award(difficulty: Difficulty, attempts: &[&AttemptRecord]) -> Option<XpAward> {
    let first_success = attempts
        .iter()
        .position(|a| a.outcome == Outcome::Correct)?;
    let before = &attempts[..first_success];
    if before.iter().any(|a| a.outcome == Outcome::Revealed) {
        return None;
    }
    let success = attempts[first_success];
    let base = base_xp(difficulty);
    let bonus = |percent: u32| base * percent / 100;
    Some(XpAward {
        base,
        no_hint_bonus: if success.hints_used == 0 {
            bonus(NO_HINT_BONUS_PERCENT)
        } else {
            0
        },
        first_try_bonus: if before.is_empty() {
            bonus(FIRST_TRY_BONUS_PERCENT)
        } else {
            0
        },
        at: success.at,
    })
}

/// Niveau global.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    /// Numéro (à partir de 1).
    pub number: u32,
    /// XP acquis dans le niveau courant.
    pub xp_into: u32,
    /// XP nécessaire pour passer au niveau suivant (depuis le début du niveau).
    pub xp_span: u32,
}

/// XP total nécessaire pour atteindre un niveau : 0, 100, 300, 600, 1000…
pub fn level_threshold(level: u32) -> u32 {
    let n = level.saturating_sub(1);
    LEVEL_STEP.saturating_mul(n.saturating_mul(n + 1) / 2)
}

/// Niveau correspondant à un total d'XP.
pub fn level_for(total_xp: u32) -> Level {
    let mut number = 1;
    while level_threshold(number + 1) <= total_xp {
        number += 1;
    }
    Level {
        number,
        xp_into: total_xp - level_threshold(number),
        xp_span: level_threshold(number + 1) - level_threshold(number),
    }
}

/// Degré de maîtrise d'une compétence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mastery {
    /// Aucun exercice réussi.
    NotStarted,
    /// Moins de 40 % des exercices réussis.
    Discovering,
    /// De 40 à 79 %.
    Progressing,
    /// 80 % ou plus.
    Mastered,
}

/// Maîtrise d'une compétence : part des exercices réussis (sans avoir
/// affiché la solution avant) parmi ceux des chapitres qui la travaillent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMastery {
    /// Compétence.
    pub skill: ContentId,
    /// Exercices réussis.
    pub solved: usize,
    /// Exercices disponibles.
    pub total: usize,
}

impl SkillMastery {
    /// Pourcentage (0 si aucun exercice).
    pub fn percent(&self) -> u32 {
        if self.total == 0 {
            return 0;
        }
        u32::try_from(self.solved * 100 / self.total).unwrap_or(100)
    }

    /// Degré de maîtrise.
    pub fn mastery(&self) -> Mastery {
        match self.percent() {
            0 if self.solved == 0 => Mastery::NotStarted,
            p if p >= 80 => Mastery::Mastered,
            p if p >= 40 => Mastery::Progressing,
            _ => Mastery::Discovering,
        }
    }
}

/// État d'un chapitre dans un parcours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChapterStatus {
    /// Un prérequis n'est pas terminé.
    Locked {
        /// Prérequis à terminer.
        missing: Vec<ContentId>,
    },
    /// Accessible, exercices en cours ou non commencés.
    Available,
    /// Tous les exercices sont terminés (réussis ou solution affichée).
    Completed,
}

/// Bilan de progression d'un profil.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    /// Progression par exercice.
    pub exercises: HashMap<ContentId, ExerciseProgress>,
    /// XP obtenu par exercice.
    pub awards: HashMap<ContentId, XpAward>,
    /// XP total.
    pub total_xp: u32,
    /// Maîtrise par compétence, dans l'ordre du catalogue.
    pub skills: Vec<SkillMastery>,
}

impl Report {
    /// Calcule le bilan à partir des tentatives (ordre chronologique).
    /// Les tentatives portant sur un exercice absent du catalogue sont
    /// ignorées.
    pub fn compute(catalog: &Catalog, attempts: &[AttemptRecord]) -> Self {
        let mut by_exercise: HashMap<&ContentId, Vec<&AttemptRecord>> = HashMap::new();
        for attempt in attempts {
            by_exercise
                .entry(&attempt.exercise)
                .or_default()
                .push(attempt);
        }

        let mut report = Self::default();
        for exercise in catalog.exercises() {
            let Some(list) = by_exercise.get(&exercise.id) else {
                continue;
            };
            let mut progress = ExerciseProgress::default();
            for attempt in list {
                progress.record(attempt.outcome, attempt.hints_used);
            }
            report.exercises.insert(exercise.id.clone(), progress);
            let difficulty = catalog
                .chapter(&exercise.chapter)
                .map_or(Difficulty::Beginner, |c| c.difficulty);
            if let Some(award) = award(difficulty, list) {
                report.total_xp += award.total();
                report.awards.insert(exercise.id.clone(), award);
            }
        }

        report.skills = catalog
            .skills()
            .iter()
            .map(|skill| {
                let exercises: Vec<_> = catalog
                    .exercises()
                    .iter()
                    .filter(|e| {
                        catalog
                            .chapter(&e.chapter)
                            .is_some_and(|c| c.skills.contains(&skill.id))
                    })
                    .collect();
                SkillMastery {
                    skill: skill.id.clone(),
                    solved: exercises
                        .iter()
                        .filter(|e| report.awards.contains_key(&e.id))
                        .count(),
                    total: exercises.len(),
                }
            })
            .collect();
        report
    }

    /// Niveau global.
    pub fn level(&self) -> Level {
        level_for(self.total_xp)
    }

    /// Nombre d'exercices réussis.
    pub fn solved_count(&self) -> usize {
        self.exercises.values().filter(|p| p.solved).count()
    }

    /// Un exercice est terminé (réussi ou solution affichée).
    pub fn is_finished(&self, exercise: &ContentId) -> bool {
        self.exercises
            .get(exercise)
            .is_some_and(|p| p.solved || p.revealed)
    }

    /// Un chapitre est terminé quand tous ses exercices le sont. Un
    /// chapitre sans exercice est considéré comme terminé : la lecture
    /// n'est pas encore suivie et ne doit pas bloquer un parcours.
    pub fn is_chapter_completed(&self, catalog: &Catalog, chapter: &ContentId) -> bool {
        catalog
            .exercises_for(chapter)
            .iter()
            .all(|e| self.is_finished(&e.id))
    }

    /// État d'un chapitre (verrouillage selon ses prérequis).
    pub fn chapter_status(&self, catalog: &Catalog, chapter: &Chapter) -> ChapterStatus {
        let missing: Vec<ContentId> = chapter
            .prerequisites
            .iter()
            .filter(|p| !self.is_chapter_completed(catalog, p))
            .cloned()
            .collect();
        if !missing.is_empty() {
            ChapterStatus::Locked { missing }
        } else if self.is_chapter_completed(catalog, &chapter.id) {
            ChapterStatus::Completed
        } else {
            ChapterStatus::Available
        }
    }

    /// Prochain chapitre recommandé : le premier chapitre accessible et non
    /// terminé, dans l'ordre des parcours.
    pub fn recommended<'c>(&self, catalog: &'c Catalog) -> Option<&'c Chapter> {
        let mut seen = HashSet::new();
        catalog
            .paths()
            .iter()
            .flat_map(|p| p.chapters.iter())
            .filter(|id| seen.insert(*id))
            .filter_map(|id| catalog.chapter(id))
            .find(|c| self.chapter_status(catalog, c) == ChapterStatus::Available)
    }
}

/// Série d'activité : nombre de jours consécutifs, jusqu'à aujourd'hui (ou
/// hier, si rien n'a encore été fait aujourd'hui), comportant au moins une
/// tentative. Les dates sont celles du fuseau de l'apprenant.
pub fn streak(activity_days: &[Date], today: Date) -> u32 {
    let days: HashSet<Date> = activity_days.iter().copied().collect();
    let Ok(yesterday) = today.yesterday() else {
        return 0;
    };
    let mut day = if days.contains(&today) {
        today
    } else {
        yesterday
    };
    let mut count = 0;
    while days.contains(&day) {
        count += 1;
        match day.yesterday() {
            Ok(previous) => day = previous,
            Err(_) => break,
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Domain, LearningPath, Skill};
    use crate::exercise::{Exercise, ExerciseKind};
    use jiff::civil::date;

    fn id(v: &str) -> ContentId {
        ContentId::parse(v).unwrap()
    }

    fn attempt(exercise: &str, outcome: Outcome, hints: u8, second: i64) -> AttemptRecord {
        AttemptRecord {
            exercise: id(exercise),
            outcome,
            hints_used: hints,
            at: Timestamp::from_second(second).unwrap(),
        }
    }

    fn award_of(attempts: &[AttemptRecord]) -> Option<u32> {
        let refs: Vec<&AttemptRecord> = attempts.iter().collect();
        award(Difficulty::Beginner, &refs).map(|a| a.total())
    }

    #[test]
    fn xp_rules() {
        use Outcome::{Correct, Incorrect, Revealed};
        // Premier coup, sans indice : 10 + 5 + 2.
        assert_eq!(award_of(&[attempt("e", Correct, 0, 1)]), Some(17));
        // Après un échec, sans indice : 10 + 5.
        assert_eq!(
            award_of(&[attempt("e", Incorrect, 0, 1), attempt("e", Correct, 0, 2)]),
            Some(15)
        );
        // Du premier coup avec indice : 10 + 2.
        assert_eq!(award_of(&[attempt("e", Correct, 1, 1)]), Some(12));
        // Solution vue avant : rien.
        assert_eq!(
            award_of(&[attempt("e", Revealed, 0, 1), attempt("e", Correct, 0, 2)]),
            None
        );
        // Seule la première réussite compte (ici du premier coup, avec indices : 10 + 2) ;
        // la solution vue après n'enlève rien.
        assert_eq!(
            award_of(&[
                attempt("e", Correct, 2, 1),
                attempt("e", Revealed, 0, 2),
                attempt("e", Correct, 0, 3)
            ]),
            Some(12)
        );
        assert_eq!(award_of(&[attempt("e", Incorrect, 0, 1)]), None);
        assert_eq!(base_xp(Difficulty::Advanced), 30);
    }

    #[test]
    fn levels_grow_by_increasing_steps() {
        assert_eq!(level_threshold(1), 0);
        assert_eq!(level_threshold(2), 100);
        assert_eq!(level_threshold(3), 300);
        assert_eq!(level_threshold(4), 600);
        assert_eq!(
            level_for(0),
            Level {
                number: 1,
                xp_into: 0,
                xp_span: 100
            }
        );
        assert_eq!(level_for(99).number, 1);
        assert_eq!(
            level_for(100),
            Level {
                number: 2,
                xp_into: 0,
                xp_span: 200
            }
        );
        assert_eq!(
            level_for(450),
            Level {
                number: 3,
                xp_into: 150,
                xp_span: 300
            }
        );
    }

    #[test]
    fn mastery_thresholds() {
        let m = |solved, total| {
            SkillMastery {
                skill: id("s"),
                solved,
                total,
            }
            .mastery()
        };
        assert_eq!(m(0, 5), Mastery::NotStarted);
        assert_eq!(m(0, 0), Mastery::NotStarted);
        assert_eq!(m(1, 5), Mastery::Discovering);
        assert_eq!(m(2, 5), Mastery::Progressing);
        assert_eq!(m(4, 5), Mastery::Mastered);
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let today = date(2026, 11, 4);
        let days = [
            date(2026, 11, 4),
            date(2026, 11, 3),
            date(2026, 11, 2),
            date(2026, 10, 30),
        ];
        assert_eq!(streak(&days, today), 3);
        // Rien aujourd'hui : la série d'hier compte encore.
        assert_eq!(streak(&days[1..], today), 2);
        assert_eq!(streak(&[date(2026, 11, 1)], today), 0);
        assert_eq!(streak(&[], today), 0);
    }

    fn catalog() -> Catalog {
        let chapter = |name: &str, difficulty, prerequisites: &[&str]| Chapter {
            id: id(name),
            version: "1".into(),
            title: name.into(),
            summary: "R".into(),
            domain: Domain::Linux,
            difficulty,
            estimated_minutes: 10,
            tags: vec![],
            skills: vec![id("shell")],
            objectives: vec![],
            prerequisites: prerequisites.iter().map(|p| id(p)).collect(),
            body: "# T\n".into(),
            sources: vec![],
            authors: vec!["A".into()],
            license: "MIT".into(),
        };
        let exercise = |name: &str, chapter: &str| Exercise {
            id: id(name),
            chapter: id(chapter),
            title: "T".into(),
            prompt: "P".into(),
            kind: ExerciseKind::WriteCommand {
                answers: vec!["ls".into()],
            },
            hints: vec![],
            explanation: "E".into(),
        };
        Catalog::new(
            vec![Skill {
                id: id("shell"),
                name: "Shell".into(),
                domain: Domain::Linux,
            }],
            vec![
                chapter("a", Difficulty::Beginner, &[]),
                chapter("b", Difficulty::Intermediate, &["a"]),
                chapter("c", Difficulty::Intermediate, &["b"]),
            ],
            vec![LearningPath {
                id: id("p"),
                title: "P".into(),
                summary: "R".into(),
                chapters: vec![id("a"), id("b"), id("c")],
            }],
        )
        .unwrap()
        .with_exercises(vec![
            exercise("a-1", "a"),
            exercise("a-2", "a"),
            exercise("b-1", "b"),
        ])
        .unwrap()
    }

    #[test]
    fn report_combines_xp_mastery_and_unlocking() {
        use Outcome::{Correct, Incorrect, Revealed};
        let catalog = catalog();
        let chapter = |name: &str| catalog.chapter(&id(name)).unwrap().clone();

        let empty = Report::compute(&catalog, &[]);
        assert_eq!(empty.total_xp, 0);
        assert_eq!(
            empty.chapter_status(&catalog, &chapter("a")),
            ChapterStatus::Available
        );
        assert_eq!(
            empty.chapter_status(&catalog, &chapter("b")),
            ChapterStatus::Locked {
                missing: vec![id("a")]
            }
        );
        assert_eq!(
            empty.recommended(&catalog).map(|c| c.id.as_str()),
            Some("a")
        );

        let attempts = [
            attempt("a-1", Correct, 0, 1), // 17
            attempt("a-2", Incorrect, 0, 2),
            attempt("a-2", Revealed, 1, 3),    // terminé, 0 XP
            attempt("b-1", Correct, 1, 4),     // 20 + 5 (premier coup)
            attempt("inconnu", Correct, 0, 5), // ignoré
        ];
        let report = Report::compute(&catalog, &attempts);
        assert_eq!(report.total_xp, 17 + 25);
        assert_eq!(report.solved_count(), 2);
        assert_eq!(
            report.chapter_status(&catalog, &chapter("a")),
            ChapterStatus::Completed
        );
        assert_eq!(
            report.chapter_status(&catalog, &chapter("b")),
            ChapterStatus::Completed
        );
        // « c » n'a pas d'exercice : terminé dès qu'il est accessible.
        assert_eq!(
            report.chapter_status(&catalog, &chapter("c")),
            ChapterStatus::Completed
        );
        assert_eq!(report.recommended(&catalog), None);
        // Solution affichée : terminé mais pas maîtrisé.
        assert_eq!(report.skills[0].solved, 2);
        assert_eq!(report.skills[0].total, 3);
        assert_eq!(report.skills[0].mastery(), Mastery::Progressing);
    }
}

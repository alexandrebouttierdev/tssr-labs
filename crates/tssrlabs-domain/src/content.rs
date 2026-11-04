//! Contenu pédagogique : compétences, chapitres, parcours et recherche.
//!
//! Le catalogue est validé à sa construction : identifiants, références
//! croisées, absence de cycle dans les prérequis et absence de caractères de
//! contrôle (un contenu importé ne doit jamais pouvoir injecter de séquence
//! d'échappement dans le terminal).

use std::collections::{HashMap, HashSet};

use jiff::civil::Date;

use crate::exercise::Exercise;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Longueur maximale d'un identifiant de contenu.
pub const MAX_ID_LEN: usize = 64;

/// Identifiant de contenu : minuscules ASCII, chiffres et tirets
/// (`linux-arborescence`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId(String);

impl ContentId {
    /// Valide un identifiant.
    pub fn parse(value: &str) -> Option<Self> {
        let valid = !value.is_empty()
            && value.len() <= MAX_ID_LEN
            && value.split('-').all(|part| {
                !part.is_empty()
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            });
        valid.then(|| Self(value.to_owned()))
    }

    /// Valeur textuelle.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Domaines pédagogiques (voir `docs/PRODUCT_BRIEF.md`, section 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Domain {
    /// Linux et Bash.
    Linux,
    /// Windows et PowerShell.
    Windows,
    /// Réseaux.
    Network,
    /// Systèmes : stockage, processus, journaux, services.
    Systems,
    /// Sécurité.
    Security,
    /// Virtualisation et conteneurs.
    Virtualization,
    /// Diagnostic et dépannage.
    Troubleshooting,
    /// Automatisation.
    Automation,
}

impl Domain {
    /// Tous les domaines, dans l'ordre d'affichage.
    pub const ALL: [Self; 8] = [
        Self::Linux,
        Self::Windows,
        Self::Network,
        Self::Systems,
        Self::Security,
        Self::Virtualization,
        Self::Troubleshooting,
        Self::Automation,
    ];

    /// Identifiant stable (format de contenu).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Network => "reseau",
            Self::Systems => "systeme",
            Self::Security => "securite",
            Self::Virtualization => "virtualisation",
            Self::Troubleshooting => "depannage",
            Self::Automation => "automatisation",
        }
    }

    /// Lit un identifiant.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.as_str() == value)
    }
}

/// Niveau de difficulté.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Difficulty {
    /// Débutant.
    Beginner,
    /// Intermédiaire.
    Intermediate,
    /// Avancé.
    Advanced,
}

impl Difficulty {
    /// Tous les niveaux.
    pub const ALL: [Self; 3] = [Self::Beginner, Self::Intermediate, Self::Advanced];

    /// Identifiant stable (format de contenu).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Beginner => "debutant",
            Self::Intermediate => "intermediaire",
            Self::Advanced => "avance",
        }
    }

    /// Lit un identifiant.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.as_str() == value)
    }
}

/// Compétence évaluable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    /// Identifiant.
    pub id: ContentId,
    /// Nom affiché.
    pub name: String,
    /// Domaine.
    pub domain: Domain,
}

/// Source de référence d'un contenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Titre.
    pub title: String,
    /// Adresse.
    pub url: String,
    /// Date de dernière vérification par un humain ; `None` si non vérifiée.
    pub verified_on: Option<Date>,
}

/// Chapitre : contenu théorique d'un sujet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    /// Identifiant.
    pub id: ContentId,
    /// Version du contenu.
    pub version: String,
    /// Titre.
    pub title: String,
    /// Résumé en une ou deux phrases.
    pub summary: String,
    /// Domaine.
    pub domain: Domain,
    /// Difficulté.
    pub difficulty: Difficulty,
    /// Durée estimée, en minutes.
    pub estimated_minutes: u16,
    /// Mots-clés.
    pub tags: Vec<String>,
    /// Compétences travaillées.
    pub skills: Vec<ContentId>,
    /// Objectifs pédagogiques.
    pub objectives: Vec<String>,
    /// Chapitres à connaître avant celui-ci.
    pub prerequisites: Vec<ContentId>,
    /// Corps en Markdown.
    pub body: String,
    /// Sources de référence.
    pub sources: Vec<Source>,
    /// Auteurs.
    pub authors: Vec<String>,
    /// Licence du contenu (identifiant SPDX).
    pub license: String,
}

/// Parcours guidé : suite ordonnée de chapitres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningPath {
    /// Identifiant.
    pub id: ContentId,
    /// Titre.
    pub title: String,
    /// Résumé.
    pub summary: String,
    /// Chapitres, dans l'ordre recommandé.
    pub chapters: Vec<ContentId>,
}

/// Gravité d'une anomalie de contenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Le contenu est refusé.
    Error,
    /// Le contenu est accepté mais doit être corrigé.
    Warning,
}

/// Anomalie détectée dans un catalogue.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContentIssue {
    /// Identifiant utilisé deux fois.
    #[error("identifiant en double : {0}")]
    DuplicateId(String),
    /// Compétence inconnue.
    #[error("{chapter} : compétence inconnue « {skill} »")]
    UnknownSkill {
        /// Chapitre.
        chapter: String,
        /// Compétence citée.
        skill: String,
    },
    /// Prérequis inconnu.
    #[error("{chapter} : prérequis inconnu « {prerequisite} »")]
    UnknownPrerequisite {
        /// Chapitre.
        chapter: String,
        /// Prérequis cité.
        prerequisite: String,
    },
    /// Cycle dans les prérequis.
    #[error("{0} : cycle dans les prérequis")]
    PrerequisiteCycle(String),
    /// Chapitre de parcours inconnu.
    #[error("parcours {path} : chapitre inconnu « {chapter} »")]
    UnknownPathChapter {
        /// Parcours.
        path: String,
        /// Chapitre cité.
        chapter: String,
    },
    /// Chapitre placé avant l'un de ses prérequis dans un parcours.
    #[error("parcours {path} : « {chapter} » apparaît avant son prérequis « {prerequisite} »")]
    PathOrder {
        /// Parcours.
        path: String,
        /// Chapitre.
        chapter: String,
        /// Prérequis placé après.
        prerequisite: String,
    },
    /// Champ obligatoire vide.
    #[error("{id} : champ « {field} » vide")]
    EmptyField {
        /// Élément.
        id: String,
        /// Champ.
        field: &'static str,
    },
    /// Caractère de contrôle interdit.
    #[error("{id} : caractère de contrôle interdit dans « {field} »")]
    ControlCharacter {
        /// Élément.
        id: String,
        /// Champ.
        field: &'static str,
    },
    /// Durée nulle.
    #[error("{0} : durée estimée nulle")]
    ZeroDuration(String),
    /// Exercice rattaché à un chapitre inconnu.
    #[error("exercice {exercise} : chapitre inconnu « {chapter} »")]
    UnknownExerciseChapter {
        /// Exercice.
        exercise: String,
        /// Chapitre cité.
        chapter: String,
    },
    /// Exercice incohérent avec son type.
    #[error("exercice {exercise} : {reason}")]
    InvalidExercise {
        /// Exercice.
        exercise: String,
        /// Raison.
        reason: &'static str,
    },
    /// Source sans date de vérification.
    #[error("{chapter} : source non vérifiée « {source_title} »")]
    UnverifiedSource {
        /// Chapitre.
        chapter: String,
        /// Titre de la source.
        source_title: String,
    },
}

impl ContentIssue {
    /// Gravité de l'anomalie.
    pub fn severity(&self) -> Severity {
        match self {
            Self::UnverifiedSource { .. } => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

/// Catalogue validé.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Catalog {
    skills: Vec<Skill>,
    chapters: Vec<Chapter>,
    paths: Vec<LearningPath>,
    exercises: Vec<Exercise>,
    warnings: Vec<ContentIssue>,
}

/// Critères de recherche dans la bibliothèque.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LibraryQuery {
    /// Texte libre (insensible à la casse et aux accents).
    pub text: String,
    /// Domaine.
    pub domain: Option<Domain>,
    /// Difficulté.
    pub difficulty: Option<Difficulty>,
    /// Durée maximale, en minutes.
    pub max_minutes: Option<u16>,
}

impl LibraryQuery {
    /// Indique si un critère est actif.
    pub fn is_filtered(&self) -> bool {
        !self.text.trim().is_empty()
            || self.domain.is_some()
            || self.difficulty.is_some()
            || self.max_minutes.is_some()
    }
}

/// Texte normalisé pour la recherche : minuscules, sans accents.
pub fn fold(text: &str) -> String {
    text.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}

fn has_control(text: &str, allow_newlines: bool) -> bool {
    text.chars()
        .any(|c| c.is_control() && !(allow_newlines && (c == '\n' || c == '\r' || c == '\t')))
}

impl Catalog {
    /// Construit et valide un catalogue. Renvoie toutes les erreurs
    /// bloquantes ; les avertissements sont conservés dans le catalogue.
    pub fn new(
        skills: Vec<Skill>,
        mut chapters: Vec<Chapter>,
        paths: Vec<LearningPath>,
    ) -> Result<Self, Vec<ContentIssue>> {
        let mut issues = Vec::new();

        let mut ids = HashSet::new();
        let all_ids = skills
            .iter()
            .map(|s| &s.id)
            .chain(chapters.iter().map(|c| &c.id))
            .chain(paths.iter().map(|p| &p.id));
        for id in all_ids {
            if !ids.insert(id.clone()) {
                issues.push(ContentIssue::DuplicateId(id.to_string()));
            }
        }

        let skill_ids: HashSet<&ContentId> = skills.iter().map(|s| &s.id).collect();
        let chapter_ids: HashSet<&ContentId> = chapters.iter().map(|c| &c.id).collect();

        let mut check_text = |id: &ContentId, field: &'static str, text: &str, multiline: bool| {
            if text.trim().is_empty() {
                issues.push(ContentIssue::EmptyField {
                    id: id.to_string(),
                    field,
                });
            } else if has_control(text, multiline) {
                issues.push(ContentIssue::ControlCharacter {
                    id: id.to_string(),
                    field,
                });
            }
        };
        for skill in &skills {
            check_text(&skill.id, "name", &skill.name, false);
        }
        for path in &paths {
            check_text(&path.id, "title", &path.title, false);
            check_text(&path.id, "summary", &path.summary, false);
        }
        for chapter in &chapters {
            check_text(&chapter.id, "title", &chapter.title, false);
            check_text(&chapter.id, "summary", &chapter.summary, false);
            check_text(&chapter.id, "body", &chapter.body, true);
            check_text(&chapter.id, "version", &chapter.version, false);
            check_text(&chapter.id, "license", &chapter.license, false);
            for tag in &chapter.tags {
                check_text(&chapter.id, "tags", tag, false);
            }
            for objective in &chapter.objectives {
                check_text(&chapter.id, "objectives", objective, false);
            }
            for author in &chapter.authors {
                check_text(&chapter.id, "authors", author, false);
            }
            for source in &chapter.sources {
                check_text(&chapter.id, "sources", &source.title, false);
                check_text(&chapter.id, "sources", &source.url, false);
            }
        }

        let mut warnings = Vec::new();
        for chapter in &chapters {
            if chapter.estimated_minutes == 0 {
                issues.push(ContentIssue::ZeroDuration(chapter.id.to_string()));
            }
            for skill in &chapter.skills {
                if !skill_ids.contains(skill) {
                    issues.push(ContentIssue::UnknownSkill {
                        chapter: chapter.id.to_string(),
                        skill: skill.to_string(),
                    });
                }
            }
            for prerequisite in &chapter.prerequisites {
                if !chapter_ids.contains(prerequisite) {
                    issues.push(ContentIssue::UnknownPrerequisite {
                        chapter: chapter.id.to_string(),
                        prerequisite: prerequisite.to_string(),
                    });
                }
            }
            for source in &chapter.sources {
                if source.verified_on.is_none() {
                    warnings.push(ContentIssue::UnverifiedSource {
                        chapter: chapter.id.to_string(),
                        source_title: source.title.clone(),
                    });
                }
            }
        }
        issues.extend(prerequisite_cycles(&chapters));

        for path in &paths {
            let mut seen: Vec<&ContentId> = Vec::new();
            for id in &path.chapters {
                match chapters.iter().find(|c| &c.id == id) {
                    None => issues.push(ContentIssue::UnknownPathChapter {
                        path: path.id.to_string(),
                        chapter: id.to_string(),
                    }),
                    Some(chapter) => {
                        for prerequisite in &chapter.prerequisites {
                            if path.chapters.contains(prerequisite) && !seen.contains(&prerequisite)
                            {
                                issues.push(ContentIssue::PathOrder {
                                    path: path.id.to_string(),
                                    chapter: id.to_string(),
                                    prerequisite: prerequisite.to_string(),
                                });
                            }
                        }
                    }
                }
                seen.push(id);
            }
        }

        if !issues.is_empty() {
            return Err(issues);
        }
        // Ordre stable pour la bibliothèque : niveau, puis titre.
        chapters.sort_by(|a, b| {
            a.difficulty
                .cmp(&b.difficulty)
                .then_with(|| fold(&a.title).cmp(&fold(&b.title)))
        });
        Ok(Self {
            skills,
            chapters,
            paths,
            exercises: Vec::new(),
            warnings,
        })
    }

    /// Ajoute des exercices après validation (voir [`crate::exercise::validate`]).
    pub fn with_exercises(mut self, exercises: Vec<Exercise>) -> Result<Self, Vec<ContentIssue>> {
        let chapter_ids: HashSet<&ContentId> = self.chapters.iter().map(|c| &c.id).collect();
        let other_ids: HashSet<&ContentId> = self
            .skills
            .iter()
            .map(|s| &s.id)
            .chain(self.chapters.iter().map(|c| &c.id))
            .chain(self.paths.iter().map(|p| &p.id))
            .collect();
        let issues = crate::exercise::validate(&exercises, &chapter_ids, &other_ids);
        if !issues.is_empty() {
            return Err(issues);
        }
        self.exercises = exercises;
        Ok(self)
    }

    /// Exercices d'un chapitre, dans l'ordre de déclaration.
    pub fn exercises_for(&self, chapter: &ContentId) -> Vec<&Exercise> {
        self.exercises
            .iter()
            .filter(|e| &e.chapter == chapter)
            .collect()
    }

    /// Tous les exercices.
    pub fn exercises(&self) -> &[Exercise] {
        &self.exercises
    }

    /// Compétences.
    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    /// Chapitres (par niveau puis titre).
    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    /// Parcours.
    pub fn paths(&self) -> &[LearningPath] {
        &self.paths
    }

    /// Avertissements de validation (le contenu reste utilisable).
    pub fn warnings(&self) -> &[ContentIssue] {
        &self.warnings
    }

    /// Chapitre par identifiant.
    pub fn chapter(&self, id: &ContentId) -> Option<&Chapter> {
        self.chapters.iter().find(|c| &c.id == id)
    }

    /// Compétence par identifiant.
    pub fn skill(&self, id: &ContentId) -> Option<&Skill> {
        self.skills.iter().find(|s| &s.id == id)
    }

    /// Domaines ayant au moins un chapitre, dans l'ordre d'affichage.
    pub fn domains(&self) -> Vec<Domain> {
        Domain::ALL
            .into_iter()
            .filter(|d| self.chapters.iter().any(|c| c.domain == *d))
            .collect()
    }

    /// Chapitres correspondant à la recherche. Chaque mot du texte doit
    /// apparaître dans le titre, le résumé, les mots-clés, les objectifs ou
    /// les compétences (sans tenir compte de la casse ni des accents).
    pub fn search(&self, query: &LibraryQuery) -> Vec<&Chapter> {
        let words: Vec<String> = fold(&query.text)
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        self.chapters
            .iter()
            .filter(|c| query.domain.is_none_or(|d| c.domain == d))
            .filter(|c| query.difficulty.is_none_or(|d| c.difficulty == d))
            .filter(|c| query.max_minutes.is_none_or(|m| c.estimated_minutes <= m))
            .filter(|c| {
                if words.is_empty() {
                    return true;
                }
                let haystack = fold(&self.searchable_text(c));
                words.iter().all(|w| haystack.contains(w.as_str()))
            })
            .collect()
    }

    fn searchable_text(&self, chapter: &Chapter) -> String {
        let skills = chapter
            .skills
            .iter()
            .filter_map(|id| self.skill(id))
            .map(|s| s.name.as_str());
        [chapter.title.as_str(), chapter.summary.as_str()]
            .into_iter()
            .chain(chapter.tags.iter().map(String::as_str))
            .chain(chapter.objectives.iter().map(String::as_str))
            .chain(skills)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Détecte les cycles de prérequis (parcours en profondeur).
fn prerequisite_cycles(chapters: &[Chapter]) -> Vec<ContentIssue> {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        Visiting,
        Done,
    }
    fn visit<'a>(
        id: &'a ContentId,
        graph: &HashMap<&'a ContentId, &'a [ContentId]>,
        marks: &mut HashMap<&'a ContentId, Mark>,
        cycles: &mut Vec<ContentIssue>,
    ) {
        match marks.get(id) {
            Some(Mark::Done) => return,
            Some(Mark::Visiting) => {
                cycles.push(ContentIssue::PrerequisiteCycle(id.to_string()));
                return;
            }
            None => {}
        }
        marks.insert(id, Mark::Visiting);
        for next in graph.get(id).copied().unwrap_or_default() {
            if graph.contains_key(next) {
                visit(next, graph, marks, cycles);
            }
        }
        marks.insert(id, Mark::Done);
    }

    let graph: HashMap<&ContentId, &[ContentId]> = chapters
        .iter()
        .map(|c| (&c.id, c.prerequisites.as_slice()))
        .collect();
    let mut marks = HashMap::new();
    let mut cycles = Vec::new();
    for chapter in chapters {
        visit(&chapter.id, &graph, &mut marks, &mut cycles);
    }
    cycles
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn id(value: &str) -> ContentId {
        ContentId::parse(value).unwrap()
    }

    pub(crate) fn chapter(name: &str, prerequisites: &[&str]) -> Chapter {
        Chapter {
            id: id(name),
            version: "1.0.0".into(),
            title: format!("Titre {name}"),
            summary: "Résumé".into(),
            domain: Domain::Linux,
            difficulty: Difficulty::Beginner,
            estimated_minutes: 20,
            tags: vec!["shell".into()],
            skills: vec![id("linux-shell")],
            objectives: vec!["Comprendre".into()],
            prerequisites: prerequisites.iter().map(|p| id(p)).collect(),
            body: "# Titre\n\nTexte.\n".into(),
            sources: vec![],
            authors: vec!["Équipe".into()],
            license: "MIT".into(),
        }
    }

    pub(crate) fn skill() -> Skill {
        Skill {
            id: id("linux-shell"),
            name: "Utiliser le shell".into(),
            domain: Domain::Linux,
        }
    }

    fn path(name: &str, chapters: &[&str]) -> LearningPath {
        LearningPath {
            id: id(name),
            title: "Parcours".into(),
            summary: "Résumé".into(),
            chapters: chapters.iter().map(|c| id(c)).collect(),
        }
    }

    #[test]
    fn content_ids_are_kebab_case() {
        assert!(ContentId::parse("linux-bases-2").is_some());
        for bad in [
            "",
            "Linux",
            "a--b",
            "-a",
            "a-",
            "a_b",
            "é",
            &"a".repeat(MAX_ID_LEN + 1),
        ] {
            assert!(ContentId::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn stable_identifiers_round_trip() {
        for d in Domain::ALL {
            assert_eq!(Domain::parse(d.as_str()), Some(d));
        }
        for d in Difficulty::ALL {
            assert_eq!(Difficulty::parse(d.as_str()), Some(d));
        }
    }

    #[test]
    fn valid_catalog_is_accepted() {
        let catalog = Catalog::new(
            vec![skill()],
            vec![chapter("a", &[]), chapter("b", &["a"])],
            vec![path("p", &["a", "b"])],
        )
        .unwrap();
        assert_eq!(catalog.chapters().len(), 2);
        assert_eq!(catalog.domains(), vec![Domain::Linux]);
        assert!(catalog.warnings().is_empty());
    }

    #[test]
    fn broken_references_are_reported() {
        let mut bad = chapter("b", &["inconnu"]);
        bad.skills.push(id("competence-inconnue"));
        let issues = Catalog::new(
            vec![skill()],
            vec![chapter("a", &[]), bad, chapter("a", &[])],
            vec![path("p", &["a", "fantome"])],
        )
        .unwrap_err();
        assert!(issues.contains(&ContentIssue::DuplicateId("a".into())));
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, ContentIssue::UnknownSkill { .. }))
        );
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, ContentIssue::UnknownPrerequisite { .. }))
        );
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, ContentIssue::UnknownPathChapter { .. }))
        );
    }

    #[test]
    fn prerequisite_cycles_and_path_order_are_rejected() {
        let issues = Catalog::new(
            vec![skill()],
            vec![
                chapter("a", &["c"]),
                chapter("b", &["a"]),
                chapter("c", &["b"]),
            ],
            vec![],
        )
        .unwrap_err();
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, ContentIssue::PrerequisiteCycle(_)))
        );

        let issues = Catalog::new(
            vec![skill()],
            vec![chapter("a", &[]), chapter("b", &["a"])],
            vec![path("p", &["b", "a"])],
        )
        .unwrap_err();
        assert!(matches!(&issues[..], [ContentIssue::PathOrder { .. }]));
    }

    #[test]
    fn control_characters_and_empty_fields_are_rejected() {
        let mut injected = chapter("a", &[]);
        injected.title = "Titre\u{1b}]52;c;cGF3bmVk\u{7}".into();
        let mut empty = chapter("b", &[]);
        empty.summary = "  ".into();
        empty.estimated_minutes = 0;
        let issues = Catalog::new(vec![skill()], vec![injected, empty], vec![]).unwrap_err();
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, ContentIssue::ControlCharacter { field: "title", .. }))
        );
        assert!(issues.iter().any(|i| matches!(
            i,
            ContentIssue::EmptyField {
                field: "summary",
                ..
            }
        )));
        assert!(issues.contains(&ContentIssue::ZeroDuration("b".into())));
    }

    #[test]
    fn unverified_sources_are_warnings_only() {
        let mut c = chapter("a", &[]);
        c.sources.push(Source {
            title: "Manuel".into(),
            url: "https://exemple.org".into(),
            verified_on: None,
        });
        let catalog = Catalog::new(vec![skill()], vec![c], vec![]).unwrap();
        assert_eq!(catalog.warnings().len(), 1);
        assert_eq!(catalog.warnings()[0].severity(), Severity::Warning);
    }

    #[test]
    fn search_ignores_case_and_accents_and_combines_filters() {
        let mut perms = chapter("permissions", &[]);
        perms.title = "Droits et permissions".into();
        perms.tags = vec!["chmod".into(), "sécurité".into()];
        perms.difficulty = Difficulty::Intermediate;
        perms.estimated_minutes = 45;
        let catalog = Catalog::new(vec![skill()], vec![chapter("a", &[]), perms], vec![]).unwrap();

        let search = |text: &str| -> Vec<String> {
            catalog
                .search(&LibraryQuery {
                    text: text.into(),
                    ..LibraryQuery::default()
                })
                .iter()
                .map(|c| c.id.to_string())
                .collect()
        };
        assert_eq!(search("SECURITE chmod"), vec!["permissions"]);
        assert_eq!(search("shell").len(), 2, "recherche aussi les compétences");
        assert!(search("inexistant").is_empty());

        let filtered = catalog.search(&LibraryQuery {
            difficulty: Some(Difficulty::Intermediate),
            max_minutes: Some(30),
            ..LibraryQuery::default()
        });
        assert!(filtered.is_empty(), "45 min > 30 min");
        assert_eq!(catalog.search(&LibraryQuery::default()).len(), 2);
    }

    #[test]
    fn fold_removes_accents() {
        assert_eq!(fold("Élève À ÇA"), "eleve a ca");
    }
}

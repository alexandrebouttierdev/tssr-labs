//! Chargement du contenu pédagogique.
//!
//! Format (version 1) :
//!
//! ```text
//! content/
//! ├── catalog.json                 compétences et parcours
//! └── chapters/<id>/
//!     ├── chapter.json             métadonnées du chapitre
//!     └── chapter.md               corps en Markdown
//! ```
//!
//! Les fichiers JSON sont lus strictement (champ inconnu = erreur), puis
//! convertis en types du domaine et validés ([`Catalog::new`]). Le contenu
//! livré avec l'application est embarqué dans le binaire : il est disponible
//! hors ligne et ne peut pas être modifié sur le disque.

use std::collections::BTreeMap;

use include_dir::{Dir, include_dir};
use jiff::civil::Date;
use serde::Deserialize;
use tssrlabs_domain::content::{
    Catalog, Chapter, ContentId, ContentIssue, Difficulty, Domain, LearningPath, Skill, Source,
};
use tssrlabs_domain::exercise::{Exercise, ExerciseKind};

/// Version du format de contenu prise en charge.
pub const FORMAT_VERSION: u32 = 1;

static EMBEDDED: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../content");

/// Erreur de chargement du contenu.
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    /// Fichier attendu absent.
    #[error("fichier de contenu manquant : {0}")]
    MissingFile(String),
    /// Fichier non UTF-8.
    #[error("fichier de contenu non UTF-8 : {0}")]
    NotUtf8(String),
    /// JSON invalide ou champ inconnu.
    #[error("JSON invalide dans {file}")]
    Json {
        /// Fichier.
        file: String,
        /// Erreur d'origine.
        #[source]
        source: serde_json::Error,
    },
    /// Version de format non prise en charge.
    #[error("format de contenu {found} non pris en charge (attendu : {FORMAT_VERSION})")]
    UnsupportedFormat {
        /// Version trouvée.
        found: u32,
    },
    /// Valeur invalide (identifiant, domaine, niveau, date).
    #[error("{file} : valeur invalide pour « {field} » : {value}")]
    InvalidValue {
        /// Fichier.
        file: String,
        /// Champ.
        field: &'static str,
        /// Valeur.
        value: String,
    },
    /// Le répertoire d'un chapitre ne correspond pas à son identifiant.
    #[error("le chapitre du répertoire {dir} déclare l'identifiant {id}")]
    ChapterIdMismatch {
        /// Répertoire.
        dir: String,
        /// Identifiant déclaré.
        id: String,
    },
    /// Le catalogue ne passe pas la validation.
    #[error("contenu invalide : {}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ; "))]
    Invalid(Vec<ContentIssue>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogFile {
    format_version: u32,
    skills: Vec<SkillFile>,
    paths: Vec<PathFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillFile {
    id: String,
    name: String,
    domain: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PathFile {
    id: String,
    title: String,
    summary: String,
    chapters: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChapterFile {
    id: String,
    version: String,
    title: String,
    summary: String,
    domain: String,
    difficulty: String,
    estimated_minutes: u16,
    tags: Vec<String>,
    skills: Vec<String>,
    objectives: Vec<String>,
    prerequisites: Vec<String>,
    sources: Vec<SourceFile>,
    authors: Vec<String>,
    license: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExerciseFile {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    title: String,
    prompt: String,
    #[serde(default)]
    choices: Option<Vec<String>>,
    #[serde(default)]
    correct: Option<Vec<usize>>,
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    answers: Option<Vec<String>>,
    hints: Vec<String>,
    explanation: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFile {
    title: String,
    url: String,
    verified_on: Option<String>,
}

/// Fichiers de contenu indexés par chemin relatif (`chapters/x/chapter.json`).
pub type ContentFiles = BTreeMap<String, String>;

/// Charge le contenu embarqué dans le binaire.
pub fn load_embedded() -> Result<Catalog, ContentError> {
    let mut files = ContentFiles::new();
    collect(&EMBEDDED, &mut files)?;
    parse(&files)
}

fn collect(dir: &Dir<'_>, files: &mut ContentFiles) -> Result<(), ContentError> {
    for file in dir.files() {
        let path = file.path().to_string_lossy().replace('\\', "/");
        let text = file
            .contents_utf8()
            .ok_or_else(|| ContentError::NotUtf8(path.clone()))?;
        files.insert(path, text.to_owned());
    }
    for sub in dir.dirs() {
        collect(sub, files)?;
    }
    Ok(())
}

fn read<'a>(files: &'a ContentFiles, path: &str) -> Result<&'a str, ContentError> {
    files
        .get(path)
        .map(String::as_str)
        .ok_or_else(|| ContentError::MissingFile(path.to_owned()))
}

fn json<T: for<'de> Deserialize<'de>>(files: &ContentFiles, path: &str) -> Result<T, ContentError> {
    serde_json::from_str(read(files, path)?).map_err(|source| ContentError::Json {
        file: path.to_owned(),
        source,
    })
}

fn content_id(file: &str, field: &'static str, value: &str) -> Result<ContentId, ContentError> {
    ContentId::parse(value).ok_or_else(|| ContentError::InvalidValue {
        file: file.to_owned(),
        field,
        value: value.to_owned(),
    })
}

fn ids(file: &str, field: &'static str, values: &[String]) -> Result<Vec<ContentId>, ContentError> {
    values.iter().map(|v| content_id(file, field, v)).collect()
}

fn domain(file: &str, value: &str) -> Result<Domain, ContentError> {
    Domain::parse(value).ok_or_else(|| ContentError::InvalidValue {
        file: file.to_owned(),
        field: "domain",
        value: value.to_owned(),
    })
}

/// Analyse un ensemble de fichiers de contenu.
pub fn parse(files: &ContentFiles) -> Result<Catalog, ContentError> {
    let catalog: CatalogFile = json(files, "catalog.json")?;
    if catalog.format_version != FORMAT_VERSION {
        return Err(ContentError::UnsupportedFormat {
            found: catalog.format_version,
        });
    }

    let skills = catalog
        .skills
        .into_iter()
        .map(|s| {
            Ok(Skill {
                id: content_id("catalog.json", "skills.id", &s.id)?,
                name: s.name,
                domain: domain("catalog.json", &s.domain)?,
            })
        })
        .collect::<Result<Vec<_>, ContentError>>()?;

    let paths = catalog
        .paths
        .into_iter()
        .map(|p| {
            Ok(LearningPath {
                id: content_id("catalog.json", "paths.id", &p.id)?,
                title: p.title,
                summary: p.summary,
                chapters: ids("catalog.json", "paths.chapters", &p.chapters)?,
            })
        })
        .collect::<Result<Vec<_>, ContentError>>()?;

    // Chaque répertoire `chapters/<id>/` contenant un chapter.json.
    let chapter_dirs: Vec<String> = files
        .keys()
        .filter_map(|path| {
            path.strip_prefix("chapters/")
                .and_then(|rest| rest.strip_suffix("/chapter.json"))
                .filter(|dir| !dir.contains('/'))
                .map(str::to_owned)
        })
        .collect();

    let mut chapters = Vec::new();
    for dir in chapter_dirs {
        let meta_path = format!("chapters/{dir}/chapter.json");
        let meta: ChapterFile = json(files, &meta_path)?;
        if meta.id != dir {
            return Err(ContentError::ChapterIdMismatch { dir, id: meta.id });
        }
        let body = read(files, &format!("chapters/{dir}/chapter.md"))?.to_owned();
        let sources = meta
            .sources
            .into_iter()
            .map(|s| {
                let verified_on = s
                    .verified_on
                    .map(|d| {
                        d.parse::<Date>().map_err(|_| ContentError::InvalidValue {
                            file: meta_path.clone(),
                            field: "sources.verified_on",
                            value: d,
                        })
                    })
                    .transpose()?;
                Ok(Source {
                    title: s.title,
                    url: s.url,
                    verified_on,
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?;
        chapters.push(Chapter {
            id: content_id(&meta_path, "id", &meta.id)?,
            version: meta.version,
            title: meta.title,
            summary: meta.summary,
            domain: domain(&meta_path, &meta.domain)?,
            difficulty: Difficulty::parse(&meta.difficulty).ok_or_else(|| {
                ContentError::InvalidValue {
                    file: meta_path.clone(),
                    field: "difficulty",
                    value: meta.difficulty.clone(),
                }
            })?,
            estimated_minutes: meta.estimated_minutes,
            tags: meta.tags,
            skills: ids(&meta_path, "skills", &meta.skills)?,
            objectives: meta.objectives,
            prerequisites: ids(&meta_path, "prerequisites", &meta.prerequisites)?,
            body,
            sources,
            authors: meta.authors,
            license: meta.license,
        });
    }

    let exercises = parse_exercises(files, &chapters)?;
    Catalog::new(skills, chapters, paths)
        .and_then(|catalog| catalog.with_exercises(exercises))
        .map_err(ContentError::Invalid)
}

/// Lit les fichiers `exercises.json` (facultatifs) des chapitres.
fn parse_exercises(
    files: &ContentFiles,
    chapters: &[Chapter],
) -> Result<Vec<Exercise>, ContentError> {
    let mut exercises = Vec::new();
    for chapter in chapters {
        let path = format!("chapters/{}/exercises.json", chapter.id);
        if !files.contains_key(&path) {
            continue;
        }
        let entries: Vec<ExerciseFile> = json(files, &path)?;
        for entry in entries {
            exercises.push(exercise(&path, &chapter.id, entry)?);
        }
    }
    Ok(exercises)
}

fn exercise(
    file: &str,
    chapter: &ContentId,
    entry: ExerciseFile,
) -> Result<Exercise, ContentError> {
    let invalid = |field: &'static str, value: &str| ContentError::InvalidValue {
        file: file.to_owned(),
        field,
        value: value.to_owned(),
    };
    // Chaque type n'accepte que ses propres champs.
    let present = [
        ("choices", entry.choices.is_some()),
        ("correct", entry.correct.is_some()),
        ("template", entry.template.is_some()),
        ("output", entry.output.is_some()),
        ("answers", entry.answers.is_some()),
    ];
    let expected: &[&str] = match entry.kind.as_str() {
        "qcm" => &["choices", "correct"],
        "completer" => &["template", "answers"],
        "analyser" => &["output", "answers"],
        "rediger" => &["answers"],
        other => return Err(invalid("type", other)),
    };
    for (field, is_present) in present {
        if is_present != expected.contains(&field) {
            let state = if is_present {
                "champ inattendu pour ce type"
            } else {
                "champ manquant"
            };
            return Err(invalid(
                field,
                &format!("{state} ({} {})", entry.kind, entry.id),
            ));
        }
    }
    let kind = match entry.kind.as_str() {
        "qcm" => ExerciseKind::MultipleChoice {
            choices: entry.choices.unwrap_or_default(),
            correct: entry.correct.unwrap_or_default(),
        },
        "completer" => ExerciseKind::CompleteCommand {
            template: entry.template.unwrap_or_default(),
            answers: entry.answers.unwrap_or_default(),
        },
        "analyser" => ExerciseKind::AnalyzeOutput {
            output: entry.output.unwrap_or_default(),
            answers: entry.answers.unwrap_or_default(),
        },
        _ => ExerciseKind::WriteCommand {
            answers: entry.answers.unwrap_or_default(),
        },
    };
    Ok(Exercise {
        id: content_id(file, "id", &entry.id)?,
        chapter: chapter.clone(),
        title: entry.title,
        prompt: entry.prompt,
        kind,
        hints: entry.hints,
        explanation: entry.explanation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tssrlabs_domain::content::Severity;

    #[test]
    fn embedded_content_is_valid() {
        let catalog = load_embedded().unwrap_or_else(|e| panic!("{e}"));
        assert!(!catalog.paths().is_empty());
        assert!(catalog.chapters().len() >= 3);
        // Seuls des avertissements de sources non vérifiées sont tolérés.
        for warning in catalog.warnings() {
            assert_eq!(warning.severity(), Severity::Warning, "{warning}");
        }
        for chapter in catalog.chapters() {
            assert!(
                !catalog.exercises_for(&chapter.id).is_empty(),
                "{} : au moins un exercice",
                chapter.id
            );
            assert!(
                chapter.body.starts_with("# "),
                "{} : titre Markdown attendu",
                chapter.id
            );
            assert!(
                !chapter.sources.is_empty(),
                "{} : au moins une source",
                chapter.id
            );
        }
    }

    fn minimal() -> ContentFiles {
        let mut files = ContentFiles::new();
        files.insert(
            "catalog.json".into(),
            r#"{"format_version":1,"skills":[{"id":"s","name":"Compétence","domain":"linux"}],
                "paths":[{"id":"p","title":"P","summary":"R","chapters":["c"]}]}"#
                .into(),
        );
        files.insert(
            "chapters/c/chapter.json".into(),
            r#"{"id":"c","version":"1.0.0","title":"T","summary":"R","domain":"linux",
                "difficulty":"debutant","estimated_minutes":10,"tags":[],"skills":["s"],
                "objectives":["O"],"prerequisites":[],
                "sources":[{"title":"S","url":"https://exemple.org","verified_on":"2026-11-04"}],
                "authors":["A"],"license":"MIT"}"#
                .into(),
        );
        files.insert("chapters/c/chapter.md".into(), "# T\n\nCorps.\n".into());
        files
    }

    #[test]
    fn minimal_content_parses() {
        let catalog = parse(&minimal()).unwrap();
        assert_eq!(
            catalog.chapters()[0].sources[0].verified_on,
            Some(jiff::civil::date(2026, 11, 4))
        );
        assert!(catalog.warnings().is_empty());
    }

    #[test]
    fn unknown_fields_and_bad_values_are_rejected() {
        let mut files = minimal();
        let meta = files["chapters/c/chapter.json"]
            .replace("\"license\"", "\"execute\":\"rm -rf /\",\"license\"");
        files.insert("chapters/c/chapter.json".into(), meta);
        assert!(matches!(parse(&files), Err(ContentError::Json { .. })));

        let mut files = minimal();
        let meta = files["chapters/c/chapter.json"].replace("debutant", "expert");
        files.insert("chapters/c/chapter.json".into(), meta);
        assert!(matches!(
            parse(&files),
            Err(ContentError::InvalidValue {
                field: "difficulty",
                ..
            })
        ));
    }

    #[test]
    fn structural_errors_are_reported() {
        let mut files = minimal();
        files.remove("chapters/c/chapter.md");
        assert!(matches!(parse(&files), Err(ContentError::MissingFile(_))));

        let mut files = minimal();
        let catalog = files["catalog.json"].replace("\"format_version\":1", "\"format_version\":2");
        files.insert("catalog.json".into(), catalog);
        assert!(matches!(
            parse(&files),
            Err(ContentError::UnsupportedFormat { found: 2 })
        ));

        let mut files = minimal();
        let meta = files.remove("chapters/c/chapter.json").unwrap();
        files.insert("chapters/autre/chapter.json".into(), meta);
        assert!(matches!(
            parse(&files),
            Err(ContentError::ChapterIdMismatch { .. })
        ));
    }

    #[test]
    fn domain_validation_errors_are_propagated() {
        let mut files = minimal();
        let catalog = files["catalog.json"].replace("[\"c\"]", "[\"c\",\"fantome\"]");
        files.insert("catalog.json".into(), catalog);
        assert!(matches!(parse(&files), Err(ContentError::Invalid(issues)) if issues.len() == 1));
    }

    #[test]
    fn exercises_are_parsed_and_validated() {
        let mut files = minimal();
        files.insert(
            "chapters/c/exercises.json".into(),
            r#"[{"id":"ex-1","type":"qcm","title":"T","prompt":"P","choices":["a","b"],
                 "correct":[1],"hints":[],"explanation":"E"},
                {"id":"ex-2","type":"rediger","title":"T","prompt":"P","answers":["ls -l"],
                 "hints":["H"],"explanation":"E"}]"#
                .into(),
        );
        let catalog = parse(&files).unwrap();
        let id = ContentId::parse("c").unwrap();
        assert_eq!(catalog.exercises_for(&id).len(), 2);
    }

    #[test]
    fn exercise_fields_must_match_their_type() {
        let mut files = minimal();
        files.insert(
            "chapters/c/exercises.json".into(),
            r#"[{"id":"ex-1","type":"rediger","title":"T","prompt":"P","answers":["ls"],
                 "template":"___ ls","hints":[],"explanation":"E"}]"#
                .into(),
        );
        assert!(matches!(
            parse(&files),
            Err(ContentError::InvalidValue {
                field: "template",
                ..
            })
        ));

        files.insert(
            "chapters/c/exercises.json".into(),
            r#"[{"id":"ex-1","type":"executer","title":"T","prompt":"P","hints":[],"explanation":"E"}]"#
                .into(),
        );
        assert!(matches!(
            parse(&files),
            Err(ContentError::InvalidValue { field: "type", .. })
        ));

        files.insert(
            "chapters/c/exercises.json".into(),
            r#"[{"id":"ex-1","type":"qcm","title":"T","prompt":"P","choices":["a","b"],
                 "correct":[5],"hints":[],"explanation":"E"}]"#
                .into(),
        );
        assert!(matches!(parse(&files), Err(ContentError::Invalid(_))));
    }
}

//! Localisation des textes de l'interface (Fluent).
//!
//! Les fichiers `.ftl` sont embarqués dans le binaire. Chaque identifiant
//! utilisé par le code est déclaré dans [`msg`] ; les tests vérifient que
//! tous existent dans chaque langue et qu'aucun message n'est orphelin.
//!
//! Un identifiant manquant n'interrompt jamais l'application : l'identifiant
//! lui-même est affiché et l'anomalie est journalisée.

use std::borrow::Cow;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::langid;

pub use fluent_bundle::FluentValue;

/// Langues disponibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    /// Français (langue initiale).
    #[default]
    French,
}

impl Language {
    fn source(self) -> &'static str {
        match self {
            Self::French => include_str!("../i18n/fr/main.ftl"),
        }
    }
}

macro_rules! messages {
    ($($name:ident = $id:literal),* $(,)?) => {
        /// Identifiants des messages utilisés par l'interface.
        pub mod msg {
            $(
                #[allow(missing_docs)]
                pub const $name: &str = $id;
            )*
            /// Tous les identifiants déclarés.
            pub const ALL: &[&str] = &[$($id),*];
        }
    };
}

messages! {
    APP_TITLE = "app-title",
    APP_SUBTITLE = "app-subtitle",
    SCREEN_HOME = "screen-home",
    SCREEN_DIAGNOSTIC = "screen-diagnostic",
    SCREEN_PROFILES = "screen-profiles",
    HOME_VERSION = "home-version",
    HOME_UPCOMING = "home-upcoming",
    HOME_GREETING = "home-greeting",
    HOME_NO_ACTIVITY = "home-no-activity",
    HOME_EXERCISES = "home-exercises",
    HOME_NO_PROFILE = "home-no-profile",
    HOME_NO_PROFILE_HINT = "home-no-profile-hint",
    LOADING_PROFILES = "loading-profiles",
    PROFILES_EMPTY = "profiles-empty",
    PROFILES_EMPTY_HINT = "profiles-empty-hint",
    PROFILES_COL_NAME = "profiles-col-name",
    PROFILES_COL_CREATED = "profiles-col-created",
    PROFILES_COL_LAST_USED = "profiles-col-last-used",
    PROFILES_COL_STATE = "profiles-col-state",
    PROFILES_ACTIVE = "profiles-active",
    PROFILES_NEVER = "profiles-never",
    PROFILES_LOCAL_NOTE = "profiles-local-note",
    DIALOG_CREATE_TITLE = "dialog-create-title",
    DIALOG_RENAME_TITLE = "dialog-rename-title",
    DIALOG_NAME_LABEL = "dialog-name-label",
    DIALOG_DELETE_TITLE = "dialog-delete-title",
    DIALOG_DELETE_WARNING = "dialog-delete-warning",
    DIALOG_DELETE_BACKUP = "dialog-delete-backup",
    DIALOG_DELETE_PROMPT = "dialog-delete-prompt",
    DIALOG_BUSY = "dialog-busy",
    DIALOG_HINT = "dialog-hint",
    NOTICE_INFO = "notice-info",
    NOTICE_ERROR = "notice-error",
    ERROR_NAME_EMPTY = "error-name-empty",
    ERROR_NAME_TOO_LONG = "error-name-too-long",
    ERROR_NAME_CONTROL = "error-name-control",
    ERROR_NAME_TAKEN = "error-name-taken",
    ERROR_NOT_FOUND = "error-not-found",
    ERROR_CONFIRMATION = "error-confirmation",
    ERROR_STORAGE = "error-storage",
    STATUS_CREATED = "status-created",
    STATUS_RENAMED = "status-renamed",
    STATUS_SELECTED = "status-selected",
    STATUS_DELETED = "status-deleted",
    STATUS_PREFS_ERROR = "status-prefs-error",
    CMD_QUIT = "cmd-quit",
    CMD_HELP = "cmd-help",
    CMD_CLOSE = "cmd-close",
    CMD_NEXT_SCREEN = "cmd-next-screen",
    CMD_PREV_SCREEN = "cmd-prev-screen",
    CMD_CYCLE_THEME = "cmd-cycle-theme",
    CMD_UP = "cmd-up",
    CMD_DOWN = "cmd-down",
    CMD_ACTIVATE = "cmd-activate",
    CMD_NEW_PROFILE = "cmd-new-profile",
    CMD_RENAME_PROFILE = "cmd-rename-profile",
    CMD_DELETE_PROFILE = "cmd-delete-profile",
    KEY_ESC = "key-esc",
    KEY_ENTER = "key-enter",
    KEY_TAB = "key-tab",
    KEY_BACKTAB = "key-backtab",
    KEY_SPACE = "key-space",
    KEY_UP = "key-up",
    KEY_DOWN = "key-down",
    KEY_LEFT = "key-left",
    KEY_RIGHT = "key-right",
    KEY_CTRL = "key-ctrl",
    KEY_ALT = "key-alt",
    KEY_DELETE = "key-delete",
    HELP_TITLE = "help-title",
    HELP_CLOSE = "help-close",
    HELP_LAYOUT_NOTE = "help-layout-note",
    THEME_DARK = "theme-dark",
    THEME_HIGH_CONTRAST = "theme-high-contrast",
    THEME_COLORBLIND = "theme-colorblind",
    THEME_MONOCHROME = "theme-monochrome",
    STATUS_THEME = "status-theme",
    STATUS_PROFILE = "status-profile",
    STATUS_NO_PROFILE = "status-no-profile",
    TOO_SMALL_TITLE = "too-small-title",
    TOO_SMALL_SIZE = "too-small-size",
    TOO_SMALL_HINT = "too-small-hint",
    TOO_SMALL_QUIT = "too-small-quit",
    DIAG_VERSION = "diag-version",
    DIAG_SYSTEM = "diag-system",
    DIAG_TERMINAL = "diag-terminal",
    DIAG_TERM = "diag-term",
    DIAG_COLORS = "diag-colors",
    DIAG_SIZE = "diag-size",
    DIAG_THEME = "diag-theme",
    DIAG_DATA = "diag-data",
    DIAG_CONFIG = "diag-config",
    DIAG_LOGS = "diag-logs",
    DIAG_DATABASE = "diag-database",
    DIAG_NOTE = "diag-note",
    VALUE_UNKNOWN = "value-unknown",
    COLORS_NONE = "colors-none",
    COLORS_BASIC = "colors-basic",
    COLORS_256 = "colors-256",
    COLORS_TRUECOLOR = "colors-truecolor",
    COLORS_FORCED = "colors-forced",
    LAYOUT_FULL = "layout-full",
    LAYOUT_COMPACT = "layout-compact",
    SIZE_VALUE = "size-value",
    SCREEN_EXERCISE = "screen-exercise",
    CMD_OPEN_EXERCISES = "cmd-open-exercises",
    CMD_HINT = "cmd-hint",
    CMD_SHOW_SOLUTION = "cmd-show-solution",
    CMD_NEXT_EXERCISE = "cmd-next-exercise",
    CMD_PREV_EXERCISE = "cmd-prev-exercise",
    CMD_TOGGLE_CHOICE = "cmd-toggle-choice",
    CMD_VALIDATE = "cmd-validate",
    EXERCISE_HEADER = "exercise-header",
    EXERCISE_KIND_QCM = "exercise-kind-qcm",
    EXERCISE_KIND_COMPLETE = "exercise-kind-complete",
    EXERCISE_KIND_ANALYZE = "exercise-kind-analyze",
    EXERCISE_KIND_WRITE = "exercise-kind-write",
    EXERCISE_MULTIPLE = "exercise-multiple",
    EXERCISE_SINGLE = "exercise-single",
    EXERCISE_CHOICE_HINT = "exercise-choice-hint",
    EXERCISE_ANSWER = "exercise-answer",
    EXERCISE_TEXT_CHECK = "exercise-text-check",
    EXERCISE_HINT = "exercise-hint",
    EXERCISE_NO_MORE_HINT = "exercise-no-more-hint",
    EXERCISE_CORRECT = "exercise-correct",
    EXERCISE_INCORRECT = "exercise-incorrect",
    EXERCISE_EMPTY = "exercise-empty",
    EXERCISE_REVEALED = "exercise-revealed",
    EXERCISE_EXPLANATION = "exercise-explanation",
    EXERCISE_NEXT_HINT = "exercise-next-hint",
    EXERCISE_PROGRESS = "exercise-progress",
    EXERCISE_NOT_RECORDED = "exercise-not-recorded",
    EXERCISE_RECORD_ERROR = "exercise-record-error",
    EXERCISE_NONE = "exercise-none",
    READER_EXERCISES = "reader-exercises",
    PATHS_EXERCISES = "paths-exercises",
    SCREEN_PATHS = "screen-paths",
    SCREEN_LIBRARY = "screen-library",
    SCREEN_READER = "screen-reader",
    CMD_OPEN = "cmd-open",
    CMD_BACK = "cmd-back",
    CMD_FOCUS_SEARCH = "cmd-focus-search",
    CMD_CYCLE_DOMAIN = "cmd-cycle-domain",
    CMD_CYCLE_DIFFICULTY = "cmd-cycle-difficulty",
    CMD_CYCLE_DURATION = "cmd-cycle-duration",
    CMD_CLEAR_FILTERS = "cmd-clear-filters",
    CMD_PAGE_UP = "cmd-page-up",
    CMD_PAGE_DOWN = "cmd-page-down",
    CMD_TOP = "cmd-top",
    CMD_BOTTOM = "cmd-bottom",
    KEY_PAGE_UP = "key-page-up",
    KEY_PAGE_DOWN = "key-page-down",
    KEY_HOME = "key-home",
    KEY_END = "key-end",
    DOMAIN_LINUX = "domain-linux",
    DOMAIN_WINDOWS = "domain-windows",
    DOMAIN_NETWORK = "domain-reseau",
    DOMAIN_SYSTEMS = "domain-systeme",
    DOMAIN_SECURITY = "domain-securite",
    DOMAIN_VIRTUALIZATION = "domain-virtualisation",
    DOMAIN_TROUBLESHOOTING = "domain-depannage",
    DOMAIN_AUTOMATION = "domain-automatisation",
    DIFFICULTY_BEGINNER = "difficulty-debutant",
    DIFFICULTY_INTERMEDIATE = "difficulty-intermediaire",
    DIFFICULTY_ADVANCED = "difficulty-avance",
    LIBRARY_SEARCH = "library-search",
    LIBRARY_SEARCH_HINT = "library-search-hint",
    LIBRARY_SEARCH_DONE = "library-search-done",
    LIBRARY_FILTER_DOMAIN = "library-filter-domain",
    LIBRARY_FILTER_DIFFICULTY = "library-filter-difficulty",
    LIBRARY_FILTER_DURATION = "library-filter-duration",
    LIBRARY_ALL = "library-all",
    LIBRARY_ANY_DURATION = "library-any-duration",
    LIBRARY_MAX_MINUTES = "library-max-minutes",
    LIBRARY_RESULTS = "library-results",
    LIBRARY_NO_MATCH = "library-no-match",
    LIBRARY_COL_TITLE = "library-col-title",
    LIBRARY_COL_DOMAIN = "library-col-domain",
    LIBRARY_COL_LEVEL = "library-col-level",
    LIBRARY_COL_DURATION = "library-col-duration",
    MINUTES = "minutes",
    PATHS_SUMMARY = "paths-summary",
    PATHS_EMPTY = "paths-empty",
    PATHS_PREREQUISITES = "paths-prerequisites",
    PATHS_NO_PREREQUISITE = "paths-no-prerequisite",
    READER_OBJECTIVES = "reader-objectives",
    READER_PREREQUISITES = "reader-prerequisites",
    READER_SKILLS = "reader-skills",
    READER_SOURCES = "reader-sources",
    READER_VERIFIED = "reader-verified",
    READER_UNVERIFIED = "reader-unverified",
    READER_META = "reader-meta",
    READER_CREDITS = "reader-credits",
    READER_POSITION = "reader-position",
    READER_MISSING = "reader-missing",
    HOME_CONTENT = "home-content",
    HOME_START_HINT = "home-start-hint",
    DIAG_CONTENT = "diag-content",
    DIAG_CONTENT_VALUE = "diag-content-value",
}

/// Traducteur pour une langue.
pub struct Localizer {
    bundle: FluentBundle<FluentResource>,
}

impl std::fmt::Debug for Localizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Localizer")
            .field("locales", &self.bundle.locales)
            .finish_non_exhaustive()
    }
}

impl Localizer {
    /// Crée le traducteur. Les erreurs de syntaxe éventuelles sont
    /// journalisées ; les messages valides restent utilisables.
    pub fn new(language: Language) -> Self {
        let locale = match language {
            Language::French => langid!("fr"),
        };
        let mut bundle = FluentBundle::new(vec![locale]);
        // Les caractères d'isolation bidirectionnelle s'affichent mal dans
        // un terminal ; inutiles pour des langues de gauche à droite.
        bundle.set_use_isolating(false);

        let resource = match FluentResource::try_new(language.source().to_owned()) {
            Ok(resource) => resource,
            Err((resource, errors)) => {
                tracing::error!(?language, count = errors.len(), "erreurs de syntaxe Fluent");
                resource
            }
        };
        if let Err(errors) = bundle.add_resource(resource) {
            tracing::error!(?language, count = errors.len(), "messages Fluent en double");
        }
        Self { bundle }
    }

    /// Traduit un message sans argument.
    pub fn t(&self, id: &str) -> String {
        self.format(id, None)
    }

    /// Traduit un message avec arguments.
    pub fn t_args(&self, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, value.clone());
        }
        self.format(id, Some(&fluent_args))
    }

    /// Indique si le message existe et possède une valeur.
    pub fn has(&self, id: &str) -> bool {
        self.bundle
            .get_message(id)
            .is_some_and(|message| message.value().is_some())
    }

    fn format(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        let Some(pattern) = self.bundle.get_message(id).and_then(|m| m.value()) else {
            tracing::warn!(id, "message de traduction manquant");
            return id.to_owned();
        };
        let mut errors = Vec::new();
        let text: Cow<'_, str> = self.bundle.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            tracing::warn!(id, ?errors, "erreur de formatage de traduction");
        }
        text.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn french_resource_parses_without_errors() {
        assert!(FluentResource::try_new(Language::French.source().to_owned()).is_ok());
    }

    #[test]
    fn every_declared_message_exists() {
        let i18n = Localizer::new(Language::French);
        let missing: Vec<_> = msg::ALL.iter().filter(|id| !i18n.has(id)).collect();
        assert!(missing.is_empty(), "messages manquants : {missing:?}");
    }

    #[test]
    fn no_orphan_message_in_resource() {
        let declared: Vec<&str> = Language::French
            .source()
            .lines()
            .filter_map(|line| line.split_once(" = ").map(|(id, _)| id.trim()))
            .filter(|id| !id.starts_with('#') && !id.is_empty())
            .collect();
        let orphans: Vec<_> = declared
            .iter()
            .filter(|id| !msg::ALL.contains(id))
            .collect();
        assert!(orphans.is_empty(), "messages non déclarés : {orphans:?}");
    }

    #[test]
    fn arguments_are_interpolated_without_isolation_marks() {
        let i18n = Localizer::new(Language::French);
        let text = i18n.t_args(msg::HOME_GREETING, &[("name", "Alice".into())]);
        assert_eq!(
            i18n.t_args(msg::LIBRARY_RESULTS, &[("count", 0.into())]),
            "Aucun chapitre"
        );
        assert_eq!(
            i18n.t_args(msg::LIBRARY_RESULTS, &[("count", 1.into())]),
            "1 chapitre"
        );
        assert_eq!(
            i18n.t_args(msg::LIBRARY_RESULTS, &[("count", 3.into())]),
            "3 chapitres"
        );
        assert_eq!(text, "Bonjour, Alice");
    }

    #[test]
    fn numbers_are_formatted() {
        let i18n = Localizer::new(Language::French);
        let text = i18n.t_args(
            msg::SIZE_VALUE,
            &[
                ("width", 100.into()),
                ("height", 30.into()),
                ("mode", "x".into()),
            ],
        );
        assert_eq!(text, "100×30 — x");
    }

    #[test]
    fn missing_message_returns_its_id() {
        let i18n = Localizer::new(Language::French);
        assert_eq!(i18n.t("inexistant"), "inexistant");
    }
}

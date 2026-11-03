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
    HOME_VERSION = "home-version",
    HOME_UPCOMING = "home-upcoming",
    CMD_QUIT = "cmd-quit",
    CMD_HELP = "cmd-help",
    CMD_CLOSE = "cmd-close",
    CMD_NEXT_SCREEN = "cmd-next-screen",
    CMD_PREV_SCREEN = "cmd-prev-screen",
    CMD_CYCLE_THEME = "cmd-cycle-theme",
    CMD_UP = "cmd-up",
    CMD_DOWN = "cmd-down",
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
    HELP_TITLE = "help-title",
    HELP_CLOSE = "help-close",
    HELP_LAYOUT_NOTE = "help-layout-note",
    THEME_DARK = "theme-dark",
    THEME_HIGH_CONTRAST = "theme-high-contrast",
    THEME_COLORBLIND = "theme-colorblind",
    THEME_MONOCHROME = "theme-monochrome",
    STATUS_THEME = "status-theme",
    STATUS_COMPACT = "status-compact",
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
        let text = i18n.t_args(msg::HOME_VERSION, &[("version", "1.2.3".into())]);
        assert_eq!(text, "Version 1.2.3 — socle technique");
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

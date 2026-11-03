//! Détection de l'hôte et des capacités du terminal.
//!
//! La détection repose sur les variables d'environnement usuelles. Elle est
//! nécessairement heuristique : la variable [`COLORS_ENV_VAR`] permet à
//! l'utilisateur de corriger le résultat.

use tssrlabs_domain::host::{ColorSupport, DetectionSource, HostInfo, TerminalInfo};

/// Variable imposant le niveau de couleurs : `none`, `16`, `256` ou `truecolor`.
pub const COLORS_ENV_VAR: &str = "TSSRLABS_COLORS";

/// Détecte l'hôte courant à partir de l'environnement du processus.
pub fn detect() -> HostInfo {
    detect_from(
        |name| std::env::var(name).ok(),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

/// Détection à partir d'un environnement fourni (testable).
pub fn detect_from(env: impl Fn(&str) -> Option<String>, os: &str, arch: &str) -> HostInfo {
    // Une variable définie mais vide est traitée comme absente.
    let var = |name: &str| env(name).filter(|value| !value.is_empty());
    let term = var("TERM");
    let program = detect_program(&var);

    let (color, color_source) = match var(COLORS_ENV_VAR)
        .as_deref()
        .and_then(parse_color_override)
    {
        Some(color) => (color, DetectionSource::UserOverride),
        None => {
            if let Some(value) = var(COLORS_ENV_VAR) {
                tracing::warn!(%value, "valeur de {COLORS_ENV_VAR} ignorée (attendu : none, 16, 256, truecolor)");
            }
            (
                detect_color(&var, term.as_deref(), program.as_deref(), os),
                DetectionSource::Detected,
            )
        }
    };

    HostInfo {
        os: os.to_owned(),
        arch: arch.to_owned(),
        terminal: TerminalInfo {
            term,
            program,
            color,
            color_source,
        },
    }
}

fn parse_color_override(value: &str) -> Option<ColorSupport> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" | "0" | "aucune" => Some(ColorSupport::None),
        "16" | "basic" => Some(ColorSupport::Basic),
        "256" => Some(ColorSupport::Ansi256),
        "truecolor" | "24bit" | "24" => Some(ColorSupport::TrueColor),
        _ => None,
    }
}

fn detect_program(var: &impl Fn(&str) -> Option<String>) -> Option<String> {
    if let Some(program) = var("TERM_PROGRAM") {
        return Some(match var("TERM_PROGRAM_VERSION") {
            Some(version) => format!("{program} {version}"),
            None => program,
        });
    }
    let markers = [
        ("WT_SESSION", "Windows Terminal"),
        ("KITTY_WINDOW_ID", "kitty"),
        ("GHOSTTY_RESOURCES_DIR", "Ghostty"),
        ("ALACRITTY_WINDOW_ID", "Alacritty"),
        ("WEZTERM_EXECUTABLE", "WezTerm"),
        ("KONSOLE_VERSION", "Konsole"),
        ("VTE_VERSION", "Terminal VTE (GNOME Terminal, Tilix…)"),
    ];
    markers
        .iter()
        .find(|(name, _)| var(name).is_some())
        .map(|(_, label)| (*label).to_owned())
}

fn detect_color(
    var: &impl Fn(&str) -> Option<String>,
    term: Option<&str>,
    program: Option<&str>,
    os: &str,
) -> ColorSupport {
    // https://no-color.org : présente et non vide => pas de couleur.
    if var("NO_COLOR").is_some() || term == Some("dumb") {
        return ColorSupport::None;
    }
    if let Some(colorterm) = var("COLORTERM") {
        let colorterm = colorterm.to_ascii_lowercase();
        if colorterm == "truecolor" || colorterm == "24bit" {
            return ColorSupport::TrueColor;
        }
    }
    if var("WT_SESSION").is_some() {
        return ColorSupport::TrueColor;
    }
    let truecolor_programs = [
        "iTerm",
        "WezTerm",
        "ghostty",
        "Ghostty",
        "vscode",
        "kitty",
        "Alacritty",
    ];
    if program.is_some_and(|p| truecolor_programs.iter().any(|known| p.starts_with(known))) {
        return ColorSupport::TrueColor;
    }
    match term {
        Some(t)
            if ["kitty", "alacritty", "ghostty", "wezterm", "direct"]
                .iter()
                .any(|k| t.contains(k)) =>
        {
            ColorSupport::TrueColor
        }
        Some(t) if t.contains("256color") => ColorSupport::Ansi256,
        Some(_) => ColorSupport::Basic,
        // Console Windows 10+ sans TERM : séquences VT 24 bits prises en charge.
        None if os == "windows" => ColorSupport::TrueColor,
        None => ColorSupport::Basic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn detect_with(vars: &[(&str, &str)], os: &str) -> HostInfo {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        detect_from(|name| map.get(name).cloned(), os, "x86_64")
    }

    fn color(vars: &[(&str, &str)]) -> ColorSupport {
        detect_with(vars, "linux").terminal.color
    }

    #[test]
    fn no_color_and_dumb_disable_colors() {
        assert_eq!(
            color(&[("NO_COLOR", "1"), ("COLORTERM", "truecolor")]),
            ColorSupport::None
        );
        assert_eq!(color(&[("TERM", "dumb")]), ColorSupport::None);
    }

    #[test]
    fn empty_no_color_is_ignored() {
        assert_eq!(
            color(&[("NO_COLOR", ""), ("TERM", "xterm-256color")]),
            ColorSupport::Ansi256
        );
    }

    #[test]
    fn colorterm_and_known_terminals_enable_truecolor() {
        assert_eq!(
            color(&[("COLORTERM", "truecolor")]),
            ColorSupport::TrueColor
        );
        assert_eq!(color(&[("TERM", "xterm-kitty")]), ColorSupport::TrueColor);
        assert_eq!(
            color(&[("TERM_PROGRAM", "WezTerm")]),
            ColorSupport::TrueColor
        );
        assert_eq!(color(&[("WT_SESSION", "abc")]), ColorSupport::TrueColor);
    }

    #[test]
    fn term_fallbacks() {
        assert_eq!(color(&[("TERM", "xterm-256color")]), ColorSupport::Ansi256);
        assert_eq!(color(&[("TERM", "linux")]), ColorSupport::Basic);
        assert_eq!(color(&[]), ColorSupport::Basic);
        assert_eq!(
            detect_with(&[], "windows").terminal.color,
            ColorSupport::TrueColor
        );
    }

    #[test]
    fn user_override_wins_and_is_reported() {
        let host = detect_with(&[(COLORS_ENV_VAR, "256"), ("NO_COLOR", "1")], "linux");
        assert_eq!(host.terminal.color, ColorSupport::Ansi256);
        assert_eq!(host.terminal.color_source, DetectionSource::UserOverride);
    }

    #[test]
    fn invalid_override_falls_back_to_detection() {
        let host = detect_with(
            &[(COLORS_ENV_VAR, "arc-en-ciel"), ("TERM", "xterm-256color")],
            "linux",
        );
        assert_eq!(host.terminal.color, ColorSupport::Ansi256);
        assert_eq!(host.terminal.color_source, DetectionSource::Detected);
    }

    #[test]
    fn program_detection() {
        let host = detect_with(
            &[
                ("TERM_PROGRAM", "iTerm.app"),
                ("TERM_PROGRAM_VERSION", "3.5"),
            ],
            "macos",
        );
        assert_eq!(host.terminal.program.as_deref(), Some("iTerm.app 3.5"));
        let host = detect_with(&[("WT_SESSION", "x")], "windows");
        assert_eq!(host.terminal.program.as_deref(), Some("Windows Terminal"));
        assert_eq!(detect_with(&[], "linux").terminal.program, None);
    }

    #[test]
    fn os_and_arch_are_kept() {
        let host = detect_with(&[], "macos");
        assert_eq!((host.os.as_str(), host.arch.as_str()), ("macos", "x86_64"));
    }
}

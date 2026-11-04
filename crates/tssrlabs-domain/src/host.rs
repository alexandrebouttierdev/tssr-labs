//! Description de l'hôte et du terminal, telle que constatée au démarrage.
//!
//! Les fonctionnalités sont activées selon ces capacités constatées, et non
//! selon l'OS déclaré (voir `docs/ARCHITECTURE.md`, section Capabilities).

/// Niveau de prise en charge des couleurs par le terminal.
///
/// L'ordre est significatif : un niveau supérieur inclut les inférieurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorSupport {
    /// Aucune couleur (terminal `dumb` ou préférence `NO_COLOR`) : seuls les
    /// attributs de texte (gras, inversé, souligné) sont utilisés.
    None,
    /// 16 couleurs ANSI.
    Basic,
    /// Palette de 256 couleurs.
    Ansi256,
    /// Couleurs 24 bits.
    TrueColor,
}

impl ColorSupport {
    /// Indique si des couleurs peuvent être utilisées.
    pub fn has_colors(self) -> bool {
        self > Self::None
    }
}

/// Origine de la valeur [`TerminalInfo::color`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionSource {
    /// Déduite de l'environnement du terminal.
    Detected,
    /// Imposée par l'utilisateur.
    UserOverride,
}

/// Informations sur le terminal hôte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalInfo {
    /// Valeur de `TERM`, si définie.
    pub term: Option<String>,
    /// Émulateur identifié (ex. « WezTerm », « Windows Terminal »), si connu.
    pub program: Option<String>,
    /// Prise en charge des couleurs.
    pub color: ColorSupport,
    /// Origine de [`Self::color`].
    pub color_source: DetectionSource,
}

/// Informations sur l'hôte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostInfo {
    /// Système d'exploitation (`linux`, `macos`, `windows`…).
    pub os: String,
    /// Architecture CPU (`x86_64`, `aarch64`…).
    pub arch: String,
    /// Terminal.
    pub terminal: TerminalInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_levels_are_ordered() {
        assert!(ColorSupport::TrueColor > ColorSupport::Ansi256);
        assert!(ColorSupport::Ansi256 > ColorSupport::Basic);
        assert!(!ColorSupport::None.has_colors());
        assert!(ColorSupport::Basic.has_colors());
    }
}

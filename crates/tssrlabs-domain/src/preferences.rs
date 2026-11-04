//! Préférences d'un profil.

/// Thème d'interface choisi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ThemeChoice {
    /// Sombre (défaut).
    #[default]
    Dark,
    /// Contraste renforcé.
    HighContrast,
    /// Palette adaptée aux daltonismes courants.
    Colorblind,
    /// Sans couleur.
    Monochrome,
}

impl ThemeChoice {
    /// Ordre de parcours.
    pub const ALL: [Self; 4] = [
        Self::Dark,
        Self::HighContrast,
        Self::Colorblind,
        Self::Monochrome,
    ];

    /// Thème suivant dans l'ordre de parcours.
    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|k| *k == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    /// Identifiant stable, utilisé pour le stockage.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::HighContrast => "high-contrast",
            Self::Colorblind => "colorblind",
            Self::Monochrome => "monochrome",
        }
    }

    /// Lit un identifiant stocké ; `None` si inconnu.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == value)
    }
}

/// Préférences d'un profil. Les valeurs inconnues ou absentes prennent
/// leur valeur par défaut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Preferences {
    /// Thème d'interface.
    pub theme: ThemeChoice,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_round_trip() {
        for theme in ThemeChoice::ALL {
            assert_eq!(ThemeChoice::parse(theme.as_str()), Some(theme));
        }
        assert_eq!(ThemeChoice::parse("arc-en-ciel"), None);
    }

    #[test]
    fn next_cycles() {
        let mut theme = ThemeChoice::Dark;
        for _ in 0..ThemeChoice::ALL.len() {
            theme = theme.next();
        }
        assert_eq!(theme, ThemeChoice::Dark);
    }
}

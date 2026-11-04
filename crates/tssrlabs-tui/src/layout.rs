//! Modes d'affichage selon la taille du terminal.
//!
//! Référence : 100 colonnes × 30 lignes. En dessous, mode compact ; sous le
//! minimum, un message explique la situation au lieu de tronquer l'interface.

/// Largeur de référence de l'affichage complet.
pub const FULL_WIDTH: u16 = 100;
/// Hauteur de référence de l'affichage complet.
pub const FULL_HEIGHT: u16 = 30;
/// Largeur minimale utilisable (mode compact).
pub const MIN_WIDTH: u16 = 60;
/// Hauteur minimale utilisable (mode compact).
pub const MIN_HEIGHT: u16 = 16;

/// Mode d'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Toute l'interface.
    Full,
    /// Éléments secondaires réduits, actions essentielles conservées.
    Compact,
    /// Trop petit pour un affichage sûr : explication seulement.
    TooSmall,
}

impl LayoutMode {
    /// Mode correspondant à une taille de terminal.
    pub fn for_size(width: u16, height: u16) -> Self {
        if width >= FULL_WIDTH && height >= FULL_HEIGHT {
            Self::Full
        } else if width >= MIN_WIDTH && height >= MIN_HEIGHT {
            Self::Compact
        } else {
            Self::TooSmall
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds() {
        assert_eq!(LayoutMode::for_size(100, 30), LayoutMode::Full);
        assert_eq!(LayoutMode::for_size(200, 60), LayoutMode::Full);
        assert_eq!(LayoutMode::for_size(99, 30), LayoutMode::Compact);
        assert_eq!(LayoutMode::for_size(100, 29), LayoutMode::Compact);
        assert_eq!(LayoutMode::for_size(60, 16), LayoutMode::Compact);
        assert_eq!(LayoutMode::for_size(59, 40), LayoutMode::TooSmall);
        assert_eq!(LayoutMode::for_size(120, 15), LayoutMode::TooSmall);
    }
}

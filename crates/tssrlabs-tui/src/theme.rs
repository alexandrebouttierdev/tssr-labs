//! Thèmes de l'interface.
//!
//! Les palettes sont définies en couleurs 24 bits puis converties selon les
//! capacités constatées du terminal (256 ou 16 couleurs). Sans couleur
//! (`NO_COLOR`, terminal `dumb`), le thème monochrome s'appuie uniquement sur
//! les attributs de texte.
//!
//! La couleur n'est jamais le seul porteur d'information : chaque état a
//! aussi une étiquette textuelle.

use ratatui::style::{Color, Modifier, Style};
use tssrlabs_domain::host::ColorSupport;

use crate::i18n::msg;

pub use tssrlabs_domain::preferences::ThemeChoice;

/// Identifiant du nom localisé d'un thème.
pub fn theme_label_id(choice: ThemeChoice) -> &'static str {
    match choice {
        ThemeChoice::Dark => msg::THEME_DARK,
        ThemeChoice::HighContrast => msg::THEME_HIGH_CONTRAST,
        ThemeChoice::Colorblind => msg::THEME_COLORBLIND,
        ThemeChoice::Monochrome => msg::THEME_MONOCHROME,
    }
}

/// Styles sémantiques utilisés par les écrans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Thème effectivement appliqué (monochrome si le terminal n'a pas de couleur).
    pub kind: ThemeChoice,
    /// Texte courant sur le fond principal.
    pub base: Style,
    /// Barres d'en-tête et de pied de page.
    pub bar: Style,
    /// Texte secondaire.
    pub muted: Style,
    /// Mise en avant (touches, éléments actifs).
    pub accent: Style,
    /// Titres.
    pub title: Style,
    /// Bordures.
    pub border: Style,
    /// Élément sélectionné.
    pub selection: Style,
    /// Onglet actif.
    pub tab_active: Style,
    /// Succès.
    pub success: Style,
    /// Avertissement.
    pub warning: Style,
    /// Erreur.
    pub danger: Style,
}

type Rgb = (u8, u8, u8);

struct Palette {
    bg: Rgb,
    bar: Rgb,
    fg: Rgb,
    muted: Rgb,
    border: Rgb,
    accent: Rgb,
    selection_bg: Rgb,
    selection_fg: Rgb,
    success: Rgb,
    warning: Rgb,
    danger: Rgb,
}

const DARK: Palette = Palette {
    bg: (0x12, 0x15, 0x1b),
    bar: (0x1b, 0x20, 0x29),
    fg: (0xd5, 0xd9, 0xe0),
    muted: (0x80, 0x88, 0x94),
    border: (0x3a, 0x41, 0x4c),
    accent: (0x5c, 0xa4, 0xe8),
    selection_bg: (0x24, 0x3b, 0x57),
    selection_fg: (0xee, 0xf2, 0xf6),
    success: (0x55, 0xba, 0x80),
    warning: (0xdb, 0xa8, 0x48),
    danger: (0xe5, 0x6b, 0x62),
};

const HIGH_CONTRAST: Palette = Palette {
    bg: (0x00, 0x00, 0x00),
    bar: (0x00, 0x00, 0x00),
    fg: (0xff, 0xff, 0xff),
    muted: (0xd0, 0xd0, 0xd0),
    border: (0xff, 0xff, 0xff),
    accent: (0xff, 0xd7, 0x00),
    selection_bg: (0xff, 0xd7, 0x00),
    selection_fg: (0x00, 0x00, 0x00),
    success: (0x5f, 0xff, 0x87),
    warning: (0xff, 0xd7, 0x00),
    danger: (0xff, 0x6b, 0x6b),
};

/// Okabe-Ito : bleu ciel, orange, jaune et vermillon restent distinguables
/// pour les deutéranopies, protanopies et tritanopies courantes.
const COLORBLIND: Palette = Palette {
    bg: (0x12, 0x15, 0x1b),
    bar: (0x1b, 0x20, 0x29),
    fg: (0xd5, 0xd9, 0xe0),
    muted: (0x8a, 0x91, 0x9c),
    border: (0x3a, 0x41, 0x4c),
    accent: (0xe6, 0x9f, 0x00),
    selection_bg: (0x3d, 0x33, 0x14),
    selection_fg: (0xf6, 0xf2, 0xe6),
    success: (0x56, 0xb4, 0xe9),
    warning: (0xf0, 0xe4, 0x42),
    danger: (0xd5, 0x5e, 0x00),
};

impl Theme {
    /// Construit le thème demandé, adapté aux capacités du terminal.
    pub fn resolve(kind: ThemeChoice, support: ColorSupport) -> Self {
        let palette = match kind {
            _ if !support.has_colors() => return Self::monochrome(),
            ThemeChoice::Monochrome => return Self::monochrome(),
            ThemeChoice::Dark => &DARK,
            ThemeChoice::HighContrast => &HIGH_CONTRAST,
            ThemeChoice::Colorblind => &COLORBLIND,
        };
        let c = |rgb: Rgb| convert(rgb, support);
        let bold = Modifier::BOLD;
        Self {
            kind,
            base: Style::new().fg(c(palette.fg)).bg(c(palette.bg)),
            bar: Style::new().fg(c(palette.fg)).bg(c(palette.bar)),
            muted: Style::new().fg(c(palette.muted)),
            accent: Style::new().fg(c(palette.accent)).add_modifier(bold),
            title: Style::new().fg(c(palette.fg)).add_modifier(bold),
            border: Style::new().fg(c(palette.border)),
            selection: Style::new()
                .fg(c(palette.selection_fg))
                .bg(c(palette.selection_bg))
                .add_modifier(bold),
            tab_active: Style::new()
                .fg(c(palette.accent))
                .add_modifier(bold | Modifier::UNDERLINED),
            success: Style::new().fg(c(palette.success)),
            warning: Style::new().fg(c(palette.warning)),
            danger: Style::new().fg(c(palette.danger)),
        }
    }

    fn monochrome() -> Self {
        let bold = Style::new().add_modifier(Modifier::BOLD);
        Self {
            kind: ThemeChoice::Monochrome,
            base: Style::new(),
            bar: Style::new(),
            muted: Style::new(),
            accent: bold,
            title: bold,
            border: Style::new(),
            selection: Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD),
            tab_active: Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD),
            success: bold,
            warning: bold,
            danger: bold,
        }
    }
}

fn convert(rgb: Rgb, support: ColorSupport) -> Color {
    match support {
        ColorSupport::TrueColor => Color::Rgb(rgb.0, rgb.1, rgb.2),
        ColorSupport::Ansi256 => Color::Indexed(rgb_to_ansi256(rgb)),
        ColorSupport::Basic => rgb_to_ansi16(rgb),
        ColorSupport::None => Color::Reset,
    }
}

fn distance(a: Rgb, b: Rgb) -> u32 {
    let d = |x: u8, y: u8| (i32::from(x) - i32::from(y)).unsigned_abs().pow(2);
    d(a.0, b.0) + d(a.1, b.1) + d(a.2, b.2)
}

/// Couleur la plus proche dans la palette xterm 256 (cube 6×6×6 ou gris).
fn rgb_to_ansi256(rgb: Rgb) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    // Indice (0..=5) du niveau le plus proche.
    let nearest = |v: u8| -> u8 {
        (0u8..6)
            .min_by_key(|i| v.abs_diff(LEVELS[usize::from(*i)]))
            .unwrap_or(0)
    };
    let (r, g, b) = (nearest(rgb.0), nearest(rgb.1), nearest(rgb.2));
    let level = |i: u8| LEVELS[usize::from(i)];
    let cube_rgb = (level(r), level(g), level(b));
    // Au plus 16 + 36×5 + 6×5 + 5 = 231 : tient dans un u8.
    let cube_index = 16 + 36 * r + 6 * g + b;

    let average = (u16::from(rgb.0) + u16::from(rgb.1) + u16::from(rgb.2)) / 3;
    // Au plus 23 : tient dans un u8.
    let gray_step = u8::try_from((average.saturating_sub(8) / 10).min(23)).unwrap_or(23);
    let gray_value = 8 + 10 * gray_step;
    let gray_index = 232 + gray_step;

    if distance(rgb, (gray_value, gray_value, gray_value)) < distance(rgb, cube_rgb) {
        gray_index
    } else {
        cube_index
    }
}

/// Couleur la plus proche parmi les 16 couleurs ANSI (valeurs xterm usuelles).
fn rgb_to_ansi16(rgb: Rgb) -> Color {
    const TABLE: [(Rgb, Color); 16] = [
        ((0, 0, 0), Color::Black),
        ((205, 0, 0), Color::Red),
        ((0, 205, 0), Color::Green),
        ((205, 205, 0), Color::Yellow),
        ((0, 0, 238), Color::Blue),
        ((205, 0, 205), Color::Magenta),
        ((0, 205, 205), Color::Cyan),
        ((229, 229, 229), Color::Gray),
        ((127, 127, 127), Color::DarkGray),
        ((255, 0, 0), Color::LightRed),
        ((0, 255, 0), Color::LightGreen),
        ((255, 255, 0), Color::LightYellow),
        ((92, 92, 255), Color::LightBlue),
        ((255, 0, 255), Color::LightMagenta),
        ((0, 255, 255), Color::LightCyan),
        ((255, 255, 255), Color::White),
    ];
    TABLE
        .iter()
        .min_by_key(|(candidate, _)| distance(rgb, *candidate))
        .map_or(Color::Reset, |(_, color)| *color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_support_forces_monochrome() {
        let theme = Theme::resolve(ThemeChoice::Dark, ColorSupport::None);
        assert_eq!(theme.kind, ThemeChoice::Monochrome);
        assert_eq!(theme.base, Style::new());
    }

    #[test]
    fn colors_follow_terminal_capabilities() {
        let fg_of = |support| Theme::resolve(ThemeChoice::Dark, support).base.fg;
        assert!(matches!(
            fg_of(ColorSupport::TrueColor),
            Some(Color::Rgb(..))
        ));
        assert!(matches!(
            fg_of(ColorSupport::Ansi256),
            Some(Color::Indexed(_))
        ));
        assert!(!matches!(
            fg_of(ColorSupport::Basic),
            Some(Color::Rgb(..) | Color::Indexed(_))
        ));
    }

    #[test]
    fn ansi256_conversion_matches_known_values() {
        assert_eq!(rgb_to_ansi256((0, 0, 0)), 16);
        assert_eq!(rgb_to_ansi256((255, 255, 255)), 231);
        assert_eq!(rgb_to_ansi256((255, 0, 0)), 196);
        // Gris moyen : rampe de gris plutôt que le cube.
        assert_eq!(rgb_to_ansi256((128, 128, 128)), 244);
    }

    #[test]
    fn ansi16_conversion_picks_nearest() {
        assert_eq!(rgb_to_ansi16((10, 10, 10)), Color::Black);
        assert_eq!(rgb_to_ansi16((250, 250, 250)), Color::White);
        assert_eq!(rgb_to_ansi16((250, 10, 10)), Color::LightRed);
        assert_eq!(rgb_to_ansi16((200, 10, 10)), Color::Red);
    }
}

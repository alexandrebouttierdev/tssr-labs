//! Utilitaires de texte pour l'affichage en cellules de terminal.

use jiff::Timestamp;
use jiff::tz::TimeZone;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Date et heure dans le fuseau du système (« 03/11/2026 15:42 »).
pub fn format_local(timestamp: Timestamp) -> String {
    format_in(timestamp, &TimeZone::system())
}

/// Date et heure dans un fuseau donné.
pub fn format_in(timestamp: Timestamp, tz: &TimeZone) -> String {
    timestamp
        .to_zoned(tz.clone())
        .strftime("%d/%m/%Y %H:%M")
        .to_string()
}

/// Raccourcit `text` à `max_width` colonnes en remplaçant le milieu par « … ».
///
/// Adapté aux chemins : le début (racine) et la fin (nom) restent visibles.
pub fn ellipsize_middle(text: &str, max_width: usize) -> String {
    if text.width() <= max_width {
        return text.to_owned();
    }
    if max_width == 0 {
        return String::new();
    }
    let budget = max_width - 1; // une colonne pour « … »
    let head_budget = budget.div_ceil(2);
    let tail_budget = budget - head_budget;

    let mut head = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > head_budget {
            break;
        }
        head.push(c);
        used += w;
    }

    let mut tail: Vec<char> = Vec::new();
    let mut used = 0;
    for c in text.chars().rev() {
        let w = c.width().unwrap_or(0);
        if used + w > tail_budget {
            break;
        }
        tail.push(c);
        used += w;
    }
    tail.reverse();

    format!("{head}…{}", tail.into_iter().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_use_french_day_month_order() {
        let ts = Timestamp::from_second(1_762_183_320).unwrap(); // 2025-11-03 15:22 UTC
        assert_eq!(format_in(ts, &TimeZone::UTC), "03/11/2025 15:22");
    }

    #[test]
    fn short_text_is_unchanged() {
        assert_eq!(ellipsize_middle("/home/a", 20), "/home/a");
    }

    #[test]
    fn long_text_keeps_both_ends_and_fits() {
        let out = ellipsize_middle("/home/utilisateur/.local/share/tssrlabs/logs", 20);
        assert_eq!(out.width(), 20);
        assert!(out.starts_with("/home/"));
        assert!(out.ends_with("/logs"));
        assert!(out.contains('…'));
    }

    #[test]
    fn wide_characters_are_measured_in_columns() {
        let out = ellipsize_middle("日本語のパスです", 7);
        assert!(out.width() <= 7, "{out}");
    }

    #[test]
    fn degenerate_widths() {
        assert_eq!(ellipsize_middle("abcdef", 0), "");
        assert_eq!(ellipsize_middle("abcdef", 1), "…");
    }
}

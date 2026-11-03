//! Éléments d'interface partagés entre les écrans.

pub mod chrome;
pub mod help;
pub mod too_small;

use ratatui::layout::Rect;

/// Rectangle de `width`×`height` centré dans `area` (réduit si nécessaire).
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

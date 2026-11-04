//! Champ de saisie d'une ligne.
//!
//! Gère l'édition de base (caractères, effacement, déplacement du curseur)
//! en tenant compte des caractères larges. Les raccourcis globaux (lettres)
//! ne sont pas interprétés pendant la saisie.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthChar;

/// Longueur maximale acceptée par le champ (garde-fou ; la validation
/// métier impose ses propres limites).
pub const MAX_INPUT_CHARS: usize = 128;

/// Contenu et curseur d'un champ de saisie.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    chars: Vec<char>,
    /// Position du curseur, en caractères.
    cursor: usize,
}

impl TextInput {
    /// Champ prérempli, curseur en fin de texte.
    pub fn with_value(value: &str) -> Self {
        let chars: Vec<char> = value.chars().take(MAX_INPUT_CHARS).collect();
        Self {
            cursor: chars.len(),
            chars,
        }
    }

    /// Texte saisi.
    pub fn value(&self) -> String {
        self.chars.iter().collect()
    }

    /// Position du curseur (en caractères).
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Applique une touche d'édition ; renvoie `false` si elle n'est pas gérée.
    pub fn handle(&mut self, key: &KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if self.chars.len() < MAX_INPUT_CHARS && !c.is_control() {
                    self.chars.insert(self.cursor, c);
                    self.cursor += 1;
                }
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.chars.remove(self.cursor);
            }
            KeyCode::Delete if self.cursor < self.chars.len() => {
                self.chars.remove(self.cursor);
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.chars.len()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.chars.len(),
            KeyCode::Backspace | KeyCode::Delete => {}
            _ => return false,
        }
        true
    }

    /// Dessine le champ sur une ligne et place le curseur du terminal.
    /// Le texte défile horizontalement pour garder le curseur visible.
    pub fn render(&self, frame: &mut Frame, area: Rect, style: Style, show_cursor: bool) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let width = usize::from(area.width).saturating_sub(1);
        let char_width = |c: &char| c.width().unwrap_or(0);

        // Premier caractère visible : on recule depuis le curseur tant que
        // la largeur tient dans le champ.
        let mut start = self.cursor;
        let mut used = 0;
        while start > 0 && used + char_width(&self.chars[start - 1]) <= width {
            start -= 1;
            used += char_width(&self.chars[start]);
        }
        let mut visible = String::new();
        let mut visible_width = 0;
        for c in &self.chars[start..] {
            if visible_width + char_width(c) > width {
                break;
            }
            visible.push(*c);
            visible_width += char_width(c);
        }
        let padding = usize::from(area.width).saturating_sub(visible_width);
        visible.push_str(&" ".repeat(padding));

        frame.render_widget(Paragraph::new(Line::styled(visible, style)), area);
        if show_cursor {
            let offset: usize = self.chars[start..self.cursor].iter().map(char_width).sum();
            let x = area.x
                + u16::try_from(offset)
                    .unwrap_or(area.width)
                    .min(area.width - 1);
            frame.set_cursor_position(Position::new(x, area.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn type_text(input: &mut TextInput, text: &str) {
        for c in text.chars() {
            input.handle(&key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn typing_and_editing() {
        let mut input = TextInput::default();
        type_text(&mut input, "Alce");
        input.handle(&key(KeyCode::Left));
        input.handle(&key(KeyCode::Left));
        type_text(&mut input, "i");
        assert_eq!(input.value(), "Alice");
        input.handle(&key(KeyCode::Home));
        input.handle(&key(KeyCode::Delete));
        assert_eq!(input.value(), "lice");
        input.handle(&key(KeyCode::End));
        input.handle(&key(KeyCode::Backspace));
        assert_eq!(input.value(), "lic");
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn letters_are_text_not_shortcuts_and_ctrl_is_ignored() {
        let mut input = TextInput::default();
        type_text(&mut input, "qtnrs?");
        assert_eq!(input.value(), "qtnrs?");
        let ctrl = KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            ..key(KeyCode::Char('c'))
        };
        assert!(
            !input.handle(&ctrl),
            "Ctrl+lettre est laissé à l'application"
        );
        assert_eq!(input.value(), "qtnrs?");
    }

    #[test]
    fn edits_at_bounds_are_harmless_and_length_is_capped() {
        let mut input = TextInput::default();
        input.handle(&key(KeyCode::Backspace));
        input.handle(&key(KeyCode::Delete));
        input.handle(&key(KeyCode::Left));
        assert_eq!(input, TextInput::default());
        type_text(&mut input, &"x".repeat(MAX_INPUT_CHARS + 10));
        assert_eq!(input.value().chars().count(), MAX_INPUT_CHARS);
    }

    #[test]
    fn prefill_places_cursor_at_end() {
        let input = TextInput::with_value("Élodie");
        assert_eq!(input.cursor(), 6);
        assert!(!input.clone().handle(&key(KeyCode::Enter)));
    }
}

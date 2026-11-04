//! Rendu Markdown pour le terminal.
//!
//! Sous-ensemble pris en charge : titres, paragraphes, listes (puces et
//! numérotées, imbriquées), blocs de code, code en ligne, emphase, gras,
//! citations, liens (texte suivi de l'adresse) et séparateurs. Le HTML brut
//! n'est jamais interprété.
//!
//! Sécurité : tout texte est nettoyé de ses caractères de contrôle avant
//! affichage ; un contenu importé ne peut pas envoyer de séquence
//! d'échappement au terminal.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::text::sanitize;
use crate::theme::Theme;

struct Renderer<'t> {
    theme: &'t Theme,
    lines: Vec<Line<'static>>,
    current: Vec<Span<'static>>,
    styles: Vec<Style>,
    /// Compteurs des listes ouvertes (`None` : liste à puces).
    lists: Vec<Option<u64>>,
    quote_depth: usize,
    in_code_block: bool,
    link_url: Option<String>,
    /// Préfixe de la première ligne d'un élément de liste.
    pending_item: Option<String>,
}

impl<'t> Renderer<'t> {
    fn new(theme: &'t Theme) -> Self {
        Self {
            theme,
            lines: Vec::new(),
            current: Vec::new(),
            styles: vec![theme.base],
            lists: Vec::new(),
            quote_depth: 0,
            in_code_block: false,
            link_url: None,
            pending_item: None,
        }
    }

    fn style(&self) -> Style {
        self.styles.last().copied().unwrap_or(self.theme.base)
    }

    fn push_style(&mut self, patch: Style) {
        let style = self.style().patch(patch);
        self.styles.push(style);
    }

    fn pop_style(&mut self) {
        if self.styles.len() > 1 {
            self.styles.pop();
        }
    }

    fn indent(&self) -> String {
        let quote = "│ ".repeat(self.quote_depth);
        let list = "  ".repeat(self.lists.len().saturating_sub(1));
        format!("{quote}{list}")
    }

    fn text(&mut self, text: &str) {
        let style = self.style();
        self.current.push(Span::styled(sanitize(text), style));
    }

    fn flush(&mut self) {
        if self.current.is_empty() && self.pending_item.is_none() {
            return;
        }
        let mut spans = Vec::new();
        let indent = self.indent();
        if !indent.is_empty() {
            spans.push(Span::styled(indent, self.theme.muted));
        }
        if let Some(bullet) = self.pending_item.take() {
            spans.push(Span::styled(bullet, self.theme.accent));
        } else if !self.lists.is_empty() {
            // Suite d'un élément de liste : alignement sous le texte.
            spans.push(Span::raw("  "));
        }
        spans.append(&mut self.current);
        self.lines.push(Line::from(spans));
    }

    fn blank(&mut self) {
        if self.lines.last().is_some_and(|l| l.width() > 0) {
            self.lines.push(Line::from(""));
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.flush();
                self.blank();
                let style = match level {
                    HeadingLevel::H1 => self.theme.title.add_modifier(Modifier::UNDERLINED),
                    HeadingLevel::H2 => self.theme.accent,
                    _ => self.theme.title,
                };
                self.styles.push(style);
            }
            Tag::Paragraph => self.flush(),
            Tag::BlockQuote(_) => {
                self.flush();
                self.quote_depth += 1;
                self.push_style(self.theme.muted);
            }
            Tag::CodeBlock(kind) => {
                self.flush();
                self.in_code_block = true;
                if let CodeBlockKind::Fenced(lang) = kind
                    && !lang.is_empty()
                {
                    self.lines.push(Line::styled(
                        format!("  [{}]", sanitize(&lang)),
                        self.theme.muted,
                    ));
                }
            }
            Tag::List(start) => {
                self.flush();
                self.lists.push(start);
            }
            Tag::Item => {
                self.flush();
                let bullet = match self.lists.last_mut() {
                    Some(Some(n)) => {
                        let bullet = format!("{n}. ");
                        *n += 1;
                        bullet
                    }
                    _ => "• ".to_owned(),
                };
                self.pending_item = Some(bullet);
            }
            Tag::Emphasis => self.push_style(Style::new().add_modifier(Modifier::ITALIC)),
            Tag::Strong => self.push_style(Style::new().add_modifier(Modifier::BOLD)),
            Tag::Strikethrough => self.push_style(Style::new().add_modifier(Modifier::CROSSED_OUT)),
            Tag::Link { dest_url, .. } => {
                self.link_url = Some(dest_url.to_string());
                self.push_style(Style::new().add_modifier(Modifier::UNDERLINED));
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                self.flush();
                self.pop_style();
                self.lines.push(Line::from(""));
            }
            TagEnd::Paragraph => {
                self.flush();
                if self.lists.is_empty() {
                    self.lines.push(Line::from(""));
                }
            }
            TagEnd::BlockQuote(_) => {
                self.flush();
                self.quote_depth = self.quote_depth.saturating_sub(1);
                self.pop_style();
                self.blank();
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                self.lines.push(Line::from(""));
            }
            TagEnd::List(_) => {
                self.flush();
                self.lists.pop();
                if self.lists.is_empty() {
                    self.blank();
                }
            }
            TagEnd::Item => self.flush(),
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => self.pop_style(),
            TagEnd::Link => {
                self.pop_style();
                if let Some(url) = self.link_url.take() {
                    self.current.push(Span::styled(
                        format!(" ({})", sanitize(&url)),
                        self.theme.muted,
                    ));
                }
            }
            _ => {}
        }
    }

    fn code_block_text(&mut self, text: &str) {
        let style = self.theme.accent.remove_modifier(Modifier::BOLD);
        for line in text.lines() {
            self.lines.push(Line::from(vec![
                Span::styled("  │ ", self.theme.muted),
                Span::styled(sanitize(line), style),
            ]));
        }
    }

    fn render(mut self, markdown: &str) -> Vec<Line<'static>> {
        for event in Parser::new(markdown) {
            match event {
                Event::Start(tag) => self.start(tag),
                Event::End(tag) => self.end(tag),
                Event::Text(text) if self.in_code_block => self.code_block_text(&text),
                Event::Text(text) => self.text(&text),
                Event::Code(code) => {
                    let style = self.theme.accent.remove_modifier(Modifier::BOLD);
                    self.current.push(Span::styled(sanitize(&code), style));
                }
                Event::SoftBreak => self.text(" "),
                Event::HardBreak => self.flush(),
                Event::Rule => {
                    self.flush();
                    self.lines
                        .push(Line::styled("─".repeat(40), self.theme.muted));
                    self.lines.push(Line::from(""));
                }
                // HTML brut, notes, mathématiques : jamais interprétés.
                Event::Html(raw) | Event::InlineHtml(raw) => self.text(&raw),
                _ => {}
            }
        }
        self.flush();
        while self.lines.last().is_some_and(|l| l.width() == 0) {
            self.lines.pop();
        }
        self.lines
    }
}

/// Convertit du Markdown en lignes stylées pour Ratatui.
pub fn render(markdown: &str, theme: &Theme) -> Vec<Line<'static>> {
    Renderer::new(theme).render(markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeChoice;
    use tssrlabs_domain::host::ColorSupport;

    fn text_of(markdown: &str) -> Vec<String> {
        let theme = Theme::resolve(ThemeChoice::Dark, ColorSupport::TrueColor);
        render(markdown, &theme)
            .iter()
            .map(|line| line.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    #[test]
    fn headings_paragraphs_and_lists() {
        let lines = text_of(
            "# Titre\n\nUn *texte* avec `ls -l`.\n\n- un\n- deux\n\n1. premier\n2. second\n",
        );
        assert_eq!(
            lines,
            vec![
                "Titre",
                "",
                "Un texte avec ls -l.",
                "",
                "• un",
                "• deux",
                "",
                "1. premier",
                "2. second",
            ]
        );
    }

    #[test]
    fn nested_lists_are_indented() {
        let lines = text_of("- a\n  - b\n- c\n");
        assert_eq!(lines, vec!["• a", "  • b", "• c"]);
    }

    #[test]
    fn code_blocks_keep_lines_and_mark_them() {
        let lines = text_of("```\ncd /etc\nls -l\n```\n");
        assert_eq!(lines, vec!["  │ cd /etc", "  │ ls -l"]);
    }

    #[test]
    fn links_show_their_address() {
        let lines = text_of("Voir [le manuel](https://exemple.org/man).");
        assert_eq!(lines, vec!["Voir le manuel (https://exemple.org/man)."]);
    }

    #[test]
    fn control_sequences_are_neutralized() {
        let lines = text_of(
            "Attaque \u{1b}]52;c;cGF3bmVk\u{7} et `\u{1b}[2J`\n\n```\n\u{1b}[31mrouge\n```\n",
        );
        for line in &lines {
            assert!(!line.chars().any(|c| c.is_control()), "{line:?}");
        }
    }

    #[test]
    fn raw_html_is_shown_as_text_not_interpreted() {
        let lines = text_of("<script>alert(1)</script>\n");
        assert!(lines.iter().any(|l| l.contains("<script>")), "{lines:?}");
    }
}

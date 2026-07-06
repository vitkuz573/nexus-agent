//! Markdown rendering: convert markdown text into styled `Line`s for ratatui.
//!
//! Supports: bold, italic, inline code, headers (h1-h3), bullet/numbered
//! lists, code blocks (with background tint), and paragraphs. Uses
//! `pulldown-cmark` as the parser.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::tui::theme::Theme;

const CODE_BLOCK_BG: Color = Color::Rgb(20, 20, 30);

/// Render markdown text into a list of styled lines.
pub fn render_markdown(input: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(input, options);

    let mut state = RenderState::default();
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Start(tag) => {
                let is_code = matches!(tag, Tag::CodeBlock(_));
                state.start_tag(tag, theme);
                if is_code {
                    in_code_block = true;
                }
            }
            Event::End(end) => {
                let was_code = matches!(end, TagEnd::CodeBlock);
                if was_code {
                    in_code_block = false;
                    state.flush_paragraph(theme);
                }
                state.end_tag(end);
            }
            Event::Text(text) => {
                let style = if in_code_block {
                    Style::default().fg(theme.fg).bg(CODE_BLOCK_BG)
                } else {
                    state.current_style(theme)
                };
                state.push_text(&text, style);
            }
            Event::Code(code) => {
                let style = Style::default().fg(theme.accent).bg(CODE_BLOCK_BG);
                state.push_styled(format!(" {code} "), style);
            }
            Event::SoftBreak => state.push_text(" ", state.current_style(theme)),
            Event::HardBreak => state.flush_paragraph(theme),
            Event::Rule => {
                state.flush_paragraph(theme);
                state.push_styled("─".repeat(40), Style::default().fg(theme.border));
                state.flush_paragraph(theme);
            }
            _ => {}
        }
    }

    state.finish(theme)
}

#[derive(Debug, Default)]
struct RenderState {
    /// Stack of active modifiers (bold/italic) and their merge with theme.
    style_stack: Vec<Style>,
    /// Current paragraph being assembled.
    current: Vec<Span<'static>>,
    /// Output paragraphs (each is a line; multi-line paragraphs have multiple).
    output: Vec<Line<'static>>,
    /// Indentation level (for nested lists).
    indent: usize,
    /// Prefix to prepend to the next line (e.g. "  • " for bullets).
    pending_prefix: Option<String>,
    /// True while we're inside a list item.
    in_list_item: bool,
    /// Last heading level seen, used to size the heading style.
    last_heading: Option<u8>,
    /// Last code block fence language, for code block labels.
    last_code_lang: Option<String>,
}

impl RenderState {
    fn start_tag(&mut self, tag: Tag, theme: &Theme) {
        match tag {
            Tag::Paragraph => {
                self.flush_paragraph(theme);
            }
            Tag::Heading { level, .. } => {
                self.flush_paragraph(theme);
                let h = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                self.last_heading = Some(h);
            }
            Tag::BlockQuote(_kind) => {
                self.flush_paragraph(theme);
                self.indent += 2;
            }
            Tag::CodeBlock(kind) => {
                self.flush_paragraph(theme);
                let lang = match kind {
                    CodeBlockKind::Indented => None,
                    CodeBlockKind::Fenced(s) => {
                        if s.is_empty() {
                            None
                        } else {
                            Some(s.to_string())
                        }
                    }
                };
                self.last_code_lang = lang;
            }
            Tag::List(_) => {
                self.flush_paragraph(theme);
            }
            Tag::Item => {
                self.flush_paragraph(theme);
                self.in_list_item = true;
                let prefix = "  ".repeat(self.indent) + "• ";
                self.pending_prefix = Some(prefix);
                self.indent += 1;
            }
            Tag::Emphasis => {
                let s = self.current_style(theme).add_modifier(Modifier::ITALIC);
                self.style_stack.push(s);
            }
            Tag::Strong => {
                let s = self.current_style(theme).add_modifier(Modifier::BOLD);
                self.style_stack.push(s);
            }
            Tag::Strikethrough => {
                let s = self.current_style(theme).add_modifier(Modifier::CROSSED_OUT);
                self.style_stack.push(s);
            }
            Tag::Link { dest_url, .. } => {
                let s = self.current_style(theme).fg(theme.accent);
                self.style_stack.push(s);
                self.pending_prefix = Some(format!("["));
                let link_style = Style::default()
                    .fg(theme.dim)
                    .add_modifier(Modifier::UNDERLINED);
                self.push_text_styled(dest_url.as_ref(), link_style);
                self.push_text("]", self.current_style(theme));
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, end: TagEnd) {
        match end {
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                self.style_stack.pop();
            }
            TagEnd::Heading(_) => {
                self.last_heading = None;
                let theme = Theme::dark();
                self.flush_paragraph(&theme);
            }
            TagEnd::BlockQuote(_) => {
                self.indent = self.indent.saturating_sub(2);
            }
            TagEnd::Item => {
                let theme = Theme::dark();
                self.flush_paragraph(&theme);
                self.indent = self.indent.saturating_sub(1);
                self.in_list_item = false;
                self.pending_prefix = None;
            }
            TagEnd::List(_) => {}
            TagEnd::Paragraph => {
                let theme = Theme::dark();
                self.flush_paragraph(&theme);
            }
            TagEnd::CodeBlock => {
                self.last_code_lang = None;
            }
            TagEnd::Link => {
                self.pending_prefix = None;
            }
            _ => {}
        }
    }

    fn current_style(&self, theme: &Theme) -> Style {
        let mut s = Style::default().fg(theme.fg);
        if let Some(&h) = self.last_heading.as_ref() {
            s = s.fg(theme.accent).add_modifier(Modifier::BOLD);
            if h == 1 {
                s = s.add_modifier(Modifier::UNDERLINED);
            }
        }
        for st in &self.style_stack {
            s = s.patch(*st);
        }
        s
    }

    fn push_text(&mut self, text: &str, style: Style) {
        if let Some(prefix) = self.pending_prefix.take() {
            self.current.push(Span::styled(prefix, style));
        }
        for (i, line) in text.split('\n').enumerate() {
            if i > 0 {
                let theme = Theme::dark();
                self.flush_paragraph(&theme);
                if let Some(prefix) = self.pending_prefix.take() {
                    self.current.push(Span::styled(prefix, style));
                }
            }
            if !line.is_empty() {
                self.current.push(Span::styled(line.to_string(), style));
            }
        }
    }

    fn push_text_styled(&mut self, text: &str, style: Style) {
        self.current.push(Span::styled(text.to_string(), style));
    }

    fn push_styled(&mut self, text: String, style: Style) {
        self.current.push(Span::styled(text, style));
    }

    fn flush_paragraph(&mut self, theme: &Theme) {
        if self.current.is_empty() {
            // Add a blank line spacer after flushed paragraphs for spacing.
            if !self.output.is_empty() {
                self.output.push(Line::from(""));
            }
            return;
        }
        let spans = std::mem::take(&mut self.current);
        self.output.push(Line::from(spans));
        self.output.push(Line::from(""));
        let _ = theme;
    }

    fn finish(mut self, theme: &Theme) -> Vec<Line<'static>> {
        self.flush_paragraph(theme);
        if let Some(last) = self.output.last() {
            if last.spans.is_empty() {
                self.output.pop();
            }
        }
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::theme::Theme;

    fn t() -> Theme {
        Theme::dark()
    }

    #[test]
    fn test_plain_text() {
        let lines = render_markdown("hello world", &t());
        let joined: String = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect::<Vec<_>>()
            .join("");
        assert!(joined.contains("hello world"));
    }

    #[test]
    fn test_bold() {
        let lines = render_markdown("**bold text**", &t());
        let has_bold = lines.iter().any(|l| {
            l.spans.iter().any(|s| {
                s.content.contains("bold text") && s.style.add_modifier.contains(Modifier::BOLD)
            })
        });
        assert!(has_bold, "expected bold styling");
    }

    #[test]
    fn test_inline_code() {
        let lines = render_markdown("`foo()`", &t());
        let has_code = lines.iter().any(|l| l.spans.iter().any(|s| s.content.contains("foo()")));
        assert!(has_code);
    }

    #[test]
    fn test_list() {
        let lines = render_markdown("- one\n- two", &t());
        let joined: String = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("• one"), "expected '• one', got: {joined}");
        assert!(joined.contains("• two"));
    }

    #[test]
    fn test_heading() {
        let lines = render_markdown("# Title\n\nbody", &t());
        let joined: String = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("Title"));
    }

    #[test]
    fn test_emphasis() {
        let lines = render_markdown("*italic*", &t());
        let has_italic = lines.iter().any(|l| {
            l.spans.iter().any(|s| {
                s.content.contains("italic") && s.style.add_modifier.contains(Modifier::ITALIC)
            })
        });
        assert!(has_italic);
    }

    #[test]
    fn test_code_block() {
        let lines = render_markdown("```rust\nfn main() {}\n```", &t());
        let joined: String = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("fn main"));
    }
}

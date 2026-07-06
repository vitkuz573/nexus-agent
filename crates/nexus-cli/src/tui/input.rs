//! Multi-line input with history, editing, and slash-command awareness.
//!
//! Tracks a single text buffer that may span multiple lines. Cursor is a
//! `(line, col)` position. History holds previous inputs for Up/Down recall.

#![allow(dead_code)]

use ratatui::text::{Line, Span};

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InputBuffer {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    draft: Option<String>,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            history: Vec::new(),
            history_index: None,
            draft: None,
        }
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|l| l.is_empty())
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        let row = row.min(self.lines.len().saturating_sub(1));
        let col = col.min(self.lines[row].chars().count());
        self.cursor_row = row;
        self.cursor_col = col;
    }

    pub fn insert_char(&mut self, c: char) {
        let row = self.lines[self.cursor_row].clone();
        let byte_col = char_to_byte_col(&row, self.cursor_col);
        let mut new_row = row;
        new_row.insert(byte_col, c);
        self.lines[self.cursor_row] = new_row;
        self.cursor_col += 1;
    }

    pub fn insert_newline(&mut self) {
        let current = self.lines[self.cursor_row].clone();
        let byte_col = char_to_byte_col(&current, self.cursor_col);
        let (left, right) = current.split_at(byte_col);
        self.lines[self.cursor_row] = left.to_string();
        self.lines.insert(self.cursor_row + 1, right.to_string());
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let row = self.lines[self.cursor_row].clone();
            let byte_col = char_to_byte_col(&row, self.cursor_col - 1);
            let mut new_row = row;
            new_row.remove(byte_col);
            self.lines[self.cursor_row] = new_row;
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].chars().count();
            self.lines[self.cursor_row].push_str(&current);
        }
    }

    pub fn delete_forward(&mut self) {
        let row = self.lines[self.cursor_row].clone();
        let char_count = row.chars().count();
        if self.cursor_col < char_count {
            let byte_col = char_to_byte_col(&row, self.cursor_col);
            let mut new_row = row;
            new_row.remove(byte_col);
            self.lines[self.cursor_row] = new_row;
        } else if self.cursor_row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].chars().count();
        }
    }

    pub fn move_right(&mut self) {
        let char_count = self.lines[self.cursor_row].chars().count();
        if self.cursor_col < char_count {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].chars().count());
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].chars().count());
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_row].chars().count();
    }

    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.draft = None;
    }

    pub fn submit(&mut self) -> String {
        let text = self.text();
        if !text.trim().is_empty() {
            self.history.push(text.clone());
        }
        self.history_index = None;
        self.draft = None;
        self.clear();
        text
    }

    pub fn history_recall_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let new_index = match self.history_index {
            None => self.history.len() - 1,
            Some(0) => return,
            Some(i) => i - 1,
        };
        if self.history_index.is_none() {
            self.draft = Some(self.text());
        }
        self.history_index = Some(new_index);
        self.replace_with_history();
    }

    pub fn history_recall_next(&mut self) {
        let Some(idx) = self.history_index else {
            return;
        };
        if idx + 1 >= self.history.len() {
            self.history_index = None;
            if let Some(draft) = self.draft.take() {
                self.set_text(&draft);
            }
            return;
        }
        self.history_index = Some(idx + 1);
        self.replace_with_history();
    }

    fn replace_with_history(&mut self) {
        let Some(idx) = self.history_index else {
            return;
        };
        let text = self.history[idx].clone();
        self.set_text(&text);
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines[self.cursor_row].chars().count();
    }

    pub fn set_text(&mut self, text: &str) {
        self.lines = text.split('\n').map(String::from).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.lines[self.cursor_row].chars().count();
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn as_display_lines(&self, width: usize) -> Vec<Line<'static>> {
        let width = width.max(1);
        let mut out = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            if line.is_empty() {
                out.push(Line::from(Span::raw(" ")));
                continue;
            }
            let chars: Vec<char> = line.chars().collect();
            for chunk in chars.chunks(width) {
                out.push(Line::from(Span::raw(chunk.iter().collect::<String>())));
            }
            let _ = i;
        }
        out
    }

    pub fn slash_command(&self) -> Option<&str> {
        let first = self.lines.first()?.trim();
        if first.starts_with('/') {
            Some(first)
        } else {
            None
        }
    }
}

fn char_to_byte_col(s: &str, char_col: usize) -> usize {
    s.char_indices()
        .nth(char_col)
        .map(|(b, _)| b)
        .unwrap_or_else(|| s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_input() {
        let mut b = InputBuffer::new();
        b.insert_char('h');
        b.insert_char('i');
        assert_eq!(b.text(), "hi");
        assert_eq!(b.cursor(), (0, 2));
    }

    #[test]
    fn test_newline() {
        let mut b = InputBuffer::new();
        b.insert_char('a');
        b.insert_newline();
        b.insert_char('b');
        assert_eq!(b.text(), "a\nb");
        assert_eq!(b.cursor(), (1, 1));
    }

    #[test]
    fn test_backspace() {
        let mut b = InputBuffer::new();
        b.insert_char('a');
        b.insert_char('b');
        b.backspace();
        assert_eq!(b.text(), "a");
        assert_eq!(b.cursor(), (0, 1));
    }

    #[test]
    fn test_backspace_merges_lines() {
        let mut b = InputBuffer::new();
        b.insert_char('a');
        b.insert_newline();
        b.insert_char('b');
        assert_eq!(b.text(), "a\nb");
        b.move_home();
        b.backspace();
        assert_eq!(b.text(), "ab");
        assert_eq!(b.cursor(), (0, 1));
    }

    #[test]
    fn test_history() {
        let mut b = InputBuffer::new();
        b.insert_char('h');
        b.insert_char('i');
        b.submit();
        assert!(b.is_empty());
        b.history_recall_prev();
        assert_eq!(b.text(), "hi");
    }

    #[test]
    fn test_slash_command() {
        let mut b = InputBuffer::new();
        b.set_text("/help");
        assert_eq!(b.slash_command(), Some("/help"));
    }
}

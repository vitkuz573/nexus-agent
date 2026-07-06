//! Files panel: project tree on the left side.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::tui::state::TuiState;
use crate::tui::theme::Theme;

pub fn render_files(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme, focused: bool) {
    let border_color = if focused { theme.accent } else { theme.border };
    let title = if focused { " 📁 Files (focused) " } else { " 📁 Files " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let items: Vec<ListItem> = state
        .file_tree
        .iter()
        .map(|entry| {
            let indent = "  ".repeat(entry.depth);
            let marker = if entry.is_dir {
                if entry.expanded { "▾ " } else { "▸ " }
            } else {
                "  "
            };
            let style = if entry.is_dir {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(Line::from(Span::styled(
                format!("{indent}{marker}{}", entry.path),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

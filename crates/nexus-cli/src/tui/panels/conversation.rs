//! Conversation panel: streaming messages, user/assistant roles,
//! tool call events inline.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Wrap};
use ratatui::Frame;

use crate::tui::state::{MessageRole, ToolStatus, TuiState};
use crate::tui::theme::Theme;

pub fn render_conversation(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme, focused: bool) {
    let border_color = if focused { theme.accent } else { theme.border };
    let title = if focused { " 💬 Conversation (focused) " } else { " 💬 Conversation " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let items = build_items(state, theme);

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(theme.selection).add_modifier(Modifier::BOLD));

    let mut state_buf = ratatui::widgets::ListState::default();
    let total = list_height_hint(state);
    let viewport_height = area.height.saturating_sub(2) as usize;
    let offset = state.scroll.conversation_offset.min(total.saturating_sub(viewport_height));
    state_buf.select(Some(offset));

    f.render_stateful_widget(list, area, &mut state_buf);
    let _ = Wrap::default();
}

fn build_items(state: &TuiState, theme: &Theme) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    for msg in &state.messages {
        items.extend(message_to_items(msg, theme));
    }
    for tool in &state.tool_events {
        items.push(tool_item(tool, theme));
    }
    items
}

fn message_to_items(msg: &crate::tui::state::Message, theme: &Theme) -> Vec<ListItem<'static>> {
    let (prefix, color) = match msg.role {
        MessageRole::User => ("> ", theme.user),
        MessageRole::Assistant => ("⚡ ", theme.assistant),
        MessageRole::System => ("* ", theme.dim),
        MessageRole::Tool => ("⚙ ", theme.tool),
        MessageRole::Error => ("✗ ", theme.error),
    };

    let mut items = Vec::new();
    for (i, line) in msg.content.lines().enumerate() {
        let spans = if i == 0 {
            vec![
                Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(line.to_string(), Style::default().fg(color_for_role(msg.role, theme))),
            ]
        } else {
            vec![Span::styled(
                format!("  {line}"),
                Style::default().fg(color_for_role(msg.role, theme)),
            )]
        };
        items.push(ListItem::new(Line::from(spans)));
    }
    if msg.streaming {
        items.push(ListItem::new(Line::from(Span::styled(
            "  ▌",
            Style::default().fg(theme.accent),
        ))));
    }
    items.push(ListItem::new(Line::from("")));
    items
}

fn color_for_role(role: MessageRole, theme: &Theme) -> Color {
    match role {
        MessageRole::User => theme.user,
        MessageRole::Assistant => theme.fg,
        MessageRole::System => theme.dim,
        MessageRole::Tool => theme.tool,
        MessageRole::Error => theme.error,
    }
}

use ratatui::style::Color;

fn tool_item(tool: &crate::tui::state::ToolEvent, theme: &Theme) -> ListItem<'static> {
    let (marker, color) = match tool.status {
        ToolStatus::Running => ("⟳", theme.warning),
        ToolStatus::Ok => ("✓", theme.success),
        ToolStatus::Failed => ("✗", theme.error),
    };
    let line = Line::from(vec![
        Span::styled(format!("  {marker} "), Style::default().fg(color)),
        Span::styled(tool.name.clone(), Style::default().fg(theme.tool).add_modifier(Modifier::BOLD)),
        Span::styled("  ", Style::default()),
        Span::styled(
            truncate(&tool.arguments, 60),
            Style::default().fg(theme.dim),
        ),
    ]);
    ListItem::new(line)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

fn list_height_hint(state: &TuiState) -> usize {
    let mut total = 0;
    for msg in &state.messages {
        total += msg.content.lines().count().max(1) + 1;
        if msg.streaming {
            total += 1;
        }
    }
    total += state.tool_events.len();
    total
}

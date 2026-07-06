//! Input panel: multi-line input box with cursor, slash-command hints.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::input::InputBuffer;
use crate::tui::state::{AgentStatus, TuiState};
use crate::tui::theme::Theme;

pub fn render_input(f: &mut Frame, area: Rect, input: &InputBuffer, state: &TuiState, theme: &Theme, focused: bool) {
    let border_color = if focused { theme.accent } else { theme.border };
    let title = match state.status {
        AgentStatus::Idle => " ▸ Input (Enter to send, Shift+Enter for newline) ",
        AgentStatus::Thinking => " ⏳ Thinking… ",
        AgentStatus::Streaming => " ⚡ Streaming response… ",
        AgentStatus::Running => " ⚙ Running tool… ",
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let text: String = if input.is_empty() {
        let hint = match state.status {
            AgentStatus::Idle => "Type a message…",
            _ => "",
        };
        hint.to_string()
    } else {
        input.text()
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(theme.fg))
        .block(block);

    f.render_widget(paragraph, area);

    if focused {
        let (row, col) = input.cursor();
        let cursor_x = area.x.saturating_add(1).saturating_add(col as u16);
        let cursor_y = area.y.saturating_add(1).saturating_add(row as u16);
        let max_x = area.x.saturating_add(area.width).saturating_sub(1);
        let max_y = area.y.saturating_add(area.height).saturating_sub(1);
        if cursor_x <= max_x && cursor_y <= max_y {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }

    if let Some(cmd) = input.slash_command() {
        render_slash_hint(f, area, cmd, theme);
    }
}

fn render_slash_hint(f: &mut Frame, area: Rect, cmd: &str, theme: &Theme) {
    let hint = match cmd {
        "/help" | "/?" => Some("show help"),
        "/clear" | "/c" | "/cls" => Some("clear conversation"),
        "/tools" => Some("list tools"),
        "/theme" => Some("cycle theme"),
        "/model" | "/m" => Some("show model"),
        "/providers" => Some("list providers"),
        "/verify" => Some("re-run verifier"),
        "/history" => Some("show history"),
        "/quit" | "/q" | "/exit" => Some("exit"),
        _ => None,
    };
    if let Some(h) = hint {
        let hint_y = area.y.saturating_add(area.height).saturating_sub(1);
        let hint_w = area.width.saturating_sub(4);
        let hint_area = Rect::new(area.x.saturating_add(2), hint_y, hint_w, 1);
        let line = Line::from(vec![
            Span::styled(cmd, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" — ", Style::default().fg(theme.dim)),
            Span::styled(h, Style::default().fg(theme.dim)),
        ]);
        f.render_widget(Paragraph::new(line), hint_area);
    }
}

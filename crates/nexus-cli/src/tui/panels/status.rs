//! Bottom status bar: hotkeys, current mode, contextual hints.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::state::{AgentStatus, Focus, TuiState};
use crate::tui::theme::Theme;

pub fn render_status(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme) {
    let mode_label = match state.focus {
        Focus::Input => "INPUT",
        Focus::Conversation => "CONVERSATION",
        Focus::Sidebar => "SIDEBAR",
    };
    let mode_color = match state.focus {
        Focus::Input => theme.accent,
        Focus::Conversation => theme.assistant,
        Focus::Sidebar => theme.tool,
    };

    let agent_label = match state.status {
        AgentStatus::Idle => "ready",
        AgentStatus::Thinking => "thinking",
        AgentStatus::Streaming => "streaming",
        AgentStatus::Running => "running",
    };

    let hotkeys = vec![
        ("F1", "Help"),
        ("Tab", "Focus"),
        ("Shift+Tab", "Back"),
        ("↑/↓", "History"),
        ("PgUp/PgDn", "Scroll"),
        ("Ctrl+C", "Quit"),
    ];

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled(
        format!(" [{mode_label}] "),
        Style::default().bg(mode_color).fg(theme.bg).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        format!(" {agent_label} "),
        Style::default().fg(theme.dim),
    ));
    spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));

    for (i, (key, action)) in hotkeys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        }
        spans.push(Span::styled(format!(" {key} "), Style::default().fg(theme.accent)));
        spans.push(Span::styled(*action, Style::default().fg(theme.dim)));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg));
    f.render_widget(paragraph, area);
}

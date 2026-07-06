//! Header panel: a single compact line with model, status, tokens.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::state::{AgentStatus, TuiState};
use crate::tui::theme::Theme;

pub fn render_header(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme) {
    let spinner = spinner_frame(state.spinner_frame);
    let status_label = match state.status {
        AgentStatus::Idle => Span::styled(" ● idle ", Style::default().fg(theme.dim)),
        AgentStatus::Thinking => {
            Span::styled(format!(" {spinner} thinking "), Style::default().fg(theme.warning))
        }
        AgentStatus::Streaming => {
            Span::styled(format!(" {spinner} streaming "), Style::default().fg(theme.accent))
        }
        AgentStatus::Running => {
            Span::styled(format!(" {spinner} running "), Style::default().fg(theme.success))
        }
    };

    let token_pct = if state.max_tokens > 0 {
        (state.tokens_used as f32 / state.max_tokens as f32 * 100.0) as u32
    } else {
        0
    };

    let line = Line::from(vec![
        Span::styled(" ⚡ Nexus ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("│ ", Style::default().fg(theme.border)),
        Span::styled(&state.model, Style::default().fg(theme.fg)),
        Span::styled(" @ ", Style::default().fg(theme.dim)),
        Span::styled(&state.provider, Style::default().fg(theme.dim)),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        status_label,
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            format!("round {}/{}", state.round, state.max_rounds),
            Style::default().fg(theme.fg),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            format!("tok {}k/{}k", state.tokens_used / 1000, state.max_tokens / 1000),
            Style::default().fg(if token_pct > 80 { theme.warning } else { theme.dim }),
        ),
    ]);

    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}

fn spinner_frame(frame: usize) -> &'static str {
    const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    FRAMES[frame % FRAMES.len()]
}

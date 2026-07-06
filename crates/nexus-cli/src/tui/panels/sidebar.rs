//! Sidebar panel: file tree (left) and cognitive state + verifications (right).

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::tui::state::TuiState;
use crate::tui::theme::Theme;

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme, focused: bool) {
    let border_color = if focused { theme.accent } else { theme.border };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_files(f, chunks[0], state, theme, border_color);
    render_cognitive(f, chunks[1], state, theme, border_color);
}

fn render_files(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme, border_color: Color) {
    let title = " 📁 Files ";
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

fn render_cognitive(f: &mut Frame, area: Rect, state: &TuiState, theme: &Theme, border_color: Color) {
    let block = Block::default()
        .title(" 🧠 Cognitive ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        " ─── Thinking ─── ",
        Style::default().fg(theme.accent),
    )));

    if let Some(chain) = &state.thought_chain {
        for node in chain.all_nodes() {
            let confidence_pct = (node.confidence * 100.0) as u32;
            let conf_color = if confidence_pct >= 80 {
                theme.success
            } else if confidence_pct >= 50 {
                theme.warning
            } else {
                theme.error
            };
            let type_label = format!("{:?}", node.thought_type);
            lines.push(Line::from(vec![
                Span::styled(format!("  {:>12} ", type_label), Style::default().fg(theme.dim)),
                Span::styled(
                    format!("[{confidence_pct}%]"),
                    Style::default().fg(conf_color),
                ),
            ]));
            for content_line in node.content.lines().take(3) {
                lines.push(Line::from(Span::styled(
                    format!("    {content_line}"),
                    Style::default().fg(theme.fg),
                )));
            }
        }
    } else {
        lines.push(Line::from(Span::styled("    (idle)", Style::default().fg(theme.dim))));
    }

    if !state.verifications.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " ─── Verifications ─── ",
            Style::default().fg(theme.accent),
        )));
        for v in state.verifications.iter().rev().take(3) {
            let score_pct = (v.score * 100.0) as u32;
            let color = if v.passed {
                theme.success
            } else if v.score >= 0.5 {
                theme.warning
            } else {
                theme.error
            };
            lines.push(Line::from(Span::styled(
                format!("  Score: {score_pct}%"),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));
            for check in &v.checks {
                let mark = if check.passed { "✓" } else { "✗" };
                let c = if check.passed { theme.success } else { theme.error };
                lines.push(Line::from(vec![
                    Span::styled(format!("    {mark} "), Style::default().fg(c)),
                    Span::styled(check.name.clone(), Style::default().fg(theme.fg)),
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

use ratatui::style::Color;

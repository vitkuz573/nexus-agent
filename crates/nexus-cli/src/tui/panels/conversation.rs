//! Conversation panel: block-based rendering with auto-scroll.
//!
//! Each event (user message, assistant message, tool call, thinking) is
//! rendered as an independent unit with its own background tint and a
//! top border showing the role, time, and duration. Blocks never share
//! a selection and never bleed into each other — the only thing that
//! scrolls is the cumulative position of the blocks.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::state::{Block as ConvBlock, BlockKind, ToolCallStatus, TuiState};
use crate::tui::theme::Theme;

pub fn render_conversation(
    f: &mut Frame,
    area: Rect,
    state: &TuiState,
    theme: &Theme,
    focused: bool,
) {
    let border_color = if focused { theme.accent } else { theme.border };
    let title = if focused { " 💬 Conversation (focused) " } else { " 💬 Conversation " };

    let outer = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    if state.blocks.is_empty() {
        let hint = Paragraph::new(Line::from(Span::styled(
            "  Type a message below to start. Try /help for commands.",
            Style::default().fg(theme.dim),
        )));
        f.render_widget(hint, inner);
        return;
    }

    let width = inner.width.saturating_sub(2) as usize;

    // For each block compute its wrapped lines and total height
    let mut block_heights: Vec<usize> = Vec::with_capacity(state.blocks.len());
    for blk in &state.blocks {
        let lines = render_block_lines(blk, theme, width);
        let height = lines.len() + 2; // top + bottom border
        block_heights.push(height);
    }

    let total: usize = block_heights.iter().sum::<usize>() + state.blocks.len().saturating_sub(1);
    let viewport = inner.height as usize;

    let scroll = compute_scroll(total, viewport, state.scroll.auto_follow, state.scroll.manual_offset);

    // Build the flat visible line list with bg colors per line
    let mut visible_lines: Vec<Line<'static>> = Vec::new();
    let mut consumed = 0usize;
    for (blk, h) in state.blocks.iter().zip(block_heights.iter()) {
        let blk_end = consumed + h;
        if blk_end <= scroll {
            consumed = blk_end + 1; // +1 for spacing
            continue;
        }
        if consumed >= scroll + viewport {
            break;
        }
        let bg = bg_for(&blk.kind, theme);
        let header_color = header_color_for(&blk.kind, theme);
        let (header_text, header_style) = render_header(blk, theme, header_color);

        let block_lines = render_block_lines(blk, theme, width);

        // Top border
        if consumed + 1 > scroll {
            visible_lines.push(styled_line(
                border_line("▏", &header_text, theme),
                bg,
                header_style,
            ));
        }

        // Content lines
        for l in block_lines {
            if consumed + 2 > scroll && consumed + 2 <= scroll + viewport {
                visible_lines.push(with_bg(l, bg));
            }
        }

        // Bottom border
        if consumed + h > scroll && consumed + h <= scroll + viewport {
            visible_lines.push(styled_line(
                border_line("▏", "", theme),
                bg,
                Style::default().fg(theme.border),
            ));
        }

        consumed = blk_end + 1; // +1 for spacing
    }

    f.render_widget(Clear, inner);

    let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn compute_scroll(total: usize, viewport: usize, auto_follow: bool, manual: usize) -> usize {
    if total <= viewport {
        return 0;
    }
    if auto_follow {
        return total - viewport;
    }
    manual.min(total.saturating_sub(viewport))
}

fn bg_for(kind: &BlockKind, theme: &Theme) -> ratatui::style::Color {
    match kind {
        BlockKind::User => theme.block_bg_user,
        BlockKind::Assistant | BlockKind::StreamingAssistant => theme.block_bg_assistant,
        BlockKind::ToolCall { .. } => theme.block_bg_tool,
        BlockKind::Thinking => theme.block_bg_thinking,
        BlockKind::System => theme.block_bg_system,
        BlockKind::Error => theme.error,
    }
}

fn header_color_for(kind: &BlockKind, theme: &Theme) -> ratatui::style::Color {
    match kind {
        BlockKind::User => theme.user,
        BlockKind::Assistant | BlockKind::StreamingAssistant => theme.assistant,
        BlockKind::ToolCall { status, .. } => match status {
            ToolCallStatus::Running => theme.warning,
            ToolCallStatus::Ok => theme.success,
            ToolCallStatus::Failed => theme.error,
        },
        BlockKind::Thinking => theme.warning,
        BlockKind::System => theme.dim,
        BlockKind::Error => theme.error,
    }
}

fn render_header(blk: &ConvBlock, _theme: &Theme, color: ratatui::style::Color) -> (String, Style) {
    let elapsed = blk
        .elapsed_ms
        .map(format_elapsed_short)
        .unwrap_or_default();
    let header = match &blk.kind {
        BlockKind::User => format!("▸ You  {}  {elapsed}", format_time_ago(blk)),
        BlockKind::Assistant => {
            if elapsed.is_empty() {
                format!("⚡ Assistant  {}", format_time_ago(blk))
            } else {
                format!("⚡ Assistant  {}  {elapsed}", format_time_ago(blk))
            }
        }
        BlockKind::StreamingAssistant => format!(
            "⚡ Assistant  {}  {elapsed}…",
            format_time_ago(blk)
        ),
        BlockKind::Thinking => format!(
            "⟳ Thinking  {}  {elapsed}…",
            format_time_ago(blk)
        ),
        BlockKind::ToolCall { name, status, .. } => {
            let marker = match status {
                ToolCallStatus::Running => "⟳",
                ToolCallStatus::Ok => "✓",
                ToolCallStatus::Failed => "✗",
            };
            format!("{marker} {name}  {}  {elapsed}", format_time_ago(blk))
        }
        BlockKind::System => format!("* System  {}", format_time_ago(blk)),
        BlockKind::Error => format!("✗ Error  {}", format_time_ago(blk)),
    };
    (header, Style::default().fg(color).add_modifier(Modifier::BOLD))
}

fn border_line(left: &str, right: &str, _theme: &Theme) -> String {
    let _ = right;
    format!("{left} {}", "")
}

fn render_block_lines(blk: &ConvBlock, theme: &Theme, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    match &blk.kind {
        BlockKind::User => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {l}"),
                    Style::default().fg(theme.fg),
                )));
            }
            if blk.content.is_empty() {
                lines.push(Line::from(Span::styled("  ", Style::default())));
            }
        }
        BlockKind::Assistant | BlockKind::StreamingAssistant => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {l}"),
                    Style::default().fg(theme.fg),
                )));
            }
            if blk.content.is_empty() {
                lines.push(Line::from(Span::styled("  ", Style::default())));
            }
            if matches!(blk.kind, BlockKind::StreamingAssistant) {
                lines.push(Line::from(Span::styled(
                    "  ▌",
                    Style::default().fg(theme.accent),
                )));
            }
        }
        BlockKind::Thinking => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {l}"),
                    Style::default().fg(theme.dim),
                )));
            }
        }
        BlockKind::ToolCall { args, .. } => {
            if !args.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  → {}", truncate(args, 80)),
                    Style::default().fg(theme.dim),
                )));
            }
            if !blk.content.is_empty() {
                for l in blk.content.lines().take(6) {
                    lines.push(Line::from(Span::styled(
                        format!("  │ {l}"),
                        Style::default().fg(theme.dim),
                    )));
                }
            }
            if lines.is_empty() {
                lines.push(Line::from(Span::styled("  ", Style::default())));
            }
        }
        BlockKind::System => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {l}"),
                    Style::default().fg(theme.dim),
                )));
            }
        }
        BlockKind::Error => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {l}"),
                    Style::default().fg(theme.error),
                )));
            }
        }
    }

    // Hard wrap if needed
    if width < 4 {
        return lines;
    }
    let mut out: Vec<Line<'static>> = Vec::new();
    for line in lines {
        let w = line.width();
        if w <= width {
            out.push(line);
            continue;
        }
        // Simple character-based wrap: rebuild spans with owned Strings
        // when adjacent characters share the same style. We rebuild the
        // line as: a fresh buffer of (text, style) pairs flushed whenever
        // style changes or we hit the wrap boundary.
        let mut buf: Vec<(String, Style)> = Vec::new();
        let mut buf_w = 0usize;

        fn flush(buf: &mut Vec<(String, Style)>, out: &mut Vec<Line<'static>>) {
            if buf.is_empty() {
                return;
            }
            let spans: Vec<Span<'static>> = buf
                .drain(..)
                .map(|(s, st)| Span::styled(s, st))
                .collect();
            out.push(Line::from(spans));
        }

        for span in line.spans.into_iter() {
            let style = span.style;
            let content: String = span.content.into_owned();
            for c in content.chars() {
                if buf_w >= width {
                    flush(&mut buf, &mut out);
                    buf_w = 0;
                }
                if let Some((text, st)) = buf.last_mut() {
                    if *st == style {
                        text.push(c);
                    } else {
                        buf.push((c.to_string(), style));
                    }
                } else {
                    buf.push((c.to_string(), style));
                }
                buf_w += 1;
            }
        }
        flush(&mut buf, &mut out);
    }
    out
}

fn with_bg(line: Line<'static>, bg: ratatui::style::Color) -> Line<'static> {
    let mut new_spans = Vec::with_capacity(line.spans.len());
    for span in line.spans {
        let mut s = span.style;
        s = s.bg(bg);
        new_spans.push(Span { content: span.content, style: s });
    }
    Line::from(new_spans)
}

fn styled_line(content: String, bg: ratatui::style::Color, fg_style: Style) -> Line<'static> {
    let mut s = fg_style;
    s = s.bg(bg);
    Line::from(Span::styled(content, s))
}

fn format_time_ago(blk: &ConvBlock) -> String {
    let secs = blk.created_at.elapsed().as_secs();
    if secs < 5 {
        "now".to_string()
    } else if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}

fn format_elapsed_short(ms: u128) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", ms as f64 / 1000.0)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}…")
    }
}

// Borders placeholder type so the unused `Block` import doesn't warn
#[allow(dead_code)]
type _B = Borders;

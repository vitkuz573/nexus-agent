//! Conversation panel — modern block-based rendering.
//!
//! Each block (user, assistant, tool, thinking, system, error) is rendered
//! as its own widget in its own Rect, computed via `Layout::split()`.
//! This means:
//!   - No manual line tracking
//!   - No per-line background fills
//!   - No "shared selection" because blocks are independent Paragraphs
//!   - Unicode rounded borders (╭─╮│╰─╯) for a modern look
//!   - Distinct color per role with bold headers + icons
//!
//! Auto-scroll follows the bottom by default. The user can scroll up
//! (PgUp/Up/wheel) to disable auto-follow; pressing End or scrolling to
//! the bottom re-enables it. New blocks snap back to auto-follow.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::state::{Block as ConvBlock, BlockKind, ToolCallStatus, TuiState};
use crate::tui::theme::Theme;

/// Vertical space (in lines) between consecutive blocks.
const BLOCK_SPACING: u16 = 1;

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
            "  Type a message below to start.  Try /help for commands.",
            Style::default().fg(theme.dim),
        )));
        f.render_widget(hint, inner);
        return;
    }

    let content_width = inner.width.saturating_sub(2) as usize;
    let viewport = inner.height;

    // Heights for each block (header + wrapped content + 2 for borders)
    let heights: Vec<u16> = state
        .blocks
        .iter()
        .map(|b| block_height(b, content_width))
        .collect();

    let total = heights
        .iter()
        .sum::<u16>()
        .saturating_add((state.blocks.len() as u16).saturating_sub(1) * BLOCK_SPACING);

    let scroll = compute_scroll(total, viewport, state.scroll.auto_follow, state.scroll.manual_offset);

    // Compute cumulative Y positions for each block, then determine which
    // fall inside [scroll, scroll+viewport).
    let mut positions: Vec<u16> = Vec::with_capacity(state.blocks.len());
    let mut y = 0u16;
    for &h in &heights {
        positions.push(y);
        y = y.saturating_add(h).saturating_add(BLOCK_SPACING);
    }

    // Build the visible rects via Layout::split(). We pass the heights
    // of all blocks intersected with the viewport.
    let mut visible_indices: Vec<usize> = Vec::new();
    let mut visible_constraints: Vec<Constraint> = Vec::new();
    for (i, &h) in heights.iter().enumerate() {
        let blk_top = positions[i];
        let blk_bot = blk_top.saturating_add(h);
        if blk_bot <= scroll {
            continue;
        }
        if blk_top >= scroll.saturating_add(viewport) {
            break;
        }
        visible_indices.push(i);
        visible_constraints.push(Constraint::Length(h));
    }

    if visible_indices.is_empty() {
        return;
    }

    // Add spacers between visible blocks
    let mut full_constraints: Vec<Constraint> = Vec::new();
    for (i, c) in visible_constraints.iter().enumerate() {
        full_constraints.push(*c);
        if i + 1 < visible_indices.len() {
            full_constraints.push(Constraint::Length(BLOCK_SPACING));
        }
    }

    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(full_constraints)
        .split(inner);

    // Render each block in its Rect
    for (slot_idx, &blk_idx) in visible_indices.iter().enumerate() {
        let rect_idx = slot_idx * 2; // each block followed by a spacer
        if rect_idx >= rects.len() {
            break;
        }
        let rect = rects[rect_idx];
        let blk = &state.blocks[blk_idx];
        let visible_top = scroll.saturating_sub(positions[blk_idx]);
        render_block(f, rect, blk, theme, visible_top, content_width);
    }
}

fn render_block(
    f: &mut Frame,
    rect: Rect,
    blk: &ConvBlock,
    theme: &Theme,
    visible_top: u16,
    content_width: usize,
) {
    let (border_color, bg, header_color, icon) = block_style(&blk.kind, theme);

    // Build the rounded border block
    let block_widget = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::default().fg(border_color).bg(bg))
        .style(Style::default().bg(bg));

    let inner = block_widget.inner(rect);
    f.render_widget(Clear, rect);
    f.render_widget(block_widget, rect);

    // Header line + content lines
    let header = render_header_line(blk, theme, header_color, icon);
    let content_lines = render_block_content(blk, content_width);

    let mut all_lines: Vec<Line<'static>> = Vec::with_capacity(1 + content_lines.len());
    all_lines.push(header);
    all_lines.extend(content_lines);

    let skip = visible_top as usize;
    let take = inner.height as usize;
    let visible: Vec<Line<'static>> = all_lines.into_iter().skip(skip).take(take).collect();

    let paragraph = Paragraph::new(visible)
        .style(Style::default().bg(bg))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn block_style(
    kind: &BlockKind,
    theme: &Theme,
) -> (
    ratatui::style::Color, // border
    ratatui::style::Color, // bg
    ratatui::style::Color, // header
    &'static str,          // icon
) {
    match kind {
        BlockKind::User => (theme.user, theme.block_bg_user, theme.user, "▸"),
        BlockKind::Assistant | BlockKind::StreamingAssistant => (
            theme.assistant,
            theme.block_bg_assistant,
            theme.assistant,
            "⚡",
        ),
        BlockKind::ToolCall { status, .. } => {
            let (header_c, icon) = match status {
                ToolCallStatus::Running => (theme.warning, "⟳"),
                ToolCallStatus::Ok => (theme.success, "✓"),
                ToolCallStatus::Failed => (theme.error, "✗"),
            };
            (header_c, theme.block_bg_tool, header_c, icon)
        }
        BlockKind::Thinking => (theme.warning, theme.block_bg_thinking, theme.warning, "💭"),
        BlockKind::System => (theme.dim, theme.block_bg_system, theme.dim, "·"),
        BlockKind::Error => (theme.error, theme.bg, theme.error, "✗"),
    }
}

fn render_header_line(
    blk: &ConvBlock,
    _theme: &Theme,
    color: ratatui::style::Color,
    icon: &str,
) -> Line<'static> {
    let elapsed = blk
        .elapsed_ms
        .map(format_elapsed_short)
        .unwrap_or_default();
    let time = format_time_short(blk);

    let (title, right_suffix) = match &blk.kind {
        BlockKind::User => ("You".to_string(), elapsed),
        BlockKind::Assistant => ("Assistant".to_string(), elapsed),
        BlockKind::StreamingAssistant => ("Assistant".to_string(), format!("{elapsed}…")),
        BlockKind::Thinking => ("Thinking".to_string(), format!("{elapsed}…")),
        BlockKind::ToolCall { name, .. } => (name.clone(), elapsed),
        BlockKind::System => ("system".to_string(), String::new()),
        BlockKind::Error => ("Error".to_string(), String::new()),
    };

    let mut spans = vec![
        Span::styled(
            format!(" {icon} "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ];
    if !right_suffix.is_empty() || !time.is_empty() {
        spans.push(Span::styled(
            "  ",
            Style::default().fg(ratatui::style::Color::DarkGray),
        ));
        if !right_suffix.is_empty() {
            spans.push(Span::styled(
                right_suffix,
                Style::default().fg(ratatui::style::Color::Yellow),
            ));
            spans.push(Span::styled(
                " · ",
                Style::default().fg(ratatui::style::Color::DarkGray),
            ));
        }
        spans.push(Span::styled(
            time,
            Style::default().fg(ratatui::style::Color::DarkGray),
        ));
    }
    Line::from(spans)
}

fn render_block_content(blk: &ConvBlock, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    match &blk.kind {
        BlockKind::User | BlockKind::Assistant | BlockKind::StreamingAssistant => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::raw(format!(" {l}"))));
            }
            if blk.content.is_empty() {
                lines.push(Line::from(Span::styled(
                    " ",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )));
            }
            if matches!(blk.kind, BlockKind::StreamingAssistant) {
                lines.push(Line::from(Span::styled(
                    " ▌",
                    Style::default()
                        .fg(ratatui::style::Color::Cyan)
                        .add_modifier(Modifier::SLOW_BLINK),
                )));
            }
        }
        BlockKind::Thinking => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!(" {l}"),
                    Style::default().fg(ratatui::style::Color::Gray),
                )));
            }
            if lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  reasoning…",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )));
            }
        }
        BlockKind::ToolCall { args, .. } => {
            if !args.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!(" → {}", truncate(args, 80)),
                    Style::default().fg(ratatui::style::Color::Gray),
                )));
            }
            if !blk.content.is_empty() {
                for l in blk.content.lines().take(10) {
                    lines.push(Line::from(Span::styled(
                        format!(" │ {l}"),
                        Style::default().fg(ratatui::style::Color::Gray),
                    )));
                }
            }
            if lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  running…",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )));
            }
        }
        BlockKind::System => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!(" {l}"),
                    Style::default().fg(ratatui::style::Color::Gray),
                )));
            }
        }
        BlockKind::Error => {
            for l in blk.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!(" {l}"),
                    Style::default().fg(ratatui::style::Color::Red),
                )));
            }
        }
    }
    wrap_lines(lines, width)
}

fn wrap_lines(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
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
        let mut buf: Vec<(String, Style)> = Vec::new();
        let mut buf_w = 0usize;
        for span in line.spans.into_iter() {
            let style = span.style;
            let content: String = span.content.into_owned();
            for c in content.chars() {
                if buf_w >= width {
                    flush_wrap(&mut buf, &mut out);
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
        flush_wrap(&mut buf, &mut out);
    }
    out
}

fn flush_wrap(buf: &mut Vec<(String, Style)>, out: &mut Vec<Line<'static>>) {
    if buf.is_empty() {
        return;
    }
    let spans: Vec<Span<'static>> = buf
        .drain(..)
        .map(|(s, st)| Span::styled(s, st))
        .collect();
    out.push(Line::from(spans));
}

fn block_height(blk: &ConvBlock, content_width: usize) -> u16 {
    let n = render_block_content(blk, content_width).len() as u16;
    n.saturating_add(2) // +1 top border, +1 bottom border, content goes inside
}

fn compute_scroll(total: u16, viewport: u16, auto_follow: bool, manual: usize) -> u16 {
    if total <= viewport {
        return 0;
    }
    if auto_follow {
        return total - viewport;
    }
    (manual as u16).min(total.saturating_sub(viewport))
}

fn format_time_short(blk: &ConvBlock) -> String {
    let secs = blk.created_at.elapsed().as_secs();
    if secs < 5 {
        "now".to_string()
    } else if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
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

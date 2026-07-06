//! Panel renderers. Each panel is a function that takes a frame area and
//! the shared state, then renders into that area.

pub mod conversation;
pub mod header;
pub mod input;
pub mod sidebar;
pub mod status;

pub use conversation::render_conversation;
pub use header::render_header;
pub use input::render_input;
pub use sidebar::render_sidebar;
pub use status::render_status;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Compute the standard 5-region layout.
///
/// ```
/// ┌───────────────────────────────────────────────────────────────┐
/// │                          Header (3)                          │
/// ├──────────┬───────────────────────────────┬───────────────────┤
/// │ Sidebar  │       Conversation            │  Cognitive        │
/// │ (left)   │       (center)                │  (right)          │
/// │          │                               │                   │
/// ├──────────┴───────────────────────────────┴───────────────────┤
/// │                          Input (5)                           │
/// ├───────────────────────────────────────────────────────────────┤
/// │                          Status (1)                          │
/// └───────────────────────────────────────────────────────────────┘
/// ```
pub fn layout(area: Rect, sidebar_left: bool) -> Vec<Rect> {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(area);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if sidebar_left {
            [
                Constraint::Percentage(20),
                Constraint::Percentage(55),
                Constraint::Percentage(25),
            ]
        } else {
            [
                Constraint::Percentage(25),
                Constraint::Percentage(55),
                Constraint::Percentage(20),
            ]
        })
        .split(vertical[1]);

    vec![vertical[0], middle[0], middle[1], middle[2], vertical[2], vertical[3]]
}

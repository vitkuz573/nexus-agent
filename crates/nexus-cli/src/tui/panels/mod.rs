//! Panel renderers. Each panel is a function that takes a frame area and
//! the shared state, then renders into that area.

pub mod conversation;
pub mod files;
pub mod header;
pub mod input;
pub mod status;

pub use conversation::render_conversation;
pub use files::render_files;
pub use header::render_header;
pub use input::render_input;
pub use status::render_status;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Compute the standard 4-region layout.
///
/// ```
/// ┌───────────────────────────────────────────────────────────────┐
/// │                          Header (1)                          │
/// ├──────────┬────────────────────────────────────────────────────┤
/// │          │                                                     │
/// │  Files   │              Conversation                           │
/// │  (20%)   │              (75%, block-based)                     │
/// │          │                                                     │
/// ├──────────┴────────────────────────────────────────────────────┤
/// │                          Input (5)                            │
/// ├───────────────────────────────────────────────────────────────┤
/// │                          Status (1)                           │
/// └───────────────────────────────────────────────────────────────┘
/// ```
pub fn layout(area: Rect) -> Vec<Rect> {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(area);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(vertical[1]);

    vec![vertical[0], middle[0], middle[1], vertical[2], vertical[3]]
}

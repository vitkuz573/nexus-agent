//! Event bus between the spawned agent task and the TUI event loop.
//!
//! The spawned task pushes `UiEvent`s into the channel; the main loop
//! drains the receiver each frame and mutates `TuiState` accordingly.

#![allow(dead_code)]

use tokio::sync::mpsc;

use crate::tui::state::MessageRole;

/// UI events pushed from background tasks into the main event loop.
#[derive(Debug, Clone)]
pub enum UiEvent {
    StatusChanged(String),
    AppendMessage {
        role: MessageRole,
        content: String,
    },
    ClearConversation,
    SwitchTheme,
    ToolCallStarted(String),
    ToolCallFinished {
        name: String,
        ok: bool,
        output: String,
    },
    VerificationDone(crate::tui::state::VerificationDisplay),
}

pub type UiSink = mpsc::UnboundedSender<UiEvent>;

//! Event bus between the spawned agent task and the TUI event loop.
//!
//! The spawned task pushes `UiEvent`s into the channel; the main loop
//! drains the receiver each frame and mutates `TuiState` accordingly.

#![allow(dead_code)]

use tokio::sync::mpsc;

use crate::tui::state::{MessageRole, ToolCallStatus};

/// UI events pushed from background tasks into the main event loop.
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Update the agent status (shown in the header spinner).
    StatusChanged(String),
    /// Append a fully-rendered message (used for final assistant output).
    AppendMessage {
        role: MessageRole,
        content: String,
    },
    /// Append an error block.
    AppendError(String),
    /// Streaming: a single token arrived for the current assistant block.
    StreamToken(String),
    /// Streaming: the assistant block is now complete.
    StreamDone(String),
    /// A "thinking" sub-block started (the model produced reasoning text
    /// before its main response).
    ThinkingStarted(String),
    /// A tool call started executing.
    ToolCallStarted {
        name: String,
        args: String,
    },
    /// A tool call finished.
    ToolCallFinished {
        name: String,
        ok: bool,
        output: String,
    },
    /// Internal: predictor's analysis before the run started.
    Predicted {
        confidence: f32,
        approach: String,
        risks: Vec<String>,
    },
    /// Internal: verifier result on the final assistant response.
    Verified {
        score: f32,
        passed: bool,
        issues: Vec<String>,
    },
    /// Internal: stored a long-term memory entry.
    Stored {
        key: String,
        category: String,
    },
    /// Clear all blocks.
    ClearConversation,
    /// Switch to the next theme.
    SwitchTheme,
    /// Verification result ready (for sidebar display).
    VerificationDone(crate::tui::state::VerificationDisplay),
}

pub type UiSink = mpsc::UnboundedSender<UiEvent>;

#[allow(dead_code)]
pub type _T = ToolCallStatus;

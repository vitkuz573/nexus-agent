//! Shared state for the TUI.
//!
//! Everything the UI needs to render lives here. Mutations happen on the
//! main thread; the streaming layer pushes events in via `mpsc`.

#![allow(dead_code)]

use std::time::Instant;

use nexus_brain::thought::ThoughtChain;
use nexus_brain::verify::VerificationResult;

/// One entry in the conversation transcript.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Instant,
    pub streaming: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
    Error,
}

/// One tool call as it is being executed.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolEvent {
    pub name: String,
    pub status: ToolStatus,
    pub arguments: String,
    pub result: Option<String>,
    pub started_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ToolStatus {
    Running,
    Ok,
    Failed,
}

/// Which panel currently has focus for keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Conversation,
    Sidebar,
}

/// Top-level status shown in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Streaming,
    Running,
}

/// Lightweight summary of a verification result, rendered in the sidebar.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VerificationDisplay {
    pub label: String,
    pub score: f32,
    pub passed: bool,
}

/// Aggregated state passed into the renderer each frame.
#[allow(dead_code)]
pub struct TuiState {
    pub messages: Vec<Message>,
    pub tool_events: Vec<ToolEvent>,
    pub thought_chain: Option<ThoughtChain>,
    pub verifications: Vec<VerificationResult>,
    pub status: AgentStatus,
    pub focus: Focus,
    pub model: String,
    pub provider: String,
    pub round: usize,
    pub max_rounds: usize,
    pub tokens_used: u32,
    pub max_tokens: u32,
    pub started_at: Instant,
    pub elapsed_ms: u128,
    pub file_tree: Vec<FileEntry>,
    pub scroll: ScrollState,
    pub command_palette: bool,
    pub help_visible: bool,
    pub spinner_frame: usize,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ScrollState {
    pub conversation_offset: usize,
    pub sidebar_offset: usize,
    pub input_offset: usize,
}

impl TuiState {
    pub fn new(model: String, provider: String, max_rounds: usize, max_tokens: u32) -> Self {
        Self {
            messages: Vec::new(),
            tool_events: Vec::new(),
            thought_chain: None,
            verifications: Vec::new(),
            status: AgentStatus::Idle,
            focus: Focus::Input,
            model,
            provider,
            round: 0,
            max_rounds,
            tokens_used: 0,
            max_tokens,
            started_at: Instant::now(),
            elapsed_ms: 0,
            file_tree: Vec::new(),
            scroll: ScrollState::default(),
            command_palette: false,
            help_visible: false,
            spinner_frame: 0,
        }
    }

    pub fn tick(&mut self) {
        self.elapsed_ms = self.started_at.elapsed().as_millis();
        self.spinner_frame = self.spinner_frame.wrapping_add(1);
    }

    pub fn push_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn push_tool(&mut self, tool: ToolEvent) {
        self.tool_events.push(tool);
    }

    pub fn last_message_mut(&mut self) -> Option<&mut Message> {
        self.messages.last_mut()
    }
}

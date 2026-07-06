//! Shared state for the TUI.
//!
//! Everything the UI needs to render lives here. Mutations happen on the
//! main thread; the streaming layer pushes events in via `mpsc`.

#![allow(dead_code)]

use std::time::Instant;

use nexus_brain::thought::ThoughtChain;
use nexus_brain::verify::VerificationResult;

/// A single visible block in the conversation panel.
///
/// Each block is rendered as an independent unit (its own header line,
/// its own content lines). Blocks never share a selection. They scroll
/// together as one continuous transcript.
#[derive(Debug, Clone)]
pub struct Block {
    pub kind: BlockKind,
    pub content: String,
    pub created_at: Instant,
    pub elapsed_ms: Option<u128>,
}

#[derive(Debug, Clone)]
pub enum BlockKind {
    User,
    Assistant,
    StreamingAssistant,
    Thinking,
    ToolCall {
        name: String,
        args: String,
        status: ToolCallStatus,
    },
    System,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    Running,
    Ok,
    Failed,
}

/// Legacy message type — kept for the slash-command code paths that still
/// construct it. New code should push `Block`s instead.
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

/// Legacy tool event — kept for compatibility with the old event bus.
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
    Files,
}

/// Top-level status shown in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Thinking,
    Streaming,
    Running,
}

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
    /// All blocks in display order — user msgs, assistant msgs, tool
    /// calls, thinking, errors — all interleaved.
    pub blocks: Vec<Block>,
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
    /// Timestamp of the last started agent run (for per-message timing).
    pub run_started_at: Option<Instant>,
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
    /// When true, follow new content (snap to bottom on new blocks).
    pub auto_follow: bool,
    /// Manual offset from bottom. Only meaningful when `auto_follow` is false.
    pub manual_offset: usize,
    /// Reserved for future per-region scroll.
    pub input_offset: usize,
}

impl TuiState {
    pub fn new(model: String, provider: String, max_rounds: usize, max_tokens: u32) -> Self {
        Self {
            blocks: Vec::new(),
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
            scroll: ScrollState {
                auto_follow: true,
                manual_offset: 0,
                input_offset: 0,
            },
            command_palette: false,
            help_visible: false,
            spinner_frame: 0,
            run_started_at: None,
        }
    }

    pub fn tick(&mut self) {
        self.elapsed_ms = self.started_at.elapsed().as_millis();
        self.spinner_frame = self.spinner_frame.wrapping_add(1);
    }

    pub fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn last_block_mut(&mut self) -> Option<&mut Block> {
        self.blocks.last_mut()
    }

    pub fn last_streaming_mut(&mut self) -> Option<&mut Block> {
        self.blocks
            .iter_mut()
            .rev()
            .find(|b| matches!(b.kind, BlockKind::StreamingAssistant))
    }
}

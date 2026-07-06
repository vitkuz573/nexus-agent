//! Events emitted by the agent during a run.
//!
//! The agent's run loop calls the provided callback (or pushes to the
//! provided channel) for each meaningful event so the UI can render
//! progress incrementally instead of waiting for the final result.

#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// A single token arrived during the final assistant response.
    /// For tool-call responses, no tokens are emitted — only tool events.
    Token(String),
    /// A tool call started executing. `args` is the raw JSON arguments
    /// the model supplied.
    ToolStarted {
        name: String,
        args: String,
    },
    /// A tool call finished. `ok` is true if the tool returned Ok.
    ToolFinished {
        name: String,
        ok: bool,
        output: String,
    },
    /// The model emitted reasoning text (for models that support it).
    Thinking(String),
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
    /// The agent run completed successfully with the final assistant
    /// message (the full assembled response, including any tool calls).
    Done(String),
    /// The agent run failed with the given error message.
    Failed(String),
}

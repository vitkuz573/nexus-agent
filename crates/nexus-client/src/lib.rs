pub mod error;
pub mod message;
pub mod provider;
pub mod stream;

pub use error::ClientError;
pub use message::{Message, Role, ToolCall, ToolResult};
pub use provider::{ChatRequest, ChatResponse, LlmProvider};
pub use stream::{StreamEvent, StreamParser};

#[cfg(test)]
mod tests;

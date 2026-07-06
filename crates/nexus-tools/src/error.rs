use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("tool not found: {0}")]
    NotFound(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },

    #[error("stream closed unexpectedly")]
    StreamClosed,

    #[error("invalid SSE line: {0}")]
    InvalidSse(String),

    #[error("connection timeout")]
    Timeout,
}

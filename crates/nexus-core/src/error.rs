use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("client error: {0}")]
    Client(#[from] nexus_client::ClientError),

    #[error("tool error: {0}")]
    Tool(#[from] nexus_tools::ToolError),

    #[error("config error: {0}")]
    Config(#[from] nexus_config::ConfigError),

    #[error("agent loop exceeded max rounds ({0})")]
    MaxRounds(usize),

    #[error("empty response from model")]
    EmptyResponse,

    #[error("model returned only tool calls without content")]
    NoContent,
}

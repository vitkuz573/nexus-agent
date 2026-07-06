use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config file not found at {path}")]
    NotFound { path: String },

    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("provider '{name}' not found")]
    ProviderNotFound { name: String },
}

use crate::error::ConfigError;
use crate::provider::ProviderConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "nexus-agent";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,

    #[serde(default = "default_max_tool_rounds")]
    pub max_tool_rounds: usize,

    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    #[serde(default)]
    pub log_level: String,
}

fn default_max_tool_rounds() -> usize {
    20
}

fn default_system_prompt() -> String {
    r#"You are Nexus — a coding agent that thinks before acting.

You have tools available. Use them when they help you do better work.

The best developers:
- Read code before changing it
- Understand the problem before solving it
- Test their solutions
- Learn from mistakes

You have a memory. Store important decisions and patterns. Recall them when relevant.

You have a code verifier. Use it to check your work — you'll catch bugs you'd otherwise miss.

You have a risk analyzer. Use it to detect security issues, unwrap() calls, and potential panics.

You have semantic search. Use it to find relevant code faster than grep.

You have structured thinking. Use it when a problem is complex — it helps you see the full picture.

The quality of your work matters more than speed. Think deeply. Verify thoroughly. Ship clean code."#
        .to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            max_tool_rounds: default_max_tool_rounds(),
            system_prompt: default_system_prompt(),
            log_level: "info".to_string(),
        }
    }
}

impl Settings {
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        let dir = dirs::config_dir()
            .ok_or_else(|| ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "cannot determine config directory",
            )))?
            .join(APP_NAME);
        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf, ConfigError> {
        Ok(Self::config_dir()?.join(CONFIG_FILE))
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;
        if !path.exists() {
            tracing::debug!("config not found at {}, using defaults", path.display());
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let settings: Settings = toml::from_str(&content)?;
        tracing::info!("loaded config from {}", path.display());
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(CONFIG_FILE);
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        tracing::info!("saved config to {}", path.display());
        Ok(())
    }

    pub fn get_provider(&self, name: &str) -> Result<&ProviderConfig, ConfigError> {
        if name == "default" {
            return self.providers.first().ok_or(ConfigError::ProviderNotFound {
                name: name.to_string(),
            });
        }
        self.providers
            .iter()
            .find(|p| p.name == name)
            .ok_or(ConfigError::ProviderNotFound { name: name.to_string() })
    }

    pub fn add_provider(&mut self, provider: ProviderConfig) {
        self.providers.push(provider);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = Settings::default();
        assert_eq!(s.max_tool_rounds, 20);
        assert!(s.providers.is_empty());
    }

    #[test]
    fn test_provider_lookup() {
        let mut s = Settings::default();
        s.add_provider(ProviderConfig::new("test", "http://localhost", "key", "model"));

        assert!(s.get_provider("test").is_ok());
        assert!(s.get_provider("missing").is_err());
        assert!(s.get_provider("default").is_ok());
    }
}

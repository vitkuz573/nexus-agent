use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

impl ProviderConfig {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let p = ProviderConfig::new("test", "http://localhost", "key", "model");
        assert_eq!(p.name, "test");
        assert_eq!(p.max_tokens, 4096);
    }
}

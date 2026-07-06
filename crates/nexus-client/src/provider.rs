use crate::error::ClientError;
use crate::message::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSchema>>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub function: FunctionSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMessage {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<crate::message::ToolCall>>,
}

pub struct LlmProvider {
    client: Client,
    base_url: String,
    api_key: String,
}

impl LlmProvider {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        tools: Option<&[ToolSchema]>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<ChatResponse, ClientError> {
        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            tools: tools.map(|t| t.to_vec()),
            stream: false,
            max_tokens,
            temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::Api { status, body });
        }

        let resp: ChatResponse = response.json().await?;
        Ok(resp)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub async fn complete_stream(
        &self,
        model: &str,
        messages: &[Message],
        tools: Option<&[ToolSchema]>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<futures::stream::BoxStream<'static, Result<super::stream::StreamEvent, ClientError>>, ClientError>
    {
        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            tools: tools.map(|t| t.to_vec()),
            stream: true,
            max_tokens,
            temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::Api { status, body });
        }

        let byte_stream = response.bytes_stream();
        let parser = super::stream::StreamParser::new(byte_stream);

        Ok(Box::pin(parser))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let p = LlmProvider::new("http://localhost:8080/v1", "test-key");
        assert_eq!(p.base_url(), "http://localhost:8080/v1");
        assert_eq!(p.api_key(), "test-key");
    }
}

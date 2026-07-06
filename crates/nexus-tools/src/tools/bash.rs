use crate::error::ToolError;
use crate::registry::ToolInstance;
use crate::schema::ToolDefinition;
use async_trait::async_trait;
use std::process::Command;

pub struct BashTool;

#[async_trait]
impl ToolInstance for BashTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: "Execute a shell command and return stdout/stderr".to_string(),
            parameters: crate::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![crate::schema::ToolProperty {
                    name: "command".to_string(),
                    prop_type: "string".to_string(),
                    description: "The bash command to execute".to_string(),
                }],
                required: vec!["command".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let command = args["command"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("missing 'command'".to_string()))?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("STDERR:\n");
            result.push_str(&stderr);
        }
        if result.is_empty() {
            result = "(no output)".to_string();
        }

        Ok(result)
    }
}

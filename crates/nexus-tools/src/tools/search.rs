use crate::error::ToolError;
use crate::registry::ToolInstance;
use crate::schema::ToolDefinition;
use async_trait::async_trait;
use std::process::Command;

pub struct GrepTool;

#[async_trait]
impl ToolInstance for GrepTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search for a pattern in files using regex".to_string(),
            parameters: crate::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![
                    crate::schema::ToolProperty {
                        name: "pattern".to_string(),
                        prop_type: "string".to_string(),
                        description: "Regex pattern to search for".to_string(),
                    },
                    crate::schema::ToolProperty {
                        name: "path".to_string(),
                        prop_type: "string".to_string(),
                        description: "Directory to search in (default: current dir)".to_string(),
                    },
                    crate::schema::ToolProperty {
                        name: "include".to_string(),
                        prop_type: "string".to_string(),
                        description: "File glob pattern to include (e.g. *.rs)".to_string(),
                    },
                ],
                required: vec!["pattern".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("missing 'pattern'".to_string()))?;
        let path = args["path"].as_str().unwrap_or(".");
        let include = args["include"].as_str();

        let output = if include.is_some() {
            Command::new("grep")
                .args(["-rn", "--include", include.unwrap_or("*"), pattern, path])
                .output()?
        } else {
            Command::new("grep")
                .args(["-rn", pattern, path])
                .output()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.is_empty() {
            Ok("No matches found".to_string())
        } else {
            Ok(stdout.to_string())
        }
    }
}

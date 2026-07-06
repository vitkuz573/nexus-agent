use crate::error::ToolError;
use crate::registry::ToolInstance;
use crate::schema::ToolDefinition;
use async_trait::async_trait;

pub struct ReadFileTool;

#[async_trait]
impl ToolInstance for ReadFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: crate::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![crate::schema::ToolProperty {
                    name: "path".to_string(),
                    prop_type: "string".to_string(),
                    description: "Path to the file to read".to_string(),
                }],
                required: vec!["path".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;

        let content = std::fs::read_to_string(path)?;
        Ok(content)
    }
}

pub struct WriteFileTool;

#[async_trait]
impl ToolInstance for WriteFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file, creating parent directories if needed".to_string(),
            parameters: crate::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![
                    crate::schema::ToolProperty {
                        name: "path".to_string(),
                        prop_type: "string".to_string(),
                        description: "Path to the file to write".to_string(),
                    },
                    crate::schema::ToolProperty {
                        name: "content".to_string(),
                        prop_type: "string".to_string(),
                        description: "Content to write to the file".to_string(),
                    },
                ],
                required: vec!["path".to_string(), "content".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("missing 'content'".to_string()))?;

        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        Ok(format!("Wrote {} bytes to {}", content.len(), path))
    }
}

pub struct ListDirTool;

#[async_trait]
impl ToolInstance for ListDirTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_dir".to_string(),
            description: "List files and directories at a given path".to_string(),
            parameters: crate::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![crate::schema::ToolProperty {
                    name: "path".to_string(),
                    prop_type: "string".to_string(),
                    description: "Directory path to list".to_string(),
                }],
                required: vec!["path".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;

        let entries = std::fs::read_dir(path)?;
        let mut items: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    format!("{name}/")
                } else {
                    name
                }
            })
            .collect();
        items.sort();
        Ok(items.join("\n"))
    }
}

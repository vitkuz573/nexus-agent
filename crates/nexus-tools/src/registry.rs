use crate::error::ToolError;
use crate::schema::ToolDefinition;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait ToolInstance: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolInstance>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn ToolInstance>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ToolInstance> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    pub async fn execute(&self, name: &str, args: serde_json::Value) -> Result<String, ToolError> {
        let tool = self.get(name).ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        tool.execute(args).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

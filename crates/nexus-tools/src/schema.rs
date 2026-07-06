use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: Vec<ToolProperty>,
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    pub name: String,
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
}

impl ToolDefinition {
    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut props = serde_json::Map::new();
        let mut required = Vec::new();

        for prop in &self.parameters.properties {
            let prop_schema = serde_json::json!({
                "type": prop.prop_type,
                "description": prop.description
            });
            props.insert(prop.name.clone(), prop_schema);
            if self.parameters.required.contains(&prop.name) {
                required.push(prop.name.clone());
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": props,
            "required": required
        })
    }
}

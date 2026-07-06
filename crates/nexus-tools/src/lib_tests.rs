use nexus_tools::registry::{ToolInstance, ToolRegistry};
use nexus_tools::tools::*;
use serde_json::json;

#[tokio::test]
async fn test_bash_tool() {
    let tool = BashTool;
    let def = tool.definition();
    assert_eq!(def.name, "bash");

    let result = tool.execute(json!({"command": "echo hello"})).await.unwrap();
    assert!(result.contains("hello"));
}

#[tokio::test]
async fn test_read_file_tool() {
    let tool = ReadFileTool;
    let result = tool.execute(json!({"path": "/etc/hostname"})).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_dir_tool() {
    let tool = ListDirTool;
    let result = tool.execute(json!({"path": "/tmp"})).await.unwrap();
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_tool_registry() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(BashTool));
    registry.register(Box::new(ReadFileTool));

    assert_eq!(registry.names().len(), 2);
    assert!(registry.get("bash").is_some());
    assert!(registry.get("nonexistent").is_none());
}

#[tokio::test]
async fn test_registry_execute() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(BashTool));

    let result = registry.execute("bash", json!({"command": "echo 42"})).await.unwrap();
    assert!(result.contains("42"));
}

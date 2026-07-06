#[cfg(test)]
mod bash_tests {
    use crate::tools::BashTool;
    use crate::registry::ToolInstance;
    use serde_json::json;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool;
        let result = tool.execute(json!({"command": "echo hello"})).await.unwrap();
        assert!(result.contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_error() {
        let tool = BashTool;
        let result = tool.execute(json!({"command": "exit 1"})).await;
        assert!(result.is_ok()); // Bash tool returns Ok even on non-zero exit
    }

    #[tokio::test]
    async fn test_bash_stderr() {
        let tool = BashTool;
        let result = tool.execute(json!({"command": "echo err >&2"})).await.unwrap();
        assert!(result.contains("STDERR"));
    }

    #[tokio::test]
    async fn test_bash_empty_output() {
        let tool = BashTool;
        let result = tool.execute(json!({"command": "true"})).await.unwrap();
        assert!(result.contains("no output"));
    }

    #[tokio::test]
    async fn test_bash_definition() {
        let tool = BashTool;
        let def = tool.definition();
        assert_eq!(def.name, "bash");
        assert!(!def.description.is_empty());
    }
}

#[cfg(test)]
mod file_ops_tests {
    use crate::tools::{ReadFileTool, WriteFileTool, ListDirTool};
    use crate::registry::ToolInstance;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_and_read() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");

        let writer = WriteFileTool;
        let result = writer.execute(json!({
            "path": path.to_str().unwrap(),
            "content": "hello world"
        })).await.unwrap();
        assert!(result.contains("11 bytes"));

        let reader = ReadFileTool;
        let content = reader.execute(json!({
            "path": path.to_str().unwrap()
        })).await.unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_read_nonexistent() {
        let tool = ReadFileTool;
        let result = tool.execute(json!({"path": "/nonexistent/file.txt"})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_dir() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        std::fs::write(dir.path().join("b.txt"), "b").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();

        let tool = ListDirTool;
        let result = tool.execute(json!({"path": dir.path().to_str().unwrap()})).await.unwrap();

        assert!(result.contains("a.txt"));
        assert!(result.contains("b.txt"));
        assert!(result.contains("subdir/"));
    }

    #[tokio::test]
    async fn test_list_nonexistent() {
        let tool = ListDirTool;
        let result = tool.execute(json!({"path": "/nonexistent/dir"})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_creates_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("file.txt");

        let tool = WriteFileTool;
        let result = tool.execute(json!({
            "path": path.to_str().unwrap(),
            "content": "created"
        })).await.unwrap();
        assert!(result.contains("7 bytes"));

        assert!(path.exists());
    }
}

#[cfg(test)]
mod search_tests {
    use crate::tools::GrepTool;
    use crate::registry::ToolInstance;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_grep_found() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("test.rs"), "fn main() {\n    println!(\"hello\");\n}").unwrap();

        let tool = GrepTool;
        let result = tool.execute(json!({
            "pattern": "println",
            "path": dir.path().to_str().unwrap()
        })).await.unwrap();

        assert!(result.contains("println"));
    }

    #[tokio::test]
    async fn test_grep_not_found() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();

        let tool = GrepTool;
        let result = tool.execute(json!({
            "pattern": "nonexistent_pattern_xyz",
            "path": dir.path().to_str().unwrap()
        })).await.unwrap();

        assert!(result.contains("No matches"));
    }
}

#[cfg(test)]
mod registry_tests {
    use crate::registry::ToolRegistry;
    use crate::tools::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_registry_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(BashTool));

        let result = registry.execute("bash", json!({"command": "echo 42"})).await.unwrap();
        assert!(result.contains("42"));
    }

    #[tokio::test]
    async fn test_registry_not_found() {
        let registry = ToolRegistry::new();
        let result = registry.execute("nonexistent", json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_names() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(BashTool));
        registry.register(Box::new(ReadFileTool));

        let names = registry.names();
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"read_file"));
    }

    #[test]
    fn test_registry_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(BashTool));

        let defs = registry.definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "bash");
    }
}

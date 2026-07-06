#[cfg(test)]
mod scaffold_tests {
    use crate::scaffold::CognitiveScaffold;

    #[test]
    fn test_scaffold_creation() {
        let scaffold = CognitiveScaffold::new();
        let prompt = scaffold.create_prompt("Fix bug", "Rust project");
        assert!(!prompt.system_instruction.is_empty());
        assert!(!prompt.user_prompt.is_empty());
    }

    #[test]
    fn test_system_prompt_content() {
        let scaffold = CognitiveScaffold::new();
        let prompt = scaffold.system_prompt();

        assert!(prompt.contains("COGNITIVE SCAFFOLD"));
        assert!(prompt.contains("Phase 1"));
        assert!(prompt.contains("Phase 6"));
        assert!(prompt.contains("NEVER skip"));
    }

    #[test]
    fn test_task_prompt_includes_context() {
        let scaffold = CognitiveScaffold::new();
        let prompt = scaffold.create_prompt("Write a function", "Using tokio");

        assert!(prompt.user_prompt.contains("Write a function"));
        assert!(prompt.user_prompt.contains("Using tokio"));
    }

    #[test]
    fn test_verification_criteria() {
        let scaffold = CognitiveScaffold::new();
        let prompt = scaffold.create_prompt("test", "test");

        assert!(!prompt.verification_criteria.is_empty());
        assert!(prompt.verification_criteria.iter().any(|c| c.contains("compile")));
    }

    #[test]
    fn test_analyze_response_complete() {
        let scaffold = CognitiveScaffold::new();
        let response = r#"
## Analysis

The ACTUAL problem is to handle edge cases.
Dependencies include tokio and serde.
The MINIMAL solution is to use match.

```rust
fn solve() -> Result<(), Error> {
    Ok(())
}
```

It compiles and handles None cases.
There is a simpler way to do this.
"#;

        let analysis = scaffold.analyze_response(response);
        assert!(analysis.phase_score > 0.5);
        assert!(analysis.quality_indicators.has_error_handling);
    }

    #[test]
    fn test_analyze_response_incomplete() {
        let scaffold = CognitiveScaffold::new();
        let response = "Just write the code.";

        let analysis = scaffold.analyze_response(response);
        assert!(analysis.phase_score < 0.3);
        assert!(!analysis.suggestions.is_empty());
    }
}

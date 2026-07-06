#[cfg(test)]
mod diff_tests {
    use crate::diff::{SemanticDiffEngine, ChangeType};

    #[test]
    fn test_diff_empty() {
        let engine = SemanticDiffEngine::new();
        let diff = engine.analyze_diff("", "", "test.rs");
        assert!(diff.changes.is_empty());
        assert!((diff.impact.overall - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_diff_addition() {
        let engine = SemanticDiffEngine::new();
        let diff = engine.analyze_diff("", "fn new() {}", "test.rs");
        assert!(!diff.changes.is_empty());
        assert_eq!(diff.changes[0].change_type, ChangeType::FunctionAdded);
    }

    #[test]
    fn test_diff_deletion() {
        let engine = SemanticDiffEngine::new();
        let diff = engine.analyze_diff("fn old() {}", "", "test.rs");
        assert!(!diff.changes.is_empty());
        assert_eq!(diff.changes[0].change_type, ChangeType::FunctionRemoved);
    }

    #[test]
    fn test_diff_modification() {
        let engine = SemanticDiffEngine::new();
        let old = "fn process() {\n    let x = 1;\n}";
        let new = "fn process() {\n    let x = 2;\n}";
        let diff = engine.analyze_diff(old, new, "test.rs");
        assert!(!diff.changes.is_empty());
        assert_eq!(diff.file, "test.rs");
    }

    #[test]
    fn test_impact_calculation() {
        let engine = SemanticDiffEngine::new();
        let diff = engine.analyze_diff("", "fn main() {}", "main.rs");
        assert!(diff.impact.overall >= 0.0);
        assert!(diff.impact.overall <= 1.0);
    }
}

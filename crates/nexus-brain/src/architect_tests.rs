#[cfg(test)]
mod architect_tests {
    use crate::architect::AutoArchitect;

    #[test]
    fn test_architect_creation() {
        let architect = AutoArchitect::new();
        let report = architect.analyze(&vec![]);
        assert_eq!(report.score, 1.0);
    }

    #[test]
    fn test_detect_god_object() {
        let architect = AutoArchitect::new();
        let code = "fn f1() {}\n".repeat(60);
        let report = architect.analyze(&vec![("big.rs".to_string(), code)]);
        assert!(report.issues.iter().any(|i| i.description.contains("60")));
    }

    #[test]
    fn test_detect_patterns() {
        let architect = AutoArchitect::new();
        let report = architect.analyze(&vec![
            ("builder.rs".to_string(), "struct Builder { } fn builder() {}".to_string()),
        ]);
        assert!(report.patterns.iter().any(|p| p.name.contains("Builder")));
    }

    #[test]
    fn test_suggestions_for_missing_tests() {
        let architect = AutoArchitect::new();
        let report = architect.analyze(&vec![
            ("lib.rs".to_string(), "fn test() {}".to_string()),
        ]);
        assert!(report.suggestions.iter().any(|s| s.pattern.contains("Testing")));
    }

    #[test]
    fn test_score_decreases_with_issues() {
        let architect = AutoArchitect::new();
        let code = "fn f1() {}\n".repeat(60);
        let report = architect.analyze(&vec![("big.rs".to_string(), code)]);
        assert!(report.score < 1.0);
    }
}

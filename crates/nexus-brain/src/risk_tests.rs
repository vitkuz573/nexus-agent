#[cfg(test)]
mod risk_tests {
    use crate::risk::{RiskAnalyzer, RiskLevel};

    #[test]
    fn test_analyzer_creation() {
        let analyzer = RiskAnalyzer::new();
        let report = analyzer.analyze("fn test() {}", "test.rs");
        assert_eq!(report.file, "test.rs");
    }

    #[test]
    fn test_detect_unsafe() {
        let analyzer = RiskAnalyzer::new();
        let report = analyzer.analyze("unsafe { }", "test.rs");
        assert!(report.risks.iter().any(|r| r.severity == RiskLevel::High));
    }

    #[test]
    fn test_detect_unwrap() {
        let analyzer = RiskAnalyzer::new();
        let report = analyzer.analyze("let x = val.unwrap();", "test.rs");
        assert!(report.risks.iter().any(|r| r.description.contains("panic")));
    }

    #[test]
    fn test_clean_code() {
        let analyzer = RiskAnalyzer::new();
        let report = analyzer.analyze("fn safe() -> Result<(), Error> { Ok(()) }", "test.rs");
        assert!(report.risks.is_empty() || report.overall_risk == RiskLevel::Low);
    }

    #[test]
    fn test_recommendations() {
        let analyzer = RiskAnalyzer::new();
        let report = analyzer.analyze("unsafe { let x = secret_key; }", "test.rs");
        assert!(!report.recommendations.is_empty());
    }
}

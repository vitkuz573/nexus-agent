#[cfg(test)]
mod hypothesis_tests {
    use crate::hypothesis::HypothesisEngine;

    #[test]
    fn test_engine_creation() {
        let engine = HypothesisEngine::new();
        assert!(engine.list().is_empty());
    }

    #[test]
    fn test_propose() {
        let mut engine = HypothesisEngine::new();
        let hyp = engine.propose(
            "Test Hypothesis",
            "Which approach is better?",
            "fn approach_a() {}",
            "fn approach_b() {}",
        );

        assert_eq!(hyp.id, "hyp-1");
        assert_eq!(hyp.title, "Test Hypothesis");
        assert_eq!(engine.list().len(), 1);
    }

    #[test]
    fn test_get() {
        let mut engine = HypothesisEngine::new();
        engine.propose("H1", "Desc", "A", "B");

        assert!(engine.get("hyp-1").is_some());
        assert!(engine.get("hyp-999").is_none());
    }

    #[test]
    fn test_multiple_proposals() {
        let mut engine = HypothesisEngine::new();
        engine.propose("H1", "Desc1", "A1", "B1");
        engine.propose("H2", "Desc2", "A2", "B2");

        assert_eq!(engine.list().len(), 2);
        assert_eq!(engine.list()[0].id, "hyp-1");
        assert_eq!(engine.list()[1].id, "hyp-2");
    }
}

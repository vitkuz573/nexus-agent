#[cfg(test)]
mod learner_tests {
    use crate::learner::{AdaptiveLearner, Interaction, InteractionContext, TaskComplexity};

    #[test]
    fn test_learner_creation() {
        let learner = AdaptiveLearner::new();
        let stats = learner.stats();
        assert_eq!(stats.total_interactions, 0);
    }

    #[test]
    fn test_record_interaction() {
        let mut learner = AdaptiveLearner::new();
        let interaction = Interaction {
            id: "1".to_string(),
            task: "write function".to_string(),
            approach: "simple".to_string(),
            tools_used: vec!["bash".to_string()],
            rounds: 3,
            success: true,
            quality_score: 0.9,
            timestamp: 0,
            context: InteractionContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: TaskComplexity::Simple,
                similar_past_tasks: vec![],
            },
        };

        learner.record_interaction(interaction);
        let stats = learner.stats();
        assert_eq!(stats.total_interactions, 1);
        assert!(stats.success_rate > 0.0);
    }

    #[test]
    fn test_suggest_tools() {
        let mut learner = AdaptiveLearner::new();

        for _ in 0..5 {
            let interaction = Interaction {
                id: format!("id-{}", learner.stats().total_interactions),
                task: "read file".to_string(),
                approach: "use read_file".to_string(),
                tools_used: vec!["read_file".to_string()],
                rounds: 2,
                success: true,
                quality_score: 0.9,
                timestamp: 0,
                context: InteractionContext {
                    language: Some("rust".to_string()),
                    framework: None,
                    complexity: TaskComplexity::Simple,
                    similar_past_tasks: vec![],
                },
            };
            learner.record_interaction(interaction);
        }

        let tools = learner.suggest_tools("read file content");
        assert!(tools.contains(&"read_file".to_string()));
    }

    #[test]
    fn test_learn_from_error() {
        let mut learner = AdaptiveLearner::new();
        learner.learn_from_error("unwrap failed", "use match instead");

        let recovery = learner.suggest_recovery("unwrap failed");
        assert_eq!(recovery, Some("use match instead".to_string()));
    }
}

#[cfg(test)]
mod predictor_tests {
    use crate::predictor::SuccessPredictor;

    #[test]
    fn test_predictor_creation() {
        let predictor = SuccessPredictor::new();
        let prediction = predictor.predict("write a function", &vec!["bash".to_string()]);
        assert!(prediction.confidence > 0.0);
    }

    #[test]
    fn test_record_and_predict() {
        let mut predictor = SuccessPredictor::new();

        for _ in 0..10 {
            predictor.record_task(
                "write rust function",
                "use fn",
                &vec!["bash".to_string(), "write_file".to_string()],
                3,
                true,
                0.9,
            );
        }

        let prediction = predictor.predict("write rust function", &vec!["bash".to_string()]);
        assert!(prediction.confidence > 0.5);
    }
}

#[cfg(test)]
mod memory_tests {
    use crate::memory::{LongTermMemory, MemoryCategory};

    #[test]
    fn test_memory_creation() {
        let memory = LongTermMemory::new();
        let stats = memory.stats();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_store_and_recall() {
        let mut memory = LongTermMemory::new();
        memory.store("key1", "value1", MemoryCategory::Decision, 0.9);

        let entry = memory.recall("key1");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().value, "value1");
    }

    #[test]
    fn test_search() {
        let mut memory = LongTermMemory::new();
        memory.store("rust_error", "use Result", MemoryCategory::Pattern, 0.8);
        memory.store("python_error", "use try/except", MemoryCategory::Pattern, 0.7);

        let results = memory.search("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_prune() {
        let mut memory = LongTermMemory::new();
        for i in 0..100 {
            memory.store(&format!("key{}", i), &format!("value{}", i), MemoryCategory::Learning, 0.5);
        }

        memory.prune(50);
        assert!(memory.stats().total_entries <= 50);
    }
}

#[cfg(test)]
mod patterns_tests {
    use crate::patterns::PatternMatcher;

    #[test]
    fn test_matcher_creation() {
        let _matcher = PatternMatcher::new();
        // Just test that the matcher can be created
    }

    #[test]
    fn test_suggest_pattern() {
        let matcher = PatternMatcher::new();
        let pattern = matcher.suggest_pattern("handle errors properly", "rust");
        assert!(pattern.is_some());
    }
}

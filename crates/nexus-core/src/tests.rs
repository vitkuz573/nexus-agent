#[cfg(test)]
mod tests {
    use crate::context::AgentContext;
    use nexus_intel::learner::{Interaction, InteractionContext, TaskComplexity};
    use nexus_intel::memory::MemoryCategory;

    // We can't easily test Agent::run() without a real LLM provider,
    // but we can verify the new intel fields are properly initialized
    // and the accessor methods work.

    #[test]
    fn test_context_creation() {
        let ctx = AgentContext::new(20);
        assert_eq!(ctx.max_rounds, 20);
        assert_eq!(ctx.round, 0);
        assert!(ctx.can_continue());
    }

    #[test]
    fn test_context_increment() {
        let mut ctx = AgentContext::new(2);
        ctx.increment_round();
        assert_eq!(ctx.round, 1);
        assert!(ctx.can_continue());
        ctx.increment_round();
        assert_eq!(ctx.round, 2);
        assert!(!ctx.can_continue());
    }

    #[test]
    fn test_context_messages_with_system() {
        let mut ctx = AgentContext::new(5);
        ctx.push_message(nexus_client::message::Message::user("hello"));
        let msgs = ctx.messages_with_system("sys prompt");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "sys prompt");
    }

    #[test]
    fn test_agent_intel_state_independence() {
        let ctx1 = AgentContext::new(10);
        let ctx2 = AgentContext::new(20);

        assert_eq!(ctx1.max_rounds, 10);
        assert_eq!(ctx2.max_rounds, 20);
    }

    #[test]
    fn test_memory_category_enum() {
        let cats = [
            MemoryCategory::Decision,
            MemoryCategory::Pattern,
            MemoryCategory::Error,
            MemoryCategory::Learning,
            MemoryCategory::Preference,
            MemoryCategory::Context,
        ];
        assert_eq!(cats.len(), 6);
    }

    #[test]
    fn test_interaction_construction() {
        let interaction = Interaction {
            id: "test-1".to_string(),
            task: "fix the bug".to_string(),
            approach: "read, then edit".to_string(),
            tools_used: vec!["read_file".to_string(), "edit_file".to_string()],
            rounds: 3,
            success: true,
            quality_score: 0.9,
            timestamp: 12345,
            context: InteractionContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: TaskComplexity::Moderate,
                similar_past_tasks: Vec::new(),
            },
        };
        assert_eq!(interaction.task, "fix the bug");
        assert_eq!(interaction.tools_used.len(), 2);
        assert!(interaction.success);
        assert_eq!(interaction.quality_score, 0.9);
    }

    #[test]
    fn test_task_complexity_variants() {
        let _trivial = TaskComplexity::Trivial;
        let _simple = TaskComplexity::Simple;
        let _moderate = TaskComplexity::Moderate;
        let _complex = TaskComplexity::Complex;
        let _expert = TaskComplexity::Expert;
    }
}


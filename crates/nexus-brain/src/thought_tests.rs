#[cfg(test)]
mod thought_tests {
    use crate::thought::{ThoughtChain, ThoughtType};

    #[test]
    fn test_thought_chain_creation() {
        let chain = ThoughtChain::new();
        assert!(chain.all_nodes().is_empty());
        assert!(chain.get_current().is_none());
    }

    #[test]
    fn test_add_first_thought() {
        let mut chain = ThoughtChain::new();
        let id = chain.add_thought(ThoughtType::Problem, "What is the issue?", 0.9);

        assert_eq!(id, "thought-1");
        assert_eq!(chain.all_nodes().len(), 1);
        assert!(chain.get_current().is_some());
    }

    #[test]
    fn test_add_multiple_thoughts() {
        let mut chain = ThoughtChain::new();
        chain.add_thought(ThoughtType::Problem, "Problem", 0.9);
        chain.add_thought(ThoughtType::Analysis, "Analysis", 0.85);
        chain.add_thought(ThoughtType::Decision, "Decision", 0.8);

        assert_eq!(chain.all_nodes().len(), 3);

        let current = chain.get_current().unwrap();
        assert_eq!(current.thought_type, ThoughtType::Decision);
    }

    #[test]
    fn test_thought_parent_child() {
        let mut chain = ThoughtChain::new();
        let parent_id = chain.add_thought(ThoughtType::Problem, "Parent", 0.9);
        let child_id = chain.add_thought(ThoughtType::Analysis, "Child", 0.85);

        let parent = chain.get(&parent_id).unwrap();
        assert!(parent.children.contains(&child_id));

        let child = chain.get(&child_id).unwrap();
        assert_eq!(child.parent.as_deref(), Some(parent_id.as_str()));
    }

    #[test]
    fn test_branch_thought() {
        let mut chain = ThoughtChain::new();
        let _main = chain.add_thought(ThoughtType::Problem, "Main", 0.9);
        let analysis = chain.add_thought(ThoughtType::Analysis, "Main analysis", 0.85);

        let branch_id = chain.branch(ThoughtType::Hypothesis, "Alternative", 0.7);

        assert_eq!(chain.all_nodes().len(), 3);
        let branch = chain.get(&branch_id).unwrap();
        // branch's parent is the current node (analysis), not main
        assert_eq!(branch.parent.as_deref(), Some(analysis.as_str()));

        // parent should have branch as child
        let parent = chain.get(&analysis).unwrap();
        assert!(parent.children.contains(&branch_id));
    }

    #[test]
    fn test_trace() {
        let mut chain = ThoughtChain::new();
        chain.add_thought(ThoughtType::Problem, "1", 0.9);
        chain.add_thought(ThoughtType::Analysis, "2", 0.85);
        chain.add_thought(ThoughtType::Decision, "3", 0.8);

        let trace = chain.trace();
        assert_eq!(trace.len(), 3);
        assert_eq!(trace[0].thought_type, ThoughtType::Problem);
        assert_eq!(trace[2].thought_type, ThoughtType::Decision);
    }

    #[test]
    fn test_average_confidence() {
        let mut chain = ThoughtChain::new();
        chain.add_thought(ThoughtType::Problem, "1", 0.9);
        chain.add_thought(ThoughtType::Analysis, "2", 0.7);

        let avg = chain.average_confidence();
        assert!((avg - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_has_low_confidence() {
        let mut chain = ThoughtChain::new();
        chain.add_thought(ThoughtType::Problem, "1", 0.9);
        chain.add_thought(ThoughtType::Analysis, "2", 0.3);

        assert!(chain.has_low_confidence(0.5));
        assert!(!chain.has_low_confidence(0.2));
    }
}

#[cfg(test)]
mod graph_tests {
    use crate::graph::{CodeGraphBuilder, GraphNode, NodeType, GraphEdge, EdgeType};

    #[test]
    fn test_empty_graph() {
        let builder = CodeGraphBuilder::new();
        let graph = builder.build();
        assert_eq!(graph.metrics.total_nodes, 0);
        assert_eq!(graph.metrics.total_edges, 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut builder = CodeGraphBuilder::new();
        builder.add_node(GraphNode {
            id: "n1".to_string(),
            node_type: NodeType::Function,
            name: "main".to_string(),
            file: "main.rs".to_string(),
            line: 1,
            complexity: 0.5,
            dependencies: vec![],
        });

        let graph = builder.build();
        assert_eq!(graph.metrics.total_nodes, 1);
    }

    #[test]
    fn test_add_edges() {
        let mut builder = CodeGraphBuilder::new();
        builder.add_node(GraphNode {
            id: "n1".to_string(),
            node_type: NodeType::Function,
            name: "a".to_string(),
            file: "a.rs".to_string(),
            line: 1,
            complexity: 0.3,
            dependencies: vec![],
        });
        builder.add_node(GraphNode {
            id: "n2".to_string(),
            node_type: NodeType::Function,
            name: "b".to_string(),
            file: "b.rs".to_string(),
            line: 1,
            complexity: 0.4,
            dependencies: vec![],
        });

        builder.add_edge(GraphEdge {
            from: "n1".to_string(),
            to: "n2".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
        });

        let graph = builder.build();
        assert_eq!(graph.metrics.total_nodes, 2);
        assert_eq!(graph.metrics.total_edges, 1);
    }

    #[test]
    fn test_metrics() {
        let mut builder = CodeGraphBuilder::new();
        builder.add_node(GraphNode {
            id: "n1".to_string(),
            node_type: NodeType::Function,
            name: "test".to_string(),
            file: "test.rs".to_string(),
            line: 1,
            complexity: 0.8,
            dependencies: vec![],
        });

        let graph = builder.build();
        assert!((graph.metrics.avg_complexity - 0.8).abs() < 0.01);
        assert_eq!(graph.metrics.max_depth, 0);
    }
}

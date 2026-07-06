use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metrics: GraphMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub file: String,
    pub line: u32,
    pub complexity: f32,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Calls,
    Uses,
    Implements,
    Extends,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub avg_complexity: f32,
    pub max_depth: usize,
    pub cyclomatic_complexity: usize,
    pub coupling: f32,
    pub cohesion: f32,
    pub god_modules: Vec<String>,
    pub hotspots: Vec<String>,
}

pub struct CodeGraphBuilder {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

impl CodeGraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    pub fn build(&self) -> CodeGraph {
        let metrics = self.calculate_metrics();

        CodeGraph {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            metrics,
        }
    }

    fn calculate_metrics(&self) -> GraphMetrics {
        let total_nodes = self.nodes.len();
        let total_edges = self.edges.len();

        let avg_complexity = if total_nodes > 0 {
            self.nodes.iter().map(|n| n.complexity).sum::<f32>() / total_nodes as f32
        } else {
            0.0
        };

        let max_depth = self.calculate_max_depth();
        let cyclomatic_complexity = self.calculate_cyclomatic();
        let coupling = self.calculate_coupling();
        let cohesion = self.calculate_cohesion();
        let god_modules = self.find_god_modules();
        let hotspots = self.find_hotspots();

        GraphMetrics {
            total_nodes,
            total_edges,
            avg_complexity,
            max_depth,
            cyclomatic_complexity,
            coupling,
            cohesion,
            god_modules,
            hotspots,
        }
    }

    fn calculate_max_depth(&self) -> usize {
        let mut visited = HashSet::new();
        let mut max_depth = 0;

        for node in &self.nodes {
            let depth = self.dfs_depth(&node.id, &mut visited, 0);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    fn dfs_depth(&self, node_id: &str, visited: &mut HashSet<String>, depth: usize) -> usize {
        if visited.contains(node_id) {
            return depth;
        }
        visited.insert(node_id.to_string());

        let mut max_child_depth = depth;
        for edge in &self.edges {
            if edge.from == node_id {
                let child_depth = self.dfs_depth(&edge.to, visited, depth + 1);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }

        max_child_depth
    }

    fn calculate_cyclomatic(&self) -> usize {
        let decision_nodes = self.nodes.iter()
            .filter(|n| n.node_type == NodeType::Function)
            .count();
        let edges = self.edges.len();
        let nodes = self.nodes.len();

        if nodes == 0 {
            return 0;
        }

        edges.saturating_sub(nodes) + 2 * decision_nodes
    }

    fn calculate_coupling(&self) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }

        let external_edges: usize = self.edges.iter()
            .filter(|e| e.from != e.to)
            .count();

        (external_edges as f32) / (self.nodes.len() as f32)
    }

    fn calculate_cohesion(&self) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }

        let internal_edges: usize = self.edges.iter()
            .filter(|e| e.from == e.to)
            .count();

        let max_possible = self.nodes.len() * (self.nodes.len() - 1) / 2;
        if max_possible == 0 {
            return 0.0;
        }

        (internal_edges as f32) / (max_possible as f32)
    }

    fn find_god_modules(&self) -> Vec<String> {
        let threshold = self.nodes.len() as f32 * 0.3;
        let mut module_counts: HashMap<String, usize> = HashMap::new();

        for node in &self.nodes {
            *module_counts.entry(node.file.clone()).or_insert(0) += 1;
        }

        module_counts.iter()
            .filter(|(_, &count)| count as f32 > threshold)
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn find_hotspots(&self) -> Vec<String> {
        let mut edge_counts: HashMap<String, usize> = HashMap::new();

        for edge in &self.edges {
            *edge_counts.entry(edge.from.clone()).or_insert(0) += 1;
        }

        let threshold = self.nodes.len() as f32 * 0.2;
        edge_counts.iter()
            .filter(|(_, &count)| count as f32 > threshold)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

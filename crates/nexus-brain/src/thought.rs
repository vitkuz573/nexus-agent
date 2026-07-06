use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtNode {
    pub id: String,
    pub thought_type: ThoughtType,
    pub content: String,
    pub confidence: f32,
    pub children: Vec<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThoughtType {
    Problem,
    Analysis,
    Hypothesis,
    Decision,
    Implementation,
    Verification,
    Reflection,
}

pub struct ThoughtChain {
    nodes: Vec<ThoughtNode>,
    current: Option<String>,
}

impl ThoughtChain {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            current: None,
        }
    }

    pub fn add_thought(&mut self, thought_type: ThoughtType, content: &str, confidence: f32) -> String {
        let id = format!("thought-{}", self.nodes.len() + 1);
        let parent = self.current.clone();

        let node = ThoughtNode {
            id: id.clone(),
            thought_type,
            content: content.to_string(),
            confidence,
            children: Vec::new(),
            parent: parent.clone(),
        };

        if let Some(parent_id) = &parent {
            if let Some(p) = self.nodes.iter_mut().find(|n| &n.id == parent_id) {
                p.children.push(id.clone());
            }
        }

        self.nodes.push(node);
        self.current = Some(id.clone());
        id
    }

    pub fn branch(&mut self, thought_type: ThoughtType, content: &str, confidence: f32) -> String {
        let parent = self.current.clone();
        let id = format!("thought-{}", self.nodes.len() + 1);

        if let Some(parent_id) = &parent {
            if let Some(p) = self.nodes.iter_mut().find(|n| &n.id == parent_id) {
                p.children.push(id.clone());
            }
        }

        let node = ThoughtNode {
            id: id.clone(),
            thought_type,
            content: content.to_string(),
            confidence,
            children: Vec::new(),
            parent,
        };

        self.nodes.push(node);
        id
    }

    pub fn get_current(&self) -> Option<&ThoughtNode> {
        self.current.as_ref().and_then(|id| self.nodes.iter().find(|n| &n.id == id))
    }

    pub fn get(&self, id: &str) -> Option<&ThoughtNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn all_nodes(&self) -> &[ThoughtNode] {
        &self.nodes
    }

    pub fn trace(&self) -> Vec<&ThoughtNode> {
        let mut trace = Vec::new();
        let mut current = self.current.as_ref();

        while let Some(id) = current {
            if let Some(node) = self.nodes.iter().find(|n| &n.id == id) {
                trace.push(node);
                current = node.parent.as_ref();
            } else {
                break;
            }
        }

        trace.reverse();
        trace
    }

    pub fn average_confidence(&self) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        self.nodes.iter().map(|n| n.confidence).sum::<f32>() / self.nodes.len() as f32
    }

    pub fn has_low_confidence(&self, threshold: f32) -> bool {
        self.nodes.iter().any(|n| n.confidence < threshold)
    }
}

impl Default for ThoughtChain {
    fn default() -> Self {
        Self::new()
    }
}

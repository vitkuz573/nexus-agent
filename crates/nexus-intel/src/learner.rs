use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub task: String,
    pub approach: String,
    pub tools_used: Vec<String>,
    pub rounds: usize,
    pub success: bool,
    pub quality_score: f32,
    pub timestamp: u64,
    pub context: InteractionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContext {
    pub language: Option<String>,
    pub framework: Option<String>,
    pub complexity: TaskComplexity,
    pub similar_past_tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskComplexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPattern {
    pub pattern_type: PatternType,
    pub trigger: String,
    pub action: String,
    pub confidence: f32,
    pub success_count: u32,
    pub fail_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ToolPreference,
    ApproachStyle,
    ErrorRecovery,
    Optimization,
    CodeStyle,
}

pub struct AdaptiveLearner {
    interactions: Vec<Interaction>,
    patterns: Vec<LearningPattern>,
    success_rates: HashMap<String, f32>,
    learned_rules: Vec<LearnedRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedRule {
    pub condition: String,
    pub action: String,
    pub confidence: f32,
    pub source: String,
}

impl AdaptiveLearner {
    pub fn new() -> Self {
        Self {
            interactions: Vec::new(),
            patterns: Vec::new(),
            success_rates: HashMap::new(),
            learned_rules: Vec::new(),
        }
    }

    pub fn record_interaction(&mut self, interaction: Interaction) {
        let success = interaction.success;
        let quality = interaction.quality_score;

        // Update success rates for tools used
        for tool in &interaction.tools_used {
            let rate = self.success_rates.entry(tool.clone()).or_insert(0.5);
            if success {
                *rate = (*rate * 0.9) + 0.1;
            } else {
                *rate = *rate * 0.9;
            }
        }

        // Extract patterns from successful interactions
        if success && quality > 0.8 {
            self.extract_patterns(&interaction);
        }

        self.interactions.push(interaction);

        // Keep only last 1000 interactions
        if self.interactions.len() > 1000 {
            self.interactions.drain(0..500);
        }
    }

    fn extract_patterns(&mut self, interaction: &Interaction) {
        // Learn tool preferences
        if !interaction.tools_used.is_empty() {
            let pattern = LearningPattern {
                pattern_type: PatternType::ToolPreference,
                trigger: interaction.task.clone(),
                action: interaction.tools_used.join(", "),
                confidence: interaction.quality_score,
                success_count: 1,
                fail_count: 0,
            };
            self.patterns.push(pattern);
        }

        // Learn approach styles
        let approach_pattern = LearningPattern {
            pattern_type: PatternType::ApproachStyle,
            trigger: format!("{:?}", interaction.context.complexity),
            action: interaction.approach.clone(),
            confidence: interaction.quality_score,
            success_count: 1,
            fail_count: 0,
        };
        self.patterns.push(approach_pattern);
    }

    pub fn suggest_approach(&self, task: &str, _complexity: &TaskComplexity) -> Option<String> {
        // Find similar past tasks
        let similar: Vec<&Interaction> = self.interactions
            .iter()
            .filter(|i| self.tasks_are_similar(&i.task, task))
            .collect();

        if similar.is_empty() {
            return None;
        }

        // Get the best approach from successful interactions
        let best = similar.iter()
            .filter(|i| i.success)
            .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap_or(std::cmp::Ordering::Equal))?;

        Some(best.approach.clone())
    }

    pub fn suggest_tools(&self, task: &str) -> Vec<String> {
        let mut tool_scores: HashMap<String, f32> = HashMap::new();

        for interaction in &self.interactions {
            if self.tasks_are_similar(&interaction.task, task) && interaction.success {
                for tool in &interaction.tools_used {
                    let score = tool_scores.entry(tool.clone()).or_insert(0.0);
                    *score += interaction.quality_score;
                }
            }
        }

        let mut tools: Vec<(String, f32)> = tool_scores.into_iter().collect();
        tools.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        tools.into_iter().take(5).map(|(t, _)| t).collect()
    }

    pub fn get_success_rate(&self, tool: &str) -> f32 {
        self.success_rates.get(tool).copied().unwrap_or(0.5)
    }

    pub fn learn_from_error(&mut self, error: &str, recovery: &str) {
        let pattern = LearningPattern {
            pattern_type: PatternType::ErrorRecovery,
            trigger: error.to_string(),
            action: recovery.to_string(),
            confidence: 0.6,
            success_count: 0,
            fail_count: 0,
        };
        self.patterns.push(pattern);
    }

    pub fn suggest_recovery(&self, error: &str) -> Option<String> {
        self.patterns.iter()
            .filter(|p| matches!(p.pattern_type, PatternType::ErrorRecovery))
            .find(|p| error.contains(&p.trigger) || p.trigger.contains(error))
            .map(|p| p.action.clone())
    }

    fn tasks_are_similar(&self, t1: &str, t2: &str) -> bool {
        let words1: Vec<&str> = t1.split_whitespace().collect();
        let words2: Vec<&str> = t2.split_whitespace().collect();

        let common = words1.iter()
            .filter(|w| words2.iter().any(|w2| w.to_string() == w2.to_string()))
            .count();

        let max_len = words1.len().max(words2.len());
        if max_len == 0 {
            return true;
        }

        (common as f32) / (max_len as f32) > 0.3
    }

    pub fn stats(&self) -> LearnerStats {
        let total = self.interactions.len();
        let successful = self.interactions.iter().filter(|i| i.success).count();
        let avg_quality = if total > 0 {
            self.interactions.iter().map(|i| i.quality_score).sum::<f32>() / total as f32
        } else {
            0.0
        };

        LearnerStats {
            total_interactions: total,
            success_rate: if total > 0 { successful as f32 / total as f32 } else { 0.0 },
            avg_quality,
            patterns_learned: self.patterns.len(),
            rules_learned: self.learned_rules.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerStats {
    pub total_interactions: usize,
    pub success_rate: f32,
    pub avg_quality: f32,
    pub patterns_learned: usize,
    pub rules_learned: usize,
}

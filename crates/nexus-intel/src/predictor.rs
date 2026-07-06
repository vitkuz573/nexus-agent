use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub confidence: f32,
    pub predicted_approach: String,
    pub predicted_tools: Vec<String>,
    pub predicted_rounds: usize,
    pub risk_factors: Vec<String>,
    pub success_probability: f32,
}

pub struct SuccessPredictor {
    historical_data: Vec<HistoricalTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTask {
    pub task_hash: u64,
    pub approach: String,
    pub tools: Vec<String>,
    pub rounds: usize,
    pub success: bool,
    pub quality: f32,
}

impl SuccessPredictor {
    pub fn new() -> Self {
        Self {
            historical_data: Vec::new(),
        }
    }

    pub fn predict(&self, task: &str, available_tools: &[String]) -> Prediction {
        let task_features = self.extract_features(task);
        let similar_tasks = self.find_similar(&task_features);

        if similar_tasks.is_empty() {
            return Prediction {
                confidence: 0.3,
                predicted_approach: "explore".to_string(),
                predicted_tools: available_tools.iter().take(3).cloned().collect(),
                predicted_rounds: 5,
                risk_factors: vec!["No similar past tasks".to_string()],
                success_probability: 0.5,
            };
        }

        let best_task = similar_tasks.iter()
            .max_by(|a, b| a.quality.partial_cmp(&b.quality).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        let avg_rounds: usize = similar_tasks.iter().map(|t| t.rounds).sum::<usize>() / similar_tasks.len();
        let success_rate = similar_tasks.iter().filter(|t| t.success).count() as f32 / similar_tasks.len() as f32;

        let risk_factors = self.identify_risks(task, &similar_tasks);

        Prediction {
            confidence: success_rate,
            predicted_approach: best_task.approach.clone(),
            predicted_tools: best_task.tools.clone(),
            predicted_rounds: avg_rounds,
            risk_factors,
            success_probability: success_rate,
        }
    }

    fn extract_features(&self, task: &str) -> TaskFeatures {
        let words: Vec<&str> = task.split_whitespace().collect();
        let has_code_keywords = words.iter().any(|w| {
            matches!(*w, "fn" | "struct" | "enum" | "impl" | "trait" | "use" | "mod")
        });
        let has_error_keywords = words.iter().any(|w| {
            w.to_lowercase().contains("error") || w.to_lowercase().contains("handle")
        });
        let has_async_keywords = words.iter().any(|w| {
            w.to_lowercase().contains("async") || w.to_lowercase().contains("await")
        });

        TaskFeatures {
            word_count: words.len(),
            has_code_keywords,
            has_error_keywords,
            has_async_keywords,
        }
    }

    fn find_similar(&self, features: &TaskFeatures) -> Vec<&HistoricalTask> {
        self.historical_data.iter()
            .filter(|t| {
                let similarity = self.calculate_similarity(features, &t.task_hash);
                similarity > 0.3
            })
            .collect()
    }

    fn calculate_similarity(&self, _features: &TaskFeatures, _task_hash: &u64) -> f32 {
        // Simplified similarity - in real implementation would use proper hashing
        0.5
    }

    fn identify_risks(&self, task: &str, similar_tasks: &[&HistoricalTask]) -> Vec<String> {
        let mut risks = Vec::new();

        if similar_tasks.len() < 3 {
            risks.push("Limited historical data".to_string());
        }

        let failure_rate = 1.0 - (similar_tasks.iter().filter(|t| t.success).count() as f32 / similar_tasks.len().max(1) as f32);
        if failure_rate > 0.3 {
            risks.push(format!("High failure rate ({:.0}%) for similar tasks", failure_rate * 100.0));
        }

        if task.len() < 10 {
            risks.push("Task may be too vague".to_string());
        }

        risks
    }

    pub fn record_task(&mut self, task: &str, approach: &str, tools: &[String], rounds: usize, success: bool, quality: f32) {
        let features = self.extract_features(task);
        let task_hash = self.hash_task(&features);

        self.historical_data.push(HistoricalTask {
            task_hash,
            approach: approach.to_string(),
            tools: tools.to_vec(),
            rounds,
            success,
            quality,
        });

        // Keep only last 500 tasks
        if self.historical_data.len() > 500 {
            self.historical_data.drain(0..250);
        }
    }

    fn hash_task(&self, features: &TaskFeatures) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        features.word_count.hash(&mut hasher);
        features.has_code_keywords.hash(&mut hasher);
        features.has_error_keywords.hash(&mut hasher);
        features.has_async_keywords.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug, Clone)]
struct TaskFeatures {
    word_count: usize,
    has_code_keywords: bool,
    has_error_keywords: bool,
    has_async_keywords: bool,
}

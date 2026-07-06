use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub title: String,
    pub description: String,
    pub approach_a: Approach,
    pub approach_b: Approach,
    pub status: HypothesisStatus,
    pub results: Option<TestResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approach {
    pub name: String,
    pub code: String,
    pub rationale: String,
    pub estimated_metrics: EstimatedMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedMetrics {
    pub complexity: f32,
    pub performance: f32,
    pub readability: f32,
    pub maintainability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypothesisStatus {
    Proposed,
    Testing,
    Completed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub winner: String,
    pub confidence: f32,
    pub metrics_a: ActualMetrics,
    pub metrics_b: ActualMetrics,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualMetrics {
    pub execution_time_ms: f64,
    pub memory_bytes: u64,
    pub test_pass_rate: f32,
    pub code_coverage: f32,
}

pub struct HypothesisEngine {
    hypotheses: Vec<Hypothesis>,
}

impl HypothesisEngine {
    pub fn new() -> Self {
        Self {
            hypotheses: Vec::new(),
        }
    }

    pub fn propose(
        &mut self,
        title: &str,
        description: &str,
        code_a: &str,
        code_b: &str,
    ) -> &Hypothesis {
        let id = format!("hyp-{}", self.hypotheses.len() + 1);

        let hypothesis = Hypothesis {
            id: id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            approach_a: Approach {
                name: "Approach A".to_string(),
                code: code_a.to_string(),
                rationale: "First approach".to_string(),
                estimated_metrics: self.estimate_metrics(code_a),
            },
            approach_b: Approach {
                name: "Approach B".to_string(),
                code: code_b.to_string(),
                rationale: "Second approach".to_string(),
                estimated_metrics: self.estimate_metrics(code_b),
            },
            status: HypothesisStatus::Proposed,
            results: None,
        };

        self.hypotheses.push(hypothesis);
        self.hypotheses.last().unwrap()
    }

    pub fn evaluate(&mut self, id: &str, results: TestResults) -> Option<&Hypothesis> {
        if let Some(hyp) = self.hypotheses.iter_mut().find(|h| h.id == id) {
            hyp.results = Some(results);
            hyp.status = HypothesisStatus::Completed;
            Some(hyp)
        } else {
            None
        }
    }

    pub fn get(&self, id: &str) -> Option<&Hypothesis> {
        self.hypotheses.iter().find(|h| h.id == id)
    }

    pub fn list(&self) -> &[Hypothesis] {
        &self.hypotheses
    }

    fn estimate_metrics(&self, code: &str) -> EstimatedMetrics {
        let lines = code.lines().count();
        let complexity = (lines as f32 / 10.0).min(1.0);
        let performance = if code.contains("unsafe") { 0.9 } else { 0.7 };
        let readability = if code.contains("//") { 0.8 } else { 0.5 };
        let maintainability = (readability + complexity) / 2.0;

        EstimatedMetrics {
            complexity,
            performance,
            readability,
            maintainability,
        }
    }
}

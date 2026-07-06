use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub id: String,
    pub pattern_type: PatternCategory,
    pub signature: String,
    pub examples: Vec<String>,
    pub success_rate: f32,
    pub context: PatternContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternCategory {
    ErrorHandling,
    Async,
    Builder,
    Factory,
    Observer,
    StateMachine,
    Strategy,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    pub language: String,
    pub framework: Option<String>,
    pub complexity: PatternComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternComplexity {
    Low,
    Medium,
    High,
}

pub struct PatternMatcher {
    patterns: Vec<CodePattern>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            patterns: Self::builtin_patterns(),
        }
    }

    fn builtin_patterns() -> Vec<CodePattern> {
        vec![
            CodePattern {
                id: "rust-error-result".to_string(),
                pattern_type: PatternCategory::ErrorHandling,
                signature: "-> Result<T, E>".to_string(),
                examples: vec![
                    "fn parse(input: &str) -> Result<Data, Error>".to_string(),
                    "fn load(path: &Path) -> Result<Config, io::Error>".to_string(),
                ],
                success_rate: 0.95,
                context: PatternContext {
                    language: "rust".to_string(),
                    framework: None,
                    complexity: PatternComplexity::Low,
                },
            },
            CodePattern {
                id: "rust-async-tokio".to_string(),
                pattern_type: PatternCategory::Async,
                signature: "async fn + tokio".to_string(),
                examples: vec![
                    "async fn fetch(url: &str) -> Result<Response>".to_string(),
                    "#[tokio::main]".to_string(),
                ],
                success_rate: 0.9,
                context: PatternContext {
                    language: "rust".to_string(),
                    framework: Some("tokio".to_string()),
                    complexity: PatternComplexity::Medium,
                },
            },
            CodePattern {
                id: "rust-option-map".to_string(),
                pattern_type: PatternCategory::ErrorHandling,
                signature: ".map()/.and_then() on Option".to_string(),
                examples: vec![
                    "value.map(|v| transform(v))".to_string(),
                    "value.and_then(|v| process(v))".to_string(),
                ],
                success_rate: 0.85,
                context: PatternContext {
                    language: "rust".to_string(),
                    framework: None,
                    complexity: PatternComplexity::Low,
                },
            },
        ]
    }

    pub fn match_code(&self, code: &str, language: &str) -> Vec<&CodePattern> {
        self.patterns.iter()
            .filter(|p| p.context.language == language || p.context.language == "any")
            .filter(|p| code.contains(&p.signature) || self.code_matches_pattern(code, &p.signature))
            .collect()
    }

    fn code_matches_pattern(&self, code: &str, signature: &str) -> bool {
        // Simple heuristic matching
        let sig_words: Vec<&str> = signature.split_whitespace().collect();
        let code_words: Vec<&str> = code.split_whitespace().collect();

        let matches = sig_words.iter()
            .filter(|sw| code_words.iter().any(|cw| cw.contains(*sw)))
            .count();

        matches as f32 / sig_words.len() as f32 > 0.5
    }

    pub fn suggest_pattern(&self, task: &str, language: &str) -> Option<&CodePattern> {
        self.patterns.iter()
            .filter(|p| p.context.language == language || p.context.language == "any")
            .max_by(|a, b| {
                let a_relevance = self.relevance_score(task, a);
                let b_relevance = self.relevance_score(task, b);
                a_relevance.partial_cmp(&b_relevance).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn relevance_score(&self, task: &str, pattern: &CodePattern) -> f32 {
        let task_lower = task.to_lowercase();

        let keywords = match pattern.pattern_type {
            PatternCategory::ErrorHandling => vec!["error", "result", "option", "handle", "parse"],
            PatternCategory::Async => vec!["async", "await", "http", "fetch", "request"],
            PatternCategory::Builder => vec!["builder", "construct", "create", "config"],
            _ => vec![],
        };

        let relevance = keywords.iter()
            .filter(|k| task_lower.contains(*k))
            .count() as f32;

        (relevance / keywords.len() as f32) * pattern.success_rate
    }

    pub fn add_pattern(&mut self, pattern: CodePattern) {
        self.patterns.push(pattern);
    }

    pub fn update_success(&mut self, pattern_id: &str, success: bool) {
        if let Some(pattern) = self.patterns.iter_mut().find(|p| p.id == pattern_id) {
            if success {
                pattern.success_rate = (pattern.success_rate * 0.9) + 0.1;
            } else {
                pattern.success_rate *= 0.9;
            }
        }
    }
}

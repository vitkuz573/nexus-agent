use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: String,
    pub line: u32,
    pub content: String,
    pub score: f32,
    pub context: String,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Semantic,
    Structural,
    Behavioral,
}

pub struct NeuralSearch;

impl NeuralSearch {
    pub fn new() -> Self {
        Self
    }

    pub fn search(&self, query: &str, codebase: &[(String, String)]) -> Vec<SearchResult> {
        let mut results: Vec<SearchResult> = codebase.iter()
            .flat_map(|(file, content)| self.search_in_file(query, file, content))
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(20);
        results
    }

    fn search_in_file(&self, query: &str, file: &str, content: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for (i, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            let exact_score: f32 = if line_lower.contains(&query_lower) {
                1.0
            } else {
                0.0
            };

            let semantic_score = self.semantic_similarity(query, line);
            let structural_score = self.structural_match(query, line);
            let behavioral_score = self.behavioral_match(query, line);

            let max_score = exact_score.max(semantic_score).max(structural_score).max(behavioral_score);

            if max_score > 0.3 {
                let match_type = if exact_score == max_score {
                    MatchType::Exact
                } else if semantic_score == max_score {
                    MatchType::Semantic
                } else if structural_score == max_score {
                    MatchType::Structural
                } else {
                    MatchType::Behavioral
                };

                let context_start = (i as i32 - 2).max(0) as usize;
                let context_end = (i + 3).min(content.lines().count());
                let context: Vec<&str> = content.lines().skip(context_end).take(context_end - context_start).collect();

                results.push(SearchResult {
                    file: file.to_string(),
                    line: (i + 1) as u32,
                    content: line.to_string(),
                    score: max_score,
                    context: context.join("\n"),
                    match_type,
                });
            }
        }

        results
    }

    fn semantic_similarity(&self, query: &str, line: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let line_words: Vec<&str> = line.split_whitespace().collect();

        if query_words.is_empty() || line_words.is_empty() {
            return 0.0;
        }

        let common = query_words.iter()
            .filter(|w| line_words.iter().any(|lw| lw.contains(*w) || w.contains(*lw)))
            .count();

        (common as f32) / (query_words.len().max(line_words.len()) as f32) * 0.8
    }

    fn structural_match(&self, query: &str, line: &str) -> f32 {
        let structural_patterns = ["fn ", "struct ", "enum ", "impl ", "trait ", "pub ", "mod "];
        let query_has_structure = structural_patterns.iter().any(|p| query.contains(p));
        let line_has_structure = structural_patterns.iter().any(|p| line.contains(p));

        if query_has_structure && line_has_structure {
            0.7
        } else if query_has_structure || line_has_structure {
            0.3
        } else {
            0.0
        }
    }

    fn behavioral_match(&self, query: &str, line: &str) -> f32 {
        let behavioral_patterns = ["if ", "else", "match ", "for ", "while ", "loop "];
        let query_has_behavior = behavioral_patterns.iter().any(|p| query.contains(p));
        let line_has_behavior = behavioral_patterns.iter().any(|p| line.contains(p));

        if query_has_behavior && line_has_behavior {
            0.6
        } else {
            0.0
        }
    }

    pub fn search_by_type(&self, type_name: &str, codebase: &[(String, String)]) -> Vec<SearchResult> {
        self.search(&format!("struct {} enum {}", type_name, type_name), codebase)
    }

    pub fn search_by_function(&self, func_name: &str, codebase: &[(String, String)]) -> Vec<SearchResult> {
        self.search(&format!("fn {}", func_name), codebase)
    }
}

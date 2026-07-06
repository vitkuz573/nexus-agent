use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub file: String,
    pub changes: Vec<CodeChange>,
    pub impact: ImpactScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub change_type: ChangeType,
    pub location: CodeLocation,
    pub before: Option<String>,
    pub after: Option<String>,
    pub semantic_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    FunctionAdded,
    FunctionRemoved,
    FunctionModified,
    TypeChanged,
    LogicChanged,
    Refactor,
    Bug,
    Optimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line_start: u32,
    pub line_end: u32,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScore {
    pub complexity: f32,
    pub risk: f32,
    pub performance: f32,
    pub readability: f32,
    pub testability: f32,
    pub overall: f32,
}

pub struct SemanticDiffEngine;

impl SemanticDiffEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_diff(&self, old: &str, new: &str, file: &str) -> SemanticDiff {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let mut changes = Vec::new();
        let mut old_idx = 0;
        let mut new_idx = 0;

        while old_idx < old_lines.len() || new_idx < new_lines.len() {
            if old_idx >= old_lines.len() {
                changes.push(CodeChange {
                    change_type: ChangeType::FunctionAdded,
                    location: CodeLocation {
                        file: file.to_string(),
                        line_start: (new_idx + 1) as u32,
                        line_end: (new_idx + 1) as u32,
                        context: new_lines[new_idx].to_string(),
                    },
                    before: None,
                    after: Some(new_lines[new_idx].to_string()),
                    semantic_weight: self.calculate_weight(new_lines[new_idx]),
                });
                new_idx += 1;
            } else if new_idx >= new_lines.len() {
                changes.push(CodeChange {
                    change_type: ChangeType::FunctionRemoved,
                    location: CodeLocation {
                        file: file.to_string(),
                        line_start: (old_idx + 1) as u32,
                        line_end: (old_idx + 1) as u32,
                        context: old_lines[old_idx].to_string(),
                    },
                    before: Some(old_lines[old_idx].to_string()),
                    after: None,
                    semantic_weight: self.calculate_weight(old_lines[old_idx]),
                });
                old_idx += 1;
            } else if old_lines[old_idx] != new_lines[new_idx] {
                let start_line = (old_idx + 1) as u32;
                let mut end_old = old_idx;
                let mut end_new = new_idx;

                while end_old < old_lines.len() && end_new < new_lines.len()
                    && old_lines[end_old] != new_lines[end_new]
                {
                    end_old += 1;
                    end_new += 1;
                }

                let before = old_lines[old_idx..end_old].join("\n");
                let after = new_lines[new_idx..end_new].join("\n");

                changes.push(CodeChange {
                    change_type: self.classify_change(&before, &after),
                    location: CodeLocation {
                        file: file.to_string(),
                        line_start: start_line,
                        line_end: end_old as u32,
                        context: before.lines().next().unwrap_or("").to_string(),
                    },
                    before: Some(before),
                    after: Some(after),
                    semantic_weight: self.calculate_change_weight(&changes),
                });

                old_idx = end_old;
                new_idx = end_new;
            } else {
                old_idx += 1;
                new_idx += 1;
            }
        }

        let impact = self.calculate_impact(&changes);

        SemanticDiff {
            file: file.to_string(),
            changes,
            impact,
        }
    }

    fn calculate_weight(&self, line: &str) -> f32 {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            0.9
        } else if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") {
            0.85
        } else if trimmed.starts_with("impl ") {
            0.8
        } else if trimmed.contains("unsafe") {
            1.0
        } else if trimmed.contains("unwrap()") || trimmed.contains("expect(") {
            0.7
        } else {
            0.3
        }
    }

    fn classify_change(&self, before: &str, after: &str) -> ChangeType {
        let b = before.trim();
        let a = after.trim();

        if b.starts_with("fn ") && a.starts_with("fn ") {
            ChangeType::FunctionModified
        } else if b.starts_with("fn ") && !a.starts_with("fn ") {
            ChangeType::FunctionRemoved
        } else if !b.starts_with("fn ") && a.starts_with("fn ") {
            ChangeType::FunctionAdded
        } else if b.contains("if ") && a.contains("if ") {
            ChangeType::LogicChanged
        } else if b.contains("let ") && a.contains("let ") {
            ChangeType::TypeChanged
        } else {
            ChangeType::Refactor
        }
    }

    fn calculate_change_weight(&self, changes: &[CodeChange]) -> f32 {
        changes.last().map(|c| c.semantic_weight).unwrap_or(0.5)
    }

    fn calculate_impact(&self, changes: &[CodeChange]) -> ImpactScore {
        if changes.is_empty() {
            return ImpactScore {
                complexity: 0.0,
                risk: 0.0,
                performance: 0.0,
                readability: 0.0,
                testability: 0.0,
                overall: 0.0,
            };
        }

        let total_weight: f32 = changes.iter().map(|c| c.semantic_weight).sum();
        let avg_weight = total_weight / changes.len() as f32;

        let complexity = avg_weight * 0.8;
        let risk = changes.iter()
            .filter(|c| matches!(c.change_type, ChangeType::Bug | ChangeType::LogicChanged))
            .count() as f32 / changes.len() as f32;
        let performance = avg_weight * 0.6;
        let readability = 1.0 - (complexity * 0.5);
        let testability = 1.0 - risk;

        let overall = (complexity + risk + performance + readability + testability) / 5.0;

        ImpactScore {
            complexity,
            risk,
            performance,
            readability,
            testability,
            overall,
        }
    }
}

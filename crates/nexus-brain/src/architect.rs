use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureReport {
    pub score: f32,
    pub issues: Vec<ArchitectureIssue>,
    pub suggestions: Vec<ArchitectureSuggestion>,
    pub patterns: Vec<DetectedPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub location: String,
    pub description: String,
    pub fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    GodObject,
    CircularDependency,
    TightCoupling,
    LowCohesion,
    DeepInheritance,
    FeatureEnvy,
    LongMethod,
    LargeClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureSuggestion {
    pub pattern: String,
    pub description: String,
    pub confidence: f32,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub name: String,
    pub confidence: f32,
    pub locations: Vec<String>,
}

pub struct AutoArchitect;

impl AutoArchitect {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, files: &[(String, String)]) -> ArchitectureReport {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut patterns = Vec::new();

        self.check_god_objects(files, &mut issues);
        self.check_long_methods(files, &mut issues);
        self.check_large_classes(files, &mut issues);
        self.detect_patterns(files, &mut patterns);
        self.suggest_improvements(files, &mut suggestions);

        let score = self.calculate_score(&issues);

        ArchitectureReport {
            score,
            issues,
            suggestions,
            patterns,
        }
    }

    fn check_god_objects(&self, files: &[(String, String)], issues: &mut Vec<ArchitectureIssue>) {
        for (file, content) in files {
            let line_count = content.lines().count();
            let func_count = content.matches("fn ").count();

            if line_count > 500 || func_count > 30 {
                issues.push(ArchitectureIssue {
                    issue_type: IssueType::GodObject,
                    severity: IssueSeverity::Error,
                    location: file.clone(),
                    description: format!("File has {} lines and {} functions", line_count, func_count),
                    fix: "Split into smaller modules with single responsibility".to_string(),
                });
            }
        }
    }

    fn check_long_methods(&self, files: &[(String, String)], issues: &mut Vec<ArchitectureIssue>) {
        for (file, content) in files {
            let mut in_function = false;
            let mut func_start = 0;
            let mut func_name = String::new();

            for (i, line) in content.lines().enumerate() {
                if line.contains("fn ") {
                    in_function = true;
                    func_start = i;
                    func_name = line.trim().to_string();
                } else if in_function && line.trim() == "}" {
                    let length = i - func_start;
                    if length > 50 {
                        issues.push(ArchitectureIssue {
                            issue_type: IssueType::LongMethod,
                            severity: IssueSeverity::Warning,
                            location: format!("{}:{}", file, func_start + 1),
                            description: format!("Function '{}' is {} lines long", func_name, length),
                            fix: "Extract sub-functions or use early returns".to_string(),
                        });
                    }
                    in_function = false;
                }
            }
        }
    }

    fn check_large_classes(&self, files: &[(String, String)], issues: &mut Vec<ArchitectureIssue>) {
        for (file, content) in files {
            let struct_count = content.matches("struct ").count();
            let impl_count = content.matches("impl ").count();

            if struct_count > 10 || impl_count > 10 {
                issues.push(ArchitectureIssue {
                    issue_type: IssueType::LargeClass,
                    severity: IssueSeverity::Warning,
                    location: file.clone(),
                    description: format!("File has {} structs and {} impls", struct_count, impl_count),
                    fix: "Consider splitting into multiple files or using traits".to_string(),
                });
            }
        }
    }

    fn detect_patterns(&self, files: &[(String, String)], patterns: &mut Vec<DetectedPattern>) {
        let mut builder_count = 0;
        let mut factory_count = 0;
        let mut observer_count = 0;

        for (_file, content) in files {
            if content.contains("Builder") || content.contains("builder()") {
                builder_count += 1;
            }
            if content.contains("Factory") || content.contains("create(") {
                factory_count += 1;
            }
            if content.contains("Observer") || content.contains("subscribe(") {
                observer_count += 1;
            }
        }

        if builder_count > 0 {
            patterns.push(DetectedPattern {
                name: "Builder Pattern".to_string(),
                confidence: 0.8,
                locations: files.iter()
                    .filter(|(_, c)| c.contains("Builder") || c.contains("builder()"))
                    .map(|(f, _)| f.clone())
                    .collect(),
            });
        }

        if factory_count > 0 {
            patterns.push(DetectedPattern {
                name: "Factory Pattern".to_string(),
                confidence: 0.7,
                locations: files.iter()
                    .filter(|(_, c)| c.contains("Factory") || c.contains("create("))
                    .map(|(f, _)| f.clone())
                    .collect(),
            });
        }

        if observer_count > 0 {
            patterns.push(DetectedPattern {
                name: "Observer Pattern".to_string(),
                confidence: 0.6,
                locations: files.iter()
                    .filter(|(_, c)| c.contains("Observer") || c.contains("subscribe("))
                    .map(|(f, _)| f.clone())
                    .collect(),
            });
        }
    }

    fn suggest_improvements(&self, files: &[(String, String)], suggestions: &mut Vec<ArchitectureSuggestion>) {
        let has_error_handling = files.iter().any(|(_, c)| c.contains("Result<") || c.contains("Option<"));
        if !has_error_handling {
            suggestions.push(ArchitectureSuggestion {
                pattern: "Error Handling".to_string(),
                description: "Use Result and Option types for proper error handling".to_string(),
                confidence: 0.9,
                examples: vec!["Result<T, E>".to_string(), "Option<T>".to_string()],
            });
        }

        let has_logging = files.iter().any(|(_, c)| c.contains("tracing::") || c.contains("log::"));
        if !has_logging {
            suggestions.push(ArchitectureSuggestion {
                pattern: "Observability".to_string(),
                description: "Add structured logging with tracing".to_string(),
                confidence: 0.85,
                examples: vec!["#[instrument]".to_string(), "tracing::info!()".to_string()],
            });
        }

        let has_tests = files.iter().any(|(_, c)| c.contains("#[test]") || c.contains("#[cfg(test)]"));
        if !has_tests {
            suggestions.push(ArchitectureSuggestion {
                pattern: "Testing".to_string(),
                description: "Add unit tests for critical functions".to_string(),
                confidence: 0.95,
                examples: vec!["#[test]".to_string(), "#[cfg(test)]".to_string()],
            });
        }
    }

    fn calculate_score(&self, issues: &[ArchitectureIssue]) -> f32 {
        if issues.is_empty() {
            return 1.0;
        }

        let total_penalty: f32 = issues.iter().map(|i| match i.severity {
            IssueSeverity::Info => 0.05,
            IssueSeverity::Warning => 0.1,
            IssueSeverity::Error => 0.2,
            IssueSeverity::Critical => 0.4,
        }).sum();

        (1.0 - total_penalty).max(0.0)
    }
}

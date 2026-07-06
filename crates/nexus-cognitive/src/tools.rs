use nexus_tools::error::ToolError;
use nexus_tools::registry::ToolInstance;
use nexus_tools::schema::ToolDefinition;
use async_trait::async_trait;
use nexus_brain::scaffold::CognitiveScaffold;
use nexus_brain::verify::CodeVerifier;
use nexus_brain::risk::RiskAnalyzer;
use nexus_brain::search::NeuralSearch;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref MEMORY: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub struct VerifyCodeTool;

#[async_trait]
impl ToolInstance for VerifyCodeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "verify_code".to_string(),
            description: "Verify code quality: syntax, error handling, edge cases, minimalism. Returns pass/fail with detailed issues.".to_string(),
            parameters: nexus_tools::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![
                    nexus_tools::schema::ToolProperty {
                        name: "code".to_string(),
                        prop_type: "string".to_string(),
                        description: "The code to verify".to_string(),
                    },
                    nexus_tools::schema::ToolProperty {
                        name: "context".to_string(),
                        prop_type: "string".to_string(),
                        description: "Project context (language, framework)".to_string(),
                    },
                ],
                required: vec!["code".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let code = args["code"].as_str().ok_or_else(|| ToolError::InvalidArgs("missing 'code'".to_string()))?;
        let context = args["context"].as_str().unwrap_or("");

        let verifier = CodeVerifier::new();
        let result = verifier.verify(code, context);

        let mut output = format!("Score: {:.0}%\n", result.score * 100.0);
        output.push_str(&format!("Passed: {}\n\n", result.passed));

        for check in &result.checks {
            let status = if check.passed { "✓" } else { "✗" };
            output.push_str(&format!("{} {}\n  {}\n", status, check.name, check.details));
        }

        if !result.issues.is_empty() {
            output.push_str("\nIssues:\n");
            for issue in &result.issues {
                output.push_str(&format!("  • {}\n", issue));
            }
        }

        Ok(output)
    }
}

pub struct AnalyzeRisksTool;

#[async_trait]
impl ToolInstance for AnalyzeRisksTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "analyze_risks".to_string(),
            description: "Analyze code for security risks, performance issues, and potential bugs. Detects unsafe blocks, unwrap() calls, secrets, SQL injection, and more.".to_string(),
            parameters: nexus_tools::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![
                    nexus_tools::schema::ToolProperty {
                        name: "code".to_string(),
                        prop_type: "string".to_string(),
                        description: "The code to analyze".to_string(),
                    },
                    nexus_tools::schema::ToolProperty {
                        name: "file".to_string(),
                        prop_type: "string".to_string(),
                        description: "File path for context".to_string(),
                    },
                ],
                required: vec!["code".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let code = args["code"].as_str().ok_or_else(|| ToolError::InvalidArgs("missing 'code'".to_string()))?;
        let file = args["file"].as_str().unwrap_or("unknown");

        let analyzer = RiskAnalyzer::new();
        let report = analyzer.analyze(code, file);

        let risk_level = format!("{:?}", report.overall_risk);
        let mut output = format!("Overall Risk: {}\n", risk_level);
        output.push_str(&format!("Risks found: {}\n\n", report.risks.len()));

        for risk in &report.risks {
            output.push_str(&format!("[{:?}] {:?}\n  Location: {}\n  {}\n  Mitigation: {}\n\n",
                risk.severity, risk.risk_type, risk.location, risk.description, risk.mitigation));
        }

        if !report.recommendations.is_empty() {
            output.push_str("Recommendations:\n");
            for rec in &report.recommendations {
                output.push_str(&format!("  • {}\n", rec));
            }
        }

        Ok(output)
    }
}

pub struct ThinkTool;

#[async_trait]
impl ToolInstance for ThinkTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "think".to_string(),
            description: "Force structured thinking through a problem. Returns a step-by-step analysis: Problem → Analysis → Design → Implementation plan. Use BEFORE writing code.".to_string(),
            parameters: nexus_tools::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![
                    nexus_tools::schema::ToolProperty {
                        name: "problem".to_string(),
                        prop_type: "string".to_string(),
                        description: "The problem to think through".to_string(),
                    },
                    nexus_tools::schema::ToolProperty {
                        name: "context".to_string(),
                        prop_type: "string".to_string(),
                        description: "Additional context or constraints".to_string(),
                    },
                ],
                required: vec!["problem".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let problem = args["problem"].as_str().ok_or_else(|| ToolError::InvalidArgs("missing 'problem'".to_string()))?;
        let context = args["context"].as_str().unwrap_or("");

        let scaffold = CognitiveScaffold::new();
        let _prompt = scaffold.create_prompt(problem, context);

        let mut output = "## COGNITIVE ANALYSIS\n\n".to_string();
        output.push_str("### Phase 1: UNDERSTAND\n");
        output.push_str("What is the ACTUAL problem?\n");
        output.push_str("What are the constraints?\n");
        output.push_str("What does success look like?\n\n");

        output.push_str("### Phase 2: ANALYZE\n");
        output.push_str("What existing patterns are relevant?\n");
        output.push_str("What could go wrong?\n");
        output.push_str("What are the dependencies?\n\n");

        output.push_str("### Phase 3: DESIGN\n");
        output.push_str("What is the MINIMAL solution?\n");
        output.push_str("What are the tradeoffs?\n");
        output.push_str("Why this approach?\n\n");

        output.push_str("### Phase 4: IMPLEMENT\n");
        output.push_str("[Write code here]\n\n");

        output.push_str("### Phase 5: VERIFY\n");
        output.push_str("Does it compile? Handle errors? Follow patterns?\n\n");

        output.push_str("### Phase 6: REFLECT\n");
        output.push_str("Is there a simpler way? What did we learn?\n\n");

        output.push_str("---\n");
        output.push_str(&format!("Task: {}\n", problem));
        if !context.is_empty() {
            output.push_str(&format!("Context: {}\n", context));
        }

        Ok(output)
    }
}

pub struct SearchCodeTool;

#[async_trait]
impl ToolInstance for SearchCodeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "search_code".to_string(),
            description: "Semantic code search. Finds relevant code by meaning, not just text match. Returns ranked results with context.".to_string(),
            parameters: nexus_tools::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![
                    nexus_tools::schema::ToolProperty {
                        name: "query".to_string(),
                        prop_type: "string".to_string(),
                        description: "What to search for (concept, pattern, or text)".to_string(),
                    },
                    nexus_tools::schema::ToolProperty {
                        name: "path".to_string(),
                        prop_type: "string".to_string(),
                        description: "Directory to search in (default: current)".to_string(),
                    },
                ],
                required: vec!["query".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let query = args["query"].as_str().ok_or_else(|| ToolError::InvalidArgs("missing 'query'".to_string()))?;
        let path = args["path"].as_str().unwrap_or(".");

        let entries = collect_files(path)?;
        let codebase: Vec<(String, String)> = entries.iter()
            .filter_map(|p| {
                let content = std::fs::read_to_string(p).ok()?;
                Some((p.to_string(), content))
            })
            .collect();

        let search = NeuralSearch::new();
        let results = search.search(query, &codebase);

        if results.is_empty() {
            return Ok("No matches found.".to_string());
        }

        let mut output = format!("Found {} results:\n\n", results.len());
        for (i, result) in results.iter().take(10).enumerate() {
            output.push_str(&format!("{}. {}:{} (score: {:.2})\n",
                i + 1, result.file, result.line, result.score));
            output.push_str(&format!("   {}\n\n", result.content));
        }

        Ok(output)
    }
}

pub struct RecallMemoryTool;

#[async_trait]
impl ToolInstance for RecallMemoryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "recall_memory".to_string(),
            description: "Recall information from previous interactions. Stores and retrieves learned patterns, decisions, and context.".to_string(),
            parameters: nexus_tools::schema::ToolParameters {
                param_type: "object".to_string(),
                properties: vec![
                    nexus_tools::schema::ToolProperty {
                        name: "action".to_string(),
                        prop_type: "string".to_string(),
                        description: "'store' or 'recall'".to_string(),
                    },
                    nexus_tools::schema::ToolProperty {
                        name: "key".to_string(),
                        prop_type: "string".to_string(),
                        description: "The key to store/recall".to_string(),
                    },
                    nexus_tools::schema::ToolProperty {
                        name: "value".to_string(),
                        prop_type: "string".to_string(),
                        description: "Value to store (required for 'store')".to_string(),
                    },
                ],
                required: vec!["action".to_string(), "key".to_string()],
            },
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError> {
        let action = args["action"].as_str().ok_or_else(|| ToolError::InvalidArgs("missing 'action'".to_string()))?;
        let key = args["key"].as_str().ok_or_else(|| ToolError::InvalidArgs("missing 'key'".to_string()))?;

        let mut memory = MEMORY.lock().map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        match action {
            "store" => {
                let value = args["value"].as_str().unwrap_or("");
                memory.insert(key.to_string(), value.to_string());
                Ok(format!("Stored '{}'", key))
            }
            "recall" => {
                match memory.get(key) {
                    Some(value) => Ok(format!("{}: {}", key, value)),
                    None => Ok(format!("No memory found for '{}'", key)),
                }
            }
            "list" => {
                let keys: Vec<&str> = memory.keys().map(|s| s.as_str()).collect();
                Ok(format!("Memories: {}", keys.join(", ")))
            }
            _ => Err(ToolError::InvalidArgs(format!("Unknown action: {}", action))),
        }
    }
}

fn collect_files(path: &str) -> Result<Vec<String>, ToolError> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(path).map_err(|e| ToolError::Io(e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name != "target" && name != ".git" && name != "node_modules" {
                files.extend(collect_files(&path.to_string_lossy())?);
            }
        } else if path.extension().map(|e| e == "rs" || e == "toml" || e == "md").unwrap_or(false) {
            files.push(path.to_string_lossy().to_string());
        }
    }

    Ok(files)
}

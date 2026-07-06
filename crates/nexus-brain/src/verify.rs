use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub score: f32,
    pub checks: Vec<VerificationCheck>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeProfile {
    pub lines: usize,
    pub has_functions: bool,
    pub has_structs: bool,
    pub has_unsafe: bool,
    pub has_io: bool,
    pub has_network: bool,
    pub has_user_input: bool,
    pub complexity: CodeComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeComplexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
}

pub struct CodeVerifier;

impl CodeVerifier {
    pub fn new() -> Self {
        Self
    }

    pub fn verify(&self, code: &str, context: &str) -> VerificationResult {
        let profile = self.analyze_profile(code, context);
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        // Universal checks - always apply
        checks.push(self.check_syntax(code));
        checks.push(self.check_minimalism(code, &profile));

        // Context-aware checks
        match profile.complexity {
            CodeComplexity::Trivial => {
                // Trivial code - only syntax matters
            }
            CodeComplexity::Simple => {
                checks.push(self.check_patterns(code, context));
            }
            CodeComplexity::Moderate => {
                checks.push(self.check_patterns(code, context));
                if profile.has_functions {
                    checks.push(self.check_error_handling(code, &profile));
                }
                if profile.has_io {
                    checks.push(self.check_edge_cases(code, &profile));
                }
            }
            CodeComplexity::Complex => {
                checks.push(self.check_patterns(code, context));
                checks.push(self.check_error_handling(code, &profile));
                checks.push(self.check_edge_cases(code, &profile));
                checks.push(self.check_consistency(code, context));
                checks.push(self.check_documentation(code, &profile));
            }
        }

        // Safety checks - always for complex or unsafe code
        if profile.has_unsafe || matches!(profile.complexity, CodeComplexity::Complex) {
            checks.push(self.check_safety(code));
        }

        let passed = checks.iter().all(|c| c.passed);
        let score = if checks.is_empty() {
            1.0
        } else {
            checks.iter().filter(|c| c.passed).count() as f32 / checks.len() as f32
        };

        for check in &checks {
            if !check.passed {
                issues.push(format!("{}: {}", check.name, check.details));
            }
        }

        VerificationResult {
            passed,
            score,
            checks,
            issues,
        }
    }

    fn analyze_profile(&self, code: &str, _context: &str) -> CodeProfile {
        let lines = code.lines().count();
        let has_functions = code.contains("fn ");
        let has_structs = code.contains("struct ") || code.contains("enum ");
        let has_unsafe = code.contains("unsafe");
        let has_io = code.contains("println!") || code.contains("print!")
            || code.contains("std::io") || code.contains("File::")
            || code.contains("read_to_string") || code.contains("write_all");
        let has_network = code.contains("reqwest") || code.contains("hyper")
            || code.contains("tcp") || code.contains("http");
        let has_user_input = code.contains("stdin") || code.contains("read_line")
            || code.contains("args()");

        let complexity = if lines < 5 && !has_functions {
            CodeComplexity::Trivial
        } else if lines < 20 && !has_structs {
            CodeComplexity::Simple
        } else if lines < 100 {
            CodeComplexity::Moderate
        } else {
            CodeComplexity::Complex
        };

        CodeProfile {
            lines,
            has_functions,
            has_structs,
            has_unsafe,
            has_io,
            has_network,
            has_user_input,
            complexity,
        }
    }

    fn check_syntax(&self, code: &str) -> VerificationCheck {
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();

        let balanced = open_braces == close_braces && open_parens == close_parens;
        let has_content = code.contains("fn ") || code.contains("struct ")
            || code.contains("let ") || code.contains("use ");

        let passed = balanced && has_content;

        VerificationCheck {
            name: "Syntax Structure".to_string(),
            passed,
            details: if passed {
                "Balanced braces and parens, has content".to_string()
            } else if !balanced {
                format!("Unbalanced: {} open, {} close braces", open_braces, close_braces)
            } else {
                "No recognizable Rust syntax".to_string()
            },
        }
    }

    fn check_patterns(&self, code: &str, context: &str) -> VerificationCheck {
        let uses_result = code.contains("Result<");
        let uses_option = code.contains("Option<");
        let _uses_match = code.contains("match ");

        let context_has_result = context.contains("Result") || context.contains("error");
        let context_has_option = context.contains("Option") || context.contains("nullable");

        let mut issues = Vec::new();

        if context_has_result && !uses_result && !code.contains("fn main") {
            issues.push("Expected Result usage based on context".to_string());
        }
        if context_has_option && !uses_option {
            issues.push("Expected Option usage based on context".to_string());
        }

        let passed = issues.is_empty();

        VerificationCheck {
            name: "Pattern Consistency".to_string(),
            passed,
            details: if passed {
                "Follows expected patterns".to_string()
            } else {
                issues.join("; ")
            },
        }
    }

    fn check_error_handling(&self, code: &str, _profile: &CodeProfile) -> VerificationCheck {
        let has_unwraps = code.contains("unwrap()");
        let has_expects = code.contains("expect(");
        let has_question = code.contains('?');

        let has_error_handling = code.contains("Result<")
            || code.contains("Option<")
            || code.contains("match ")
            || code.contains("if let ")
            || has_question;

        // For functions that return Result, require error handling
        let returns_result = code.contains("-> Result<");

        let passed = if returns_result {
            has_error_handling && !has_unwraps && !has_expects
        } else if has_unwraps || has_expects {
            false
        } else {
            true
        };

        VerificationCheck {
            name: "Error Handling".to_string(),
            passed,
            details: if passed {
                "Appropriate error handling".to_string()
            } else if has_unwraps {
                "Uses unwrap() - may panic".to_string()
            } else if has_expects {
                "Uses expect() - may panic".to_string()
            } else if returns_result {
                "Returns Result but doesn't handle errors".to_string()
            } else {
                "Error handling present".to_string()
            },
        }
    }

    fn check_edge_cases(&self, code: &str, profile: &CodeProfile) -> VerificationCheck {
        let mut score = 0.0;
        let mut total = 0.0;

        // Only check what's relevant
        if code.contains("Vec") || code.contains("HashMap") || code.contains("String") {
            total += 1.0;
            if code.contains("is_empty()") || code.contains("len()") {
                score += 1.0;
            }
        }

        if code.contains("Option") || code.contains("Result") {
            total += 1.0;
            if code.contains("Some") || code.contains("None") || code.contains("Ok") || code.contains("Err") {
                score += 1.0;
            }
        }

        if profile.has_user_input {
            total += 1.0;
            if code.contains("trim") || !code.is_empty() {
                score += 1.0;
            }
        }

        let passed = if total == 0.0 {
            true // No applicable edge cases
        } else {
            score / total >= 0.5
        };

        VerificationCheck {
            name: "Edge Cases".to_string(),
            passed,
            details: if total == 0.0 {
                "No applicable edge cases".to_string()
            } else {
                format!("Handles {:.0}% of applicable edge cases", (score / total) * 100.0)
            },
        }
    }

    fn check_minimalism(&self, code: &str, profile: &CodeProfile) -> VerificationCheck {
        let has_todo = code.contains("TODO");
        let has_fixme = code.contains("FIXME");
        let has_hack = code.contains("HACK");

        let complexity_penalty = match profile.complexity {
            CodeComplexity::Trivial | CodeComplexity::Simple => 0.0,
            CodeComplexity::Moderate => 0.1,
            CodeComplexity::Complex => 0.2,
        };

        let line_penalty = if profile.lines > 200 {
            0.3
        } else if profile.lines > 100 {
            0.15
        } else {
            0.0
        };

        let score = 1.0 - complexity_penalty - line_penalty;
        let passed = score > 0.6 && !has_todo && !has_fixme && !has_hack;

        VerificationCheck {
            name: "Minimalism".to_string(),
            passed,
            details: if passed {
                format!("Clean solution ({} lines)", profile.lines)
            } else if has_todo || has_fixme || has_hack {
                "Contains TODO/FIXME/HACK markers".to_string()
            } else {
                format!("May be over-complex ({} lines)", profile.lines)
            },
        }
    }

    fn check_consistency(&self, code: &str, context: &str) -> VerificationCheck {
        let code_async = code.contains("async") || code.contains("await");
        let context_async = context.contains("async") || context.contains("tokio");

        let code_serde = code.contains("Serialize") || code.contains("Deserialize");
        let context_serde = context.contains("serde") || context.contains("Serialize");

        let mut issues = Vec::new();

        if context_async && !code_async && code.contains("fn ") && !code.contains("fn main") {
            issues.push("Project uses async but function is sync".to_string());
        }
        if context_serde && !code_serde && code.contains("struct ") {
            issues.push("Project uses serde but struct lacks derives".to_string());
        }

        let passed = issues.is_empty();

        VerificationCheck {
            name: "Consistency".to_string(),
            passed,
            details: if passed {
                "Consistent with project patterns".to_string()
            } else {
                issues.join("; ")
            },
        }
    }

    fn check_documentation(&self, code: &str, profile: &CodeProfile) -> VerificationCheck {
        let has_doc_comments = code.contains("///") || code.contains("//!");
        let has_module_doc = code.contains("//!");

        let needs_docs = profile.has_functions && profile.lines > 20;
        let passed = !needs_docs || has_doc_comments || has_module_doc;

        VerificationCheck {
            name: "Documentation".to_string(),
            passed,
            details: if passed {
                if needs_docs {
                    "Has documentation".to_string()
                } else {
                    "Documentation not required".to_string()
                }
            } else {
                "Public API should be documented".to_string()
            },
        }
    }

    fn check_safety(&self, code: &str) -> VerificationCheck {
        let has_unsafe = code.contains("unsafe");
        let has_safety_comment = code.contains("// SAFETY:") || code.contains("// Safety:");

        let passed = !has_unsafe || has_safety_comment;

        VerificationCheck {
            name: "Safety Documentation".to_string(),
            passed,
            details: if passed {
                if has_unsafe {
                    "Unsafe blocks are documented".to_string()
                } else {
                    "No unsafe code".to_string()
                }
            } else {
                "Unsafe block missing SAFETY comment".to_string()
            },
        }
    }
}

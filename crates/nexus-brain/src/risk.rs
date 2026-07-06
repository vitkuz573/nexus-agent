use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskReport {
    pub file: String,
    pub risks: Vec<Risk>,
    pub overall_risk: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub risk_type: RiskType,
    pub severity: RiskLevel,
    pub location: String,
    pub description: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskType {
    Security,
    Performance,
    Reliability,
    Maintainability,
    Scalability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct RiskAnalyzer;

impl RiskAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, code: &str, file: &str) -> RiskReport {
        let mut risks = Vec::new();

        self.check_unsafe(code, file, &mut risks);
        self.check_unwrap(code, file, &mut risks);
        self.check_memory_leaks(code, file, &mut risks);
        self.check_infinite_loops(code, file, &mut risks);
        self.check_hardcoded_secrets(code, file, &mut risks);
        self.check_unchecked_errors(code, file, &mut risks);
        self.check_deadlock_risk(code, file, &mut risks);
        self.check_sql_injection(code, file, &mut risks);

        let overall_risk = self.calculate_overall_risk(&risks);
        let recommendations = self.generate_recommendations(&risks);

        RiskReport {
            file: file.to_string(),
            risks,
            overall_risk,
            recommendations,
        }
    }

    fn check_unsafe(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains("unsafe") {
                risks.push(Risk {
                    risk_type: RiskType::Security,
                    severity: RiskLevel::High,
                    location: format!("{}:{}", file, i + 1),
                    description: "Unsafe block detected".to_string(),
                    mitigation: "Review unsafe code and add safety comments".to_string(),
                });
            }
        }
    }

    fn check_unwrap(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains("unwrap()") || line.contains("expect(") {
                risks.push(Risk {
                    risk_type: RiskType::Reliability,
                    severity: RiskLevel::Medium,
                    location: format!("{}:{}", file, i + 1),
                    description: "Potential panic point".to_string(),
                    mitigation: "Use proper error handling with match or ?".to_string(),
                });
            }
        }
    }

    fn check_memory_leaks(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        if code.contains("Box::leak") || code.contains("Vec::leak") {
            risks.push(Risk {
                risk_type: RiskType::Reliability,
                severity: RiskLevel::High,
                location: file.to_string(),
                description: "Intentional memory leak detected".to_string(),
                mitigation: "Ensure this is intentional and won't cause OOM".to_string(),
            });
        }
    }

    fn check_infinite_loops(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        for (i, line) in code.lines().enumerate() {
            if line.trim().starts_with("loop") && !code.contains("break") {
                risks.push(Risk {
                    risk_type: RiskType::Reliability,
                    severity: RiskLevel::Medium,
                    location: format!("{}:{}", file, i + 1),
                    description: "Potential infinite loop".to_string(),
                    mitigation: "Add break condition or timeout".to_string(),
                });
            }
        }
    }

    fn check_hardcoded_secrets(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        let patterns = ["password", "secret", "api_key", "token", "private_key"];
        for (i, line) in code.lines().enumerate() {
            for pattern in &patterns {
                if line.to_lowercase().contains(pattern) && line.contains('=') {
                    risks.push(Risk {
                        risk_type: RiskType::Security,
                        severity: RiskLevel::Critical,
                        location: format!("{}:{}", file, i + 1),
                        description: format!("Possible hardcoded {}", pattern),
                        mitigation: "Use environment variables or secret management".to_string(),
                    });
                }
            }
        }
    }

    fn check_unchecked_errors(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        for (i, line) in code.lines().enumerate() {
            if line.contains(".ok()") && line.contains('?') {
                risks.push(Risk {
                    risk_type: RiskType::Reliability,
                    severity: RiskLevel::Low,
                    location: format!("{}:{}", file, i + 1),
                    description: "Error silently ignored".to_string(),
                    mitigation: "Log the error or propagate it".to_string(),
                });
            }
        }
    }

    fn check_deadlock_risk(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        let lock_count = code.matches(".lock()").count();
        if lock_count > 1 {
            risks.push(Risk {
                risk_type: RiskType::Reliability,
                severity: RiskLevel::High,
                location: file.to_string(),
                description: format!("Multiple locks detected ({})", lock_count),
                mitigation: "Use consistent lock ordering or reduce lock scope".to_string(),
            });
        }
    }

    fn check_sql_injection(&self, code: &str, file: &str, risks: &mut Vec<Risk>) {
        for (i, line) in code.lines().enumerate() {
            if (line.contains("format!") || line.contains("concat!"))
                && (line.contains("SELECT") || line.contains("INSERT") || line.contains("UPDATE"))
            {
                risks.push(Risk {
                    risk_type: RiskType::Security,
                    severity: RiskLevel::Critical,
                    location: format!("{}:{}", file, i + 1),
                    description: "Possible SQL injection".to_string(),
                    mitigation: "Use parameterized queries".to_string(),
                });
            }
        }
    }

    fn calculate_overall_risk(&self, risks: &[Risk]) -> RiskLevel {
        if risks.iter().any(|r| r.severity == RiskLevel::Critical) {
            RiskLevel::Critical
        } else if risks.iter().any(|r| r.severity == RiskLevel::High) {
            RiskLevel::High
        } else if risks.iter().any(|r| r.severity == RiskLevel::Medium) {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn generate_recommendations(&self, risks: &[Risk]) -> Vec<String> {
        let mut recs = Vec::new();

        if risks.iter().any(|r| r.risk_type == RiskType::Security) {
            recs.push("Run security audit before deployment".to_string());
        }
        if risks.iter().any(|r| r.risk_type == RiskType::Performance) {
            recs.push("Profile performance hotspots".to_string());
        }
        if risks.iter().any(|r| r.risk_type == RiskType::Reliability) {
            recs.push("Add comprehensive error handling".to_string());
        }
        if risks.len() > 5 {
            recs.push("Consider refactoring this module".to_string());
        }

        recs
    }
}

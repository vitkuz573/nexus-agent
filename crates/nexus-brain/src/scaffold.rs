use crate::thought::ThoughtType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldPrompt {
    pub phase: ScaffoldPhase,
    pub system_instruction: String,
    pub user_prompt: String,
    pub required_thoughts: Vec<ThoughtType>,
    pub verification_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaffoldPhase {
    Understand,
    Analyze,
    Design,
    Implement,
    Verify,
    Reflect,
}

pub struct CognitiveScaffold;

impl CognitiveScaffold {
    pub fn new() -> Self {
        Self
    }

    pub fn create_prompt(&self, task: &str, context: &str) -> ScaffoldPrompt {
        ScaffoldPrompt {
            phase: ScaffoldPhase::Understand,
            system_instruction: self.system_prompt(),
            user_prompt: self.build_task_prompt(task, context),
            required_thoughts: vec![
                ThoughtType::Problem,
                ThoughtType::Analysis,
                ThoughtType::Hypothesis,
                ThoughtType::Decision,
                ThoughtType::Implementation,
                ThoughtType::Verification,
                ThoughtType::Reflection,
            ],
            verification_criteria: self.verification_criteria(),
        }
    }

    pub fn system_prompt(&self) -> String {
        r#"You are a cognitive coding agent. You MUST follow this thinking process for EVERY task:

## COGNITIVE SCAFFOLD PROTOCOL

### Phase 1: UNDERSTAND
Before writing ANY code, explicitly state:
- What is the ACTUAL problem (not just the surface request)?
- What are the constraints and edge cases?
- What does success look like?

### Phase 2: ANALYZE
Break down the problem:
- What existing code/patterns are relevant?
- What are the dependencies?
- What could go wrong?

### Phase 3: DESIGN
Before implementation:
- What is the MINIMAL solution?
- What are the tradeoffs?
- Why this approach over alternatives?

### Phase 4: IMPLEMENT
Write code that:
- Solves the ACTUAL problem
- Handles edge cases
- Follows existing patterns
- Is minimal and clean

### Phase 5: VERIFY
After writing code:
- Does it compile?
- Does it handle errors?
- Does it follow the patterns?
- Is it testable?

### Phase 6: REFLECT
Ask yourself:
- Is there a simpler way?
- What did I learn?
- What should I remember?

RULES:
- NEVER skip phases
- ALWAYS explain your reasoning
- ALWAYS verify your work
- If uncertain, say so explicitly
- Prefer minimal solutions over clever ones"#
            .to_string()
    }

    fn build_task_prompt(&self, task: &str, context: &str) -> String {
        format!(
            r#"## TASK
{task}

## CONTEXT
{context}

## COGNITIVE SCAFFOLD
Follow the cognitive scaffold protocol. Think step by step before implementing.

### Your Analysis:
[Fill in Phase 1-3 analysis here]

### Your Solution:
[Provide implementation]

### Your Verification:
[Verify your solution]"#
        )
    }

    fn verification_criteria(&self) -> Vec<String> {
        vec![
            "Code compiles without errors".to_string(),
            "All edge cases are handled".to_string(),
            "Error handling is present".to_string(),
            "Code follows existing patterns".to_string(),
            "Solution is minimal".to_string(),
            "No unnecessary complexity".to_string(),
        ]
    }

    pub fn analyze_response(&self, response: &str) -> ScaffoldAnalysis {
        let has_understanding = response.contains("ACTUAL problem")
            || response.contains("constraints")
            || response.contains("success looks like");

        let has_analysis = response.contains("dependencies")
            || response.contains("could go wrong")
            || response.contains("existing");

        let has_design = response.contains("MINIMAL solution")
            || response.contains("tradeoffs")
            || response.contains("approach");

        let has_implementation = response.contains("```")
            || response.contains("fn ")
            || response.contains("struct ");

        let has_verification = response.contains("compiles")
            || response.contains("handles")
            || response.contains("testable");

        let has_reflection = response.contains("simpler way")
            || response.contains("learn")
            || response.contains("remember");

        let phase_score = [
            has_understanding,
            has_analysis,
            has_design,
            has_implementation,
            has_verification,
            has_reflection,
        ]
        .iter()
        .filter(|&&x| x)
        .count() as f32
            / 6.0;

        let quality_indicators = self.count_quality_indicators(response);

        ScaffoldAnalysis {
            phase_score,
            quality_indicators,
            suggestions: self.generate_suggestions(
                has_understanding,
                has_analysis,
                has_design,
                has_implementation,
                has_verification,
                has_reflection,
            ),
        }
    }

    fn count_quality_indicators(&self, response: &str) -> QualityIndicators {
        QualityIndicators {
            has_error_handling: response.contains("Result<") || response.contains("Option<") || response.contains("match"),
            has_edge_cases: response.contains("edge case") || response.contains("None") || response.contains("empty"),
            has_documentation: response.contains("///") || response.contains("//!") || response.contains("///"),
            has_minimal_solution: !response.contains("TODO") && !response.contains("FIXME"),
            has_reasoning: response.contains("because") || response.contains("therefore") || response.contains("since"),
        }
    }

    fn generate_suggestions(
        &self,
        understanding: bool,
        analysis: bool,
        design: bool,
        implementation: bool,
        verification: bool,
        reflection: bool,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if !understanding {
            suggestions.push("Add explicit problem analysis".to_string());
        }
        if !analysis {
            suggestions.push("Include dependency and risk analysis".to_string());
        }
        if !design {
            suggestions.push("Explain the design rationale".to_string());
        }
        if !implementation {
            suggestions.push("Provide actual code implementation".to_string());
        }
        if !verification {
            suggestions.push("Add verification steps".to_string());
        }
        if !reflection {
            suggestions.push("Include reflection on the solution".to_string());
        }

        suggestions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldAnalysis {
    pub phase_score: f32,
    pub quality_indicators: QualityIndicators,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIndicators {
    pub has_error_handling: bool,
    pub has_edge_cases: bool,
    pub has_documentation: bool,
    pub has_minimal_solution: bool,
    pub has_reasoning: bool,
}

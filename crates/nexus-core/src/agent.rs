use crate::context::AgentContext;
use crate::error::CoreError;
use crate::events::AgentEvent;
use crate::memory::Memory;
use nexus_brain::scaffold::CognitiveScaffold;
use nexus_brain::verify::CodeVerifier;
use nexus_brain::thought::{ThoughtChain, ThoughtType};
use nexus_client::message::{Message, ToolCall};
use nexus_client::provider::{ChatResponse, Choice, LlmProvider, ResponseMessage};
use nexus_intel::learner::{AdaptiveLearner, Interaction, InteractionContext, TaskComplexity};
use nexus_intel::memory::{LongTermMemory, MemoryCategory};
use nexus_intel::predictor::SuccessPredictor;
use nexus_tools::registry::ToolRegistry;
use std::sync::Arc;
use tracing::{info, debug, warn};

pub struct Agent {
    provider: Arc<LlmProvider>,
    registry: Arc<ToolRegistry>,
    context: AgentContext,
    memory: Memory,
    system_prompt: String,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    scaffold: CognitiveScaffold,
    verifier: CodeVerifier,
    thought_chains: Vec<ThoughtChain>,
    learner: AdaptiveLearner,
    predictor: SuccessPredictor,
    long_term_memory: LongTermMemory,
    tools_used_in_run: Vec<String>,
}

impl Agent {
    pub fn new(
        provider: Arc<LlmProvider>,
        registry: Arc<ToolRegistry>,
        system_prompt: String,
        model: String,
        max_rounds: usize,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            provider,
            registry,
            context: AgentContext::new(max_rounds),
            memory: Memory::default(),
            system_prompt,
            model,
            max_tokens,
            temperature,
            scaffold: CognitiveScaffold::new(),
            verifier: CodeVerifier::new(),
            thought_chains: Vec::new(),
            learner: AdaptiveLearner::new(),
            predictor: SuccessPredictor::new(),
            long_term_memory: LongTermMemory::new(),
            tools_used_in_run: Vec::new(),
        }
    }

    pub async fn run(&mut self, user_input: &str) -> Result<String, CoreError> {
        let mut chain = ThoughtChain::new();
        chain.add_thought(ThoughtType::Problem, user_input, 1.0);

        // Predict success BEFORE starting (uses historical data)
        let available_tools: Vec<String> = self.registry.definitions()
            .iter()
            .map(|d| d.name.clone())
            .collect();
        let prediction = self.predictor.predict(user_input, &available_tools);
        chain.add_thought(
            ThoughtType::Analysis,
            &format!("Predicted success: {:.0}%, suggested tools: {:?}, risks: {:?}",
                prediction.success_probability * 100.0,
                prediction.predicted_tools,
                prediction.risk_factors),
            prediction.confidence,
        );

        // Get approach suggestion from learned patterns
        let approach_hint = self.learner.suggest_approach(user_input, &TaskComplexity::Moderate);
        let _approach_hint = approach_hint.as_deref();

        let enhanced_prompt = self.build_enhanced_prompt(user_input);
        let user_msg = Message::user(&enhanced_prompt);
        self.memory.add(user_msg.clone());
        self.context.push_message(user_msg);

        let tool_schemas: Vec<nexus_client::provider::ToolSchema> = self
            .registry
            .definitions()
            .iter()
            .map(|def| nexus_client::provider::ToolSchema {
                schema_type: "function".to_string(),
                function: nexus_client::provider::FunctionSchema {
                    name: def.name.clone(),
                    description: def.description.clone(),
                    parameters: def.to_json_schema(),
                },
            })
            .collect();

        let mut _final_response = None;
        let mut run_success = false;
        let mut final_quality = 0.0;

        loop {
            self.context.increment_round();
            if !self.context.can_continue() {
                warn!(round = self.context.round, "max rounds reached");
                break;
            }

            let messages = self.context.messages_with_system(&self.system_prompt);
            debug!(round = self.context.round, "calling LLM");

            let response = self
                .provider
                .complete(
                    &self.model,
                    &messages,
                    Some(&tool_schemas),
                    self.max_tokens,
                    self.temperature,
                )
                .await?;

            let choice = response.choices.first().ok_or(CoreError::EmptyResponse)?;
            let resp_msg = &choice.message;

            if let Some(tool_calls) = &resp_msg.tool_calls {
                let assistant_content = resp_msg.content.clone().unwrap_or_default();
                if !assistant_content.is_empty() {
                    chain.add_thought(ThoughtType::Analysis, &assistant_content, 0.9);
                    info!(content = %assistant_content, "assistant thinking");
                }

                self.context.push_message(Message {
                    role: nexus_client::message::Role::Assistant,
                    content: assistant_content,
                    tool_calls: Some(tool_calls.clone()),
                    tool_call_id: None,
                });

                for tc in tool_calls {
                    let tool_name = tc.function.name.clone();
                    self.tools_used_in_run.push(tool_name);
                    self.execute_tool_call(tc).await?;
                }
            } else {
                let content = resp_msg.content.clone().unwrap_or_default();
                chain.add_thought(ThoughtType::Reflection, &content, 0.95);

                let verification = self.verify_response(&content, user_input);
                if !verification.passed {
                    warn!(issues = ?verification.issues, "response verification failed");
                    chain.add_thought(
                        ThoughtType::Verification,
                        &format!("Issues: {:?}", verification.issues),
                        0.5,
                    );
                    final_quality = verification.score;
                } else {
                    final_quality = if verification.score > 0.0 { verification.score } else { 0.85 };
                }

                run_success = verification.passed || verification.score >= 0.5;
                _final_response = Some(content);
                break;
            }
        }

        self.thought_chains.push(chain);

        // Record interaction in learner
        let interaction = Interaction {
            id: format!("{:x}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()),
            task: user_input.to_string(),
            approach: "agent-loop".to_string(),
            tools_used: self.tools_used_in_run.drain(..).collect(),
            rounds: self.context.round,
            success: run_success,
            quality_score: final_quality,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            context: InteractionContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: TaskComplexity::Moderate,
                similar_past_tasks: Vec::new(),
            },
        };
        self.learner.record_interaction(interaction);

        // Record in predictor too
        self.predictor.record_task(
            user_input,
            "agent-loop",
            &self.tools_used_in_run.iter().cloned().collect::<Vec<_>>(),
            self.context.round,
            run_success,
            final_quality,
        );

        // Store important learnings in long-term memory
        if final_quality > 0.8 {
            let key = format!("success:{}", self.context.round);
            self.long_term_memory.store(
                &key,
                user_input,
                MemoryCategory::Learning,
                final_quality,
            );
        }

        let response = _final_response.ok_or(CoreError::EmptyResponse)?;

        let final_msg = Message::assistant(&response);
        self.memory.add(final_msg.clone());
        self.context.push_message(final_msg);

        Ok(response)
    }

    /// Run the agent loop, calling `on_event` for each meaningful event.
    ///
    /// Tokens are streamed only for the final assistant response (after
    /// all tool-call rounds have completed). During tool-call rounds the
    /// model returns tool_calls only, so no tokens are emitted.
    pub async fn run_streaming<F>(
        &mut self,
        user_input: &str,
        mut on_event: F,
    ) -> Result<String, CoreError>
    where
        F: FnMut(AgentEvent),
    {
        let mut chain = ThoughtChain::new();
        chain.add_thought(ThoughtType::Problem, user_input, 1.0);

        let available_tools: Vec<String> = self
            .registry
            .definitions()
            .iter()
            .map(|d| d.name.clone())
            .collect();
        let prediction = self.predictor.predict(user_input, &available_tools);

        on_event(AgentEvent::Predicted {
            confidence: prediction.confidence,
            approach: prediction.predicted_approach.clone(),
            risks: prediction.risk_factors.clone(),
        });

        let enhanced_prompt = self.build_enhanced_prompt(user_input);
        let user_msg = Message::user(&enhanced_prompt);
        self.memory.add(user_msg.clone());
        self.context.push_message(user_msg);

        let tool_schemas: Vec<nexus_client::provider::ToolSchema> = self
            .registry
            .definitions()
            .iter()
            .map(|def| nexus_client::provider::ToolSchema {
                schema_type: "function".to_string(),
                function: nexus_client::provider::FunctionSchema {
                    name: def.name.clone(),
                    description: def.description.clone(),
                    parameters: def.to_json_schema(),
                },
            })
            .collect();

        let mut _final_response: Option<String> = None;
        let mut run_success = false;
        let mut final_quality = 0.0;

        loop {
            self.context.increment_round();
            if !self.context.can_continue() {
                warn!(round = self.context.round, "max rounds reached");
                break;
            }

            let messages = self.context.messages_with_system(&self.system_prompt);

            // Try streaming first. If the provider doesn't return tokens
            // (or errors), we fall back to non-streaming.
            let response = match self
                .provider
                .complete_stream(
                    &self.model,
                    &messages,
                    Some(&tool_schemas),
                    self.max_tokens,
                    self.temperature,
                )
                .await
            {
                Ok(stream) => {
                    use futures::StreamExt;
                    // Consume the stream, collecting tokens into a message
                    // and detecting tool calls.
                    let mut content = String::new();
                    let mut tool_calls_map: std::collections::HashMap<
                        usize,
                        (String, String, String),
                    > = std::collections::HashMap::new();
                    let mut finish_reason: Option<String> = None;
                    futures::pin_mut!(stream);
                    while let Some(ev) = stream.next().await {
                        match ev {
                            Ok(nexus_client::stream::StreamEvent::Content(token)) => {
                                content.push_str(&token);
                                on_event(AgentEvent::Token(token));
                            }
                            Ok(nexus_client::stream::StreamEvent::ToolCallStart { index, id, name }) => {
                                tool_calls_map.insert(index, (id, name, String::new()));
                            }
                            Ok(nexus_client::stream::StreamEvent::ToolCallArguments { index, arguments }) => {
                                if let Some(entry) = tool_calls_map.get_mut(&index) {
                                    entry.2.push_str(&arguments);
                                }
                            }
                            Ok(nexus_client::stream::StreamEvent::Done { finish_reason: r }) => {
                                finish_reason = r.or(finish_reason);
                            }
                            Err(e) => {
                                warn!(error = %e, "stream error");
                            }
                        }
                    }

                    // Reconstruct a ChatResponse from the streamed events.
                    let tool_calls: Vec<ToolCall> = tool_calls_map
                        .into_iter()
                        .map(|(_, (id, name, args))| ToolCall {
                            id,
                            call_type: "function".to_string(),
                            function: nexus_client::message::FunctionCall {
                                name,
                                arguments: args,
                            },
                        })
                        .collect();

                    let response_msg = ResponseMessage {
                        role: Some("assistant".to_string()),
                        content: if content.is_empty() { None } else { Some(content) },
                        tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                    };
                    let choice = Choice {
                        message: response_msg,
                        finish_reason: finish_reason.clone(),
                    };
                    ChatResponse {
                        id: format!("stream-{}", self.context.round),
                        choices: vec![choice],
                    }
                }
                Err(e) => {
                    debug!(error = %e, "stream failed, falling back to non-stream");
                    self.provider
                        .complete(
                            &self.model,
                            &messages,
                            Some(&tool_schemas),
                            self.max_tokens,
                            self.temperature,
                        )
                        .await?
                }
            };

            let choice = response.choices.first().ok_or(CoreError::EmptyResponse)?;
            let resp_msg = &choice.message;

            if let Some(tool_calls) = &resp_msg.tool_calls {
                let assistant_content = resp_msg.content.clone().unwrap_or_default();
                if !assistant_content.is_empty() {
                    chain.add_thought(ThoughtType::Analysis, &assistant_content, 0.9);
                }
                self.context.push_message(Message {
                    role: nexus_client::message::Role::Assistant,
                    content: assistant_content,
                    tool_calls: Some(tool_calls.clone()),
                    tool_call_id: None,
                });

                for tc in tool_calls {
                    let tool_name = tc.function.name.clone();
                    let tool_args = tc.function.arguments.clone();
                    self.tools_used_in_run.push(tool_name.clone());
                    on_event(AgentEvent::ToolStarted {
                        name: tool_name.clone(),
                        args: tool_args.clone(),
                    });

                    let exec_result = self.execute_tool_call(tc).await;
                    let (ok, output) = match exec_result {
                        Ok(out) => (true, out),
                        Err(e) => (false, format!("Error: {e}")),
                    };
                    on_event(AgentEvent::ToolFinished {
                        name: tool_name,
                        ok,
                        output,
                    });
                }
            } else {
                let content = resp_msg.content.clone().unwrap_or_default();
                chain.add_thought(ThoughtType::Reflection, &content, 0.95);

                let verification = self.verify_response(&content, user_input);
                on_event(AgentEvent::Verified {
                    score: verification.score,
                    passed: verification.passed,
                    issues: verification.issues.clone(),
                });
                if !verification.passed {
                    chain.add_thought(
                        ThoughtType::Verification,
                        &format!("Issues: {:?}", verification.issues),
                        0.5,
                    );
                    final_quality = verification.score;
                } else {
                    final_quality = if verification.score > 0.0 { verification.score } else { 0.85 };
                }

                run_success = verification.passed || verification.score >= 0.5;
                _final_response = Some(content);
                break;
            }
        }

        self.thought_chains.push(chain);

        let interaction = Interaction {
            id: format!("{:x}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()),
            task: user_input.to_string(),
            approach: "agent-loop-streaming".to_string(),
            tools_used: self.tools_used_in_run.drain(..).collect(),
            rounds: self.context.round,
            success: run_success,
            quality_score: final_quality,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            context: InteractionContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: TaskComplexity::Moderate,
                similar_past_tasks: Vec::new(),
            },
        };
        self.learner.record_interaction(interaction);

        // Store important learnings in long-term memory
        if final_quality > 0.8 {
            let key = format!("success:{}", self.context.round);
            self.long_term_memory.store(
                &key,
                user_input,
                MemoryCategory::Learning,
                final_quality,
            );
            on_event(AgentEvent::Stored {
                key,
                category: "Learning".to_string(),
            });
        }

        let response = _final_response.ok_or(CoreError::EmptyResponse)?;
        on_event(AgentEvent::Done(response.clone()));

        let final_msg = Message::assistant(&response);
        self.memory.add(final_msg.clone());
        self.context.push_message(final_msg);

        Ok(response)
    }

    fn build_enhanced_prompt(&self, task: &str) -> String {
        let _scaffold_prompt = self.scaffold.create_prompt(task, "");

        format!(
            r#"## COGNITIVE SCAFFOLD PROTOCOL

Before writing ANY code, you MUST:
1. State the ACTUAL problem (not surface request)
2. List constraints and edge cases
3. Explain your approach and WHY
4. Write minimal, clean code
5. Verify your solution works
6. Reflect on improvements

## TASK
{task}

## IMPORTANT
- Use tools to READ code before changing it
- Always verify your work
- Explain your reasoning
- Prefer minimal solutions"#
        )
    }

    fn verify_response(&self, response: &str, task: &str) -> nexus_brain::verify::VerificationResult {
        let code_blocks = self.extract_code_blocks(response);
        if code_blocks.is_empty() {
            return nexus_brain::verify::VerificationResult {
                passed: true,
                score: 1.0,
                checks: vec![],
                issues: vec![],
            };
        }

        let mut worst_result = nexus_brain::verify::VerificationResult {
            passed: true,
            score: 1.0,
            checks: vec![],
            issues: vec![],
        };

        for (lang, code) in &code_blocks {
            if lang == "rust" || lang == "rs" || lang.is_empty() {
                let result = self.verifier.verify(code, task);
                if result.score < worst_result.score {
                    worst_result = result;
                }
            }
        }

        worst_result
    }

    fn extract_code_blocks(&self, text: &str) -> Vec<(String, String)> {
        let mut blocks = Vec::new();
        let mut chars = text.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if c == '`' && text[i..].starts_with("```") {
                let lang_start = i + 3;
                let mut lang_end = lang_start;
                while lang_end < text.len() && text.as_bytes()[lang_end] != b'\n' {
                    lang_end += 1;
                }
                let lang = text[lang_start..lang_end].trim().to_string();

                let code_start = lang_end + 1;
                if let Some(code_end) = text[code_start..].find("```") {
                    let code = text[code_start..code_start + code_end].trim().to_string();
                    blocks.push((lang, code));
                }
            }
        }

        blocks
    }

    async fn execute_tool_call(&mut self, tc: &ToolCall) -> Result<String, CoreError> {
        let args: serde_json::Value =
            serde_json::from_str(&tc.function.arguments).unwrap_or(serde_json::json!({}));

        info!(tool = %tc.function.name, args = %args, "executing tool");

        let result = match self.registry.execute(&tc.function.name, args).await {
            Ok(output) => output,
            Err(e) => format!("ERROR: {e}"),
        };

        debug!(result = %result, "tool result");

        let tool_msg = Message::tool(&result, &tc.id);
        self.memory.add(tool_msg.clone());
        self.context.push_message(tool_msg);

        Ok(result)
    }

    pub fn clear_context(&mut self) {
        self.context = AgentContext::new(self.context.max_rounds);
        self.memory.clear();
    }

    pub fn context(&self) -> &AgentContext {
        &self.context
    }

    pub fn thought_chains(&self) -> &[ThoughtChain] {
        &self.thought_chains
    }

    pub fn learner(&self) -> &AdaptiveLearner {
        &self.learner
    }

    pub fn predictor(&self) -> &SuccessPredictor {
        &self.predictor
    }

    pub fn long_term_memory(&self) -> &LongTermMemory {
        &self.long_term_memory
    }

    pub fn tools_used(&self) -> &[String] {
        &self.tools_used_in_run
    }

    pub fn provider(&self) -> &Arc<LlmProvider> {
        &self.provider
    }

    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.registry
    }
}

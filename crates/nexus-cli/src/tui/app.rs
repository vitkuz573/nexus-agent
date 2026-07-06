//! TUI application: event loop, key dispatch, stream consumer.

#![allow(dead_code)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use nexus_client::message::Message as LlmMessage;
use nexus_client::provider::{LlmProvider, ToolSchema};
use nexus_config::provider::ProviderConfig;
use nexus_core::{Agent, AgentEvent};
use nexus_tools::registry::ToolRegistry;
use ratatui::prelude::*;
use tokio::sync::mpsc;

use crate::tui::command::{SlashCommand, COMMAND_HELP};
use crate::tui::input::InputBuffer;
use crate::tui::panels::{layout, render_conversation, render_files, render_header, render_input, render_status};
use crate::tui::state::{
    Block as ConvBlock, BlockKind, FileEntry, Focus, ToolCallStatus, TuiState,
};
use crate::tui::stream::UiEvent;
use crate::tui::theme::{Theme, ThemeName};

pub struct App {
    provider: Arc<LlmProvider>,
    registry: Arc<ToolRegistry>,
    tools: Vec<ToolSchema>,
    system_prompt: String,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    state: TuiState,
    input: InputBuffer,
    theme: Theme,
    theme_name: ThemeName,
    ui_rx: mpsc::UnboundedReceiver<UiEvent>,
    ui_tx: mpsc::UnboundedSender<UiEvent>,
    help_open: bool,
    should_quit: bool,
    conversation: Vec<LlmMessage>,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: Arc<LlmProvider>,
        registry: Arc<ToolRegistry>,
        tools: Vec<ToolSchema>,
        system_prompt: String,
        prov_config: ProviderConfig,
        max_rounds: usize,
    ) -> Self {
        let (ui_tx, ui_rx) = mpsc::unbounded_channel();
        let mut state = TuiState::new(
            prov_config.model.clone(),
            prov_config.name.clone(),
            max_rounds,
            prov_config.max_tokens,
        );
        state.file_tree = scan_workspace();
        Self {
            provider,
            registry,
            tools,
            system_prompt,
            model: prov_config.model.clone(),
            max_tokens: Some(prov_config.max_tokens),
            temperature: Some(prov_config.temperature),
            state,
            input: InputBuffer::new(),
            theme: Theme::dark(),
            theme_name: ThemeName::Dark,
            ui_rx,
            ui_tx,
            help_open: false,
            should_quit: false,
            conversation: Vec::new(),
        }
    }

    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        let tick = Duration::from_millis(50);
        loop {
            self.state.tick();
            self.update_running_blocks();

            while let Ok(ev) = self.ui_rx.try_recv() {
                self.apply_ui_event(ev);
            }

            terminal.draw(|f| self.render(f))?;

            if event::poll(tick)? {
                self.handle_event(event::read()?)?;
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn update_running_blocks(&mut self) {
        // Update elapsed_ms for streaming/thinking blocks
        if let Some(started) = self.state.run_started_at {
            let elapsed = started.elapsed().as_millis();
            for blk in self.state.blocks.iter_mut() {
                match blk.kind {
                    BlockKind::StreamingAssistant | BlockKind::Thinking => {
                        blk.elapsed_ms = Some(elapsed);
                    }
                    _ => {}
                }
            }
        }
    }

    fn render(&self, f: &mut Frame) {
        let area = f.area();
        let regions = layout(area);

        render_header(f, regions[0], &self.state, &self.theme);
        render_files(f, regions[1], &self.state, &self.theme, self.state.focus == Focus::Files);
        render_conversation(f, regions[2], &self.state, &self.theme, self.state.focus == Focus::Conversation);
        render_input(
            f,
            regions[3],
            &self.input,
            &self.state,
            &self.theme,
            self.state.focus == Focus::Input,
        );
        render_status(f, regions[4], &self.state, &self.theme);

        if self.help_open {
            render_help(f, area, &self.theme);
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        // Esc closes help if open, else clears input
        if key.code == KeyCode::Esc {
            if self.help_open {
                self.help_open = false;
                return;
            }
            if self.state.focus == Focus::Input && !self.input.is_empty() {
                self.input.clear();
                return;
            }
        }

        // PgUp / PgDown always scroll the conversation, regardless of focus
        match key.code {
            KeyCode::PageUp => {
                self.scroll_up(5);
                return;
            }
            KeyCode::PageDown => {
                self.scroll_down(5);
                return;
            }
            _ => {}
        }

        match self.state.focus {
            Focus::Input => self.handle_input_key(key),
            Focus::Conversation => self.handle_conversation_key(key),
            Focus::Files => self.handle_files_key(key),
        }
    }

    fn handle_input_key(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Enter) => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.input.insert_newline();
                } else {
                    let text = self.input.submit();
                    if !text.trim().is_empty() {
                        if text.trim().starts_with('/') {
                            if let Ok(cmd) = text.parse::<SlashCommand>() {
                                self.handle_slash(cmd);
                            }
                        } else {
                            self.send_message(text);
                        }
                    }
                }
            }
            (KeyModifiers::SHIFT, KeyCode::Enter) => {
                self.input.insert_newline();
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => self.input.backspace(),
            (KeyModifiers::NONE, KeyCode::Delete) => self.input.delete_forward(),
            (KeyModifiers::NONE, KeyCode::Left) => self.input.move_left(),
            (KeyModifiers::NONE, KeyCode::Right) => self.input.move_right(),
            (KeyModifiers::NONE, KeyCode::Up) => self.input.history_recall_prev(),
            (KeyModifiers::NONE, KeyCode::Down) => self.input.history_recall_next(),
            (KeyModifiers::NONE, KeyCode::Home) => self.input.move_home(),
            (KeyModifiers::NONE, KeyCode::End) => self.input.move_end(),
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => self.input.clear(),
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => self.input.move_home(),
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => self.input.move_end(),
            (KeyModifiers::NONE, KeyCode::Tab) => self.state.focus = Focus::Conversation,
            (KeyModifiers::SHIFT, KeyCode::BackTab) => self.state.focus = Focus::Files,
            (KeyModifiers::SHIFT, KeyCode::Tab) => self.state.focus = Focus::Files,
            (KeyModifiers::NONE, KeyCode::F(1)) => self.help_open = !self.help_open,
            (_, KeyCode::Char(c)) => self.input.insert_char(c),
            _ => {}
        }
    }

    fn handle_conversation_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => self.state.focus = Focus::Input,
            KeyCode::BackTab => self.state.focus = Focus::Files,
            KeyCode::PageUp => self.scroll_up(5),
            KeyCode::PageDown => self.scroll_down(5),
            KeyCode::Up => self.scroll_up(1),
            KeyCode::Down => self.scroll_down(1),
            KeyCode::Home => self.state.scroll.manual_offset = 0,
            KeyCode::End => {
                self.state.scroll.auto_follow = true;
                self.state.scroll.manual_offset = 0;
            }
            KeyCode::F(1) => self.help_open = !self.help_open,
            _ => {}
        }
    }

    fn handle_files_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => self.state.focus = Focus::Conversation,
            KeyCode::BackTab => self.state.focus = Focus::Input,
            KeyCode::F(1) => self.help_open = !self.help_open,
            _ => {}
        }
    }

    fn scroll_up(&mut self, n: usize) {
        self.state.scroll.auto_follow = false;
        self.state.scroll.manual_offset = self.state.scroll.manual_offset.saturating_add(n);
    }

    fn scroll_down(&mut self, n: usize) {
        if self.state.scroll.manual_offset > n {
            self.state.scroll.manual_offset -= n;
        } else {
            self.state.scroll.manual_offset = 0;
            self.state.scroll.auto_follow = true;
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_up(3),
            MouseEventKind::ScrollDown => self.scroll_down(3),
            _ => {}
        }
    }

    fn handle_slash(&mut self, cmd: SlashCommand) {
        match cmd {
            SlashCommand::Help => self.help_open = true,
            SlashCommand::Unknown(s) => {
                self.help_open = true;
                self.push_system(format!("Unknown command: {s} (press F1 for help)"));
            }
            SlashCommand::Clear => {
                self.state.blocks.clear();
                self.push_system("Conversation cleared.".to_string());
            }
            SlashCommand::Theme => {
                self.theme_name = self.theme_name.next();
                self.theme = Theme::by_name(self.theme_name);
                self.push_system(format!("Theme → {}", self.theme_name.as_str()));
            }
            SlashCommand::Tools => {
                let tools = self.registry.definitions();
                let body = tools
                    .iter()
                    .map(|d| format!("  • {} — {}", d.name, d.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.push_system(format!("Tools:\n{body}"));
            }
            SlashCommand::Model => {
                self.push_system(format!(
                    "Model: {} @ {}",
                    self.state.model, self.state.provider
                ));
            }
            SlashCommand::Providers => {
                let body = match nexus_config::settings::Settings::load() {
                    Ok(s) => s
                        .providers
                        .iter()
                        .map(|p| format!("  • {} → {} @ {}", p.name, p.model, p.base_url))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    Err(e) => format!("(error: {e})"),
                };
                self.push_system(format!("Providers:\n{body}"));
            }
            SlashCommand::History => {
                self.push_system(format!(
                    "Input history (use ↑/↓ to recall previous inputs)"
                ));
            }
            SlashCommand::Quit | SlashCommand::Exit => {
                self.should_quit = true;
            }
            SlashCommand::Save(path) => {
                let body = self
                    .state
                    .blocks
                    .iter()
                    .map(|b| match &b.kind {
                        BlockKind::User => format!("[USER]\n{}\n", b.content),
                        BlockKind::Assistant | BlockKind::StreamingAssistant => {
                            format!("[ASSISTANT]\n{}\n", b.content)
                        }
                        BlockKind::ToolCall { name, args, status } => {
                            format!("[TOOL {name} {:?}]\nargs: {args}\n{}\n", status, b.content)
                        }
                        BlockKind::Thinking => format!("[THINKING]\n{}\n", b.content),
                        BlockKind::System => format!("[SYSTEM]\n{}\n", b.content),
                        BlockKind::Error => format!("[ERROR]\n{}\n", b.content),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                match std::fs::write(&path, body) {
                    Ok(_) => self.push_system(format!("Saved → {path}")),
                    Err(e) => self.push_error(format!("Save failed: {e}")),
                }
            }
            SlashCommand::Load(_path) => {
                self.push_system("Load not yet implemented".to_string());
            }
            SlashCommand::Verify => {
                self.push_system("Verifying last code block…".to_string());
            }
            SlashCommand::Diff => {
                self.push_system("Diff not yet implemented".to_string());
            }
        }
    }

    fn push_user(&mut self, text: String) {
        self.state.blocks.push(ConvBlock {
            kind: BlockKind::User,
            content: text,
            created_at: Instant::now(),
            elapsed_ms: None,
        });
        self.state.scroll.auto_follow = true;
        self.state.scroll.manual_offset = 0;
    }

    fn push_assistant(&mut self, text: String) {
        let elapsed = self
            .state
            .run_started_at
            .map(|s| s.elapsed().as_millis());
        self.state.blocks.push(ConvBlock {
            kind: BlockKind::Assistant,
            content: text,
            created_at: Instant::now(),
            elapsed_ms: elapsed,
        });
        self.state.run_started_at = None;
        self.state.scroll.auto_follow = true;
        self.state.scroll.manual_offset = 0;
    }

    fn push_streaming(&mut self) {
        self.state.blocks.push(ConvBlock {
            kind: BlockKind::StreamingAssistant,
            content: String::new(),
            created_at: Instant::now(),
            elapsed_ms: Some(0),
        });
        self.state.scroll.auto_follow = true;
        self.state.scroll.manual_offset = 0;
    }

    fn push_thinking(&mut self, text: String) {
        self.state.blocks.push(ConvBlock {
            kind: BlockKind::Thinking,
            content: text,
            created_at: Instant::now(),
            elapsed_ms: None,
        });
        self.state.scroll.auto_follow = true;
        self.state.scroll.manual_offset = 0;
    }

    fn push_tool_call(&mut self, name: String, args: String) {
        self.state.blocks.push(ConvBlock {
            kind: BlockKind::ToolCall {
                name,
                args,
                status: ToolCallStatus::Running,
            },
            content: String::new(),
            created_at: Instant::now(),
            elapsed_ms: Some(0),
        });
        self.state.scroll.auto_follow = true;
        self.state.scroll.manual_offset = 0;
    }

    fn push_system(&mut self, text: String) {
        self.state.blocks.push(ConvBlock {
            kind: BlockKind::System,
            content: text,
            created_at: Instant::now(),
            elapsed_ms: None,
        });
        self.state.scroll.auto_follow = true;
        self.state.scroll.manual_offset = 0;
    }

    fn push_error(&mut self, text: String) {
        self.state.blocks.push(ConvBlock {
            kind: BlockKind::Error,
            content: text,
            created_at: Instant::now(),
            elapsed_ms: None,
        });
        self.state.scroll.auto_follow = true;
        self.state.scroll.manual_offset = 0;
    }

    fn send_message(&mut self, text: String) {
        self.push_user(text.clone());
        self.conversation.push(LlmMessage::user(&text));

        let tx = self.ui_tx.clone();
        let provider = Arc::clone(&self.provider);
        let registry = Arc::clone(&self.registry);
        let system_prompt = self.system_prompt.clone();
        let model = self.model.clone();
        let max_tokens = self.max_tokens;
        let temperature = self.temperature;
        let max_rounds = self.state.max_rounds;

        self.state.run_started_at = Some(Instant::now());
        self.push_streaming();

        tokio::spawn(async move {
            let _ = max_rounds;
            tx.send(UiEvent::StatusChanged("thinking".to_string())).ok();

            let mut agent = Agent::new(
                provider,
                registry,
                system_prompt,
                model,
                max_rounds,
                max_tokens,
                temperature,
            );

            let ui_tx = tx.clone();
            let result = agent
                .run_streaming(&text, move |event| {
                    let ev = match event {
                        AgentEvent::Token(t) => UiEvent::StreamToken(t),
                        AgentEvent::ToolStarted { name, args } => {
                            UiEvent::ToolCallStarted { name, args }
                        }
                        AgentEvent::ToolFinished { name, ok, output } => {
                            UiEvent::ToolCallFinished { name, ok, output }
                        }
                        AgentEvent::Thinking(t) => UiEvent::ThinkingStarted(t),
                        AgentEvent::Done(d) => UiEvent::StreamDone(d),
                        AgentEvent::Failed(e) => UiEvent::AppendError(e),
                    };
                    ui_tx.send(ev).ok();
                })
                .await;

            if let Err(e) = result {
                tx.send(UiEvent::AppendError(format!("Error: {e}"))).ok();
            }
            tx.send(UiEvent::StatusChanged("idle".to_string())).ok();
        });

        self.state.status = AgentStatusUsed::Thinking;
    }

    fn apply_ui_event(&mut self, ev: UiEvent) {
        match ev {
            UiEvent::StatusChanged(s) => {
                self.state.status = match s.as_str() {
                    "thinking" => AgentStatusUsed::Thinking,
                    "streaming" => AgentStatusUsed::Streaming,
                    "running" => AgentStatusUsed::Running,
                    _ => AgentStatusUsed::Idle,
                };
            }
            UiEvent::AppendMessage { role: _, content } => {
                self.push_assistant(content);
            }
            UiEvent::AppendError(content) => {
                self.push_error(content);
            }
            UiEvent::StreamDone(content) => {
                // Replace the streaming block with a final assistant block
                if let Some(blk) = self.state.blocks.last_mut() {
                    if matches!(blk.kind, BlockKind::StreamingAssistant) {
                        let elapsed = self
                            .state
                            .run_started_at
                            .map(|s| s.elapsed().as_millis())
                            .unwrap_or(0);
                        blk.kind = BlockKind::Assistant;
                        blk.content = content;
                        blk.elapsed_ms = Some(elapsed);
                    } else {
                        self.push_assistant(content);
                    }
                } else {
                    self.push_assistant(content);
                }
                self.state.run_started_at = None;
            }
            UiEvent::StreamToken(token) => {
                if let Some(blk) = self.state.last_streaming_mut() {
                    blk.content.push_str(&token);
                }
            }
            UiEvent::ThinkingStarted(text) => {
                self.push_thinking(text);
            }
            UiEvent::ToolCallStarted { name, args } => {
                self.push_tool_call(name, args);
            }
            UiEvent::ToolCallFinished { name, ok, output } => {
                if let Some(blk) = self.state.blocks.iter_mut().rev().find(|b| {
                    matches!(&b.kind, BlockKind::ToolCall { name: n, status: ToolCallStatus::Running, .. } if n == &name)
                }) {
                    if let BlockKind::ToolCall { status, .. } = &mut blk.kind {
                        *status = if ok { ToolCallStatus::Ok } else { ToolCallStatus::Failed };
                    }
                    blk.content = output;
                    blk.elapsed_ms = self
                        .state
                        .run_started_at
                        .map(|s| s.elapsed().as_millis());
                }
            }
            UiEvent::SwitchTheme => {
                self.theme_name = self.theme_name.next();
                self.theme = Theme::by_name(self.theme_name);
            }
            UiEvent::ClearConversation => {
                self.state.blocks.clear();
            }
            UiEvent::VerificationDone(_v) => {
                self.push_system("Verification complete".to_string());
            }
        }
    }
}

// Local alias because AgentStatus is in state.rs with #[allow(dead_code)]
use crate::tui::state::AgentStatus as AgentStatusUsed;

fn scan_workspace() -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    collect(&root, 0, &mut entries, 80);
    entries
}

fn collect(path: &std::path::Path, depth: usize, out: &mut Vec<FileEntry>, limit: usize) {
    if out.len() > limit || depth > 3 {
        return;
    }
    let read = match std::fs::read_dir(path) {
        Ok(r) => r,
        Err(_) => return,
    };
    let mut paths: Vec<_> = read.filter_map(|e| e.ok()).collect();
    paths.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));
    for entry in paths {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "Cargo.lock" {
            continue;
        }
        let is_dir = p.is_dir();
        out.push(FileEntry {
            path: name,
            depth,
            is_dir,
            expanded: depth < 2,
        });
        if is_dir {
            collect(&p, depth + 1, out, limit);
        }
    }
}

fn render_help(f: &mut Frame, area: Rect, theme: &Theme) {
    use ratatui::style::Modifier;
    use ratatui::text::Line;
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    let popup = centered_rect(80, 80, area);
    f.render_widget(Clear, popup);

    let mut lines: Vec<Line> = Vec::new();
    for (cmd, desc) in COMMAND_HELP {
        lines.push(Line::from(format!("  {cmd:<24} {desc}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(" Press F1 or Esc to close "));

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let p = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD));
    f.render_widget(p, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

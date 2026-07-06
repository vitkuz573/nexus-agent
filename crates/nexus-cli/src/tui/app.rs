//! TUI application: event loop, key dispatch, stream consumer.

#![allow(dead_code)]

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use nexus_client::message::Message as LlmMessage;
use nexus_client::provider::{LlmProvider, ToolSchema};
use nexus_config::provider::ProviderConfig;
use nexus_core::Agent;
use nexus_tools::registry::ToolRegistry;
use ratatui::prelude::*;
use tokio::sync::mpsc;

use crate::tui::command::{SlashCommand, COMMAND_HELP};
use crate::tui::input::InputBuffer;
use crate::tui::panels::{render_conversation, render_header, render_input, render_sidebar, render_status};
use crate::tui::state::{
    AgentStatus, FileEntry, Focus, Message, MessageRole, TuiState,
};
use crate::tui::stream::{UiEvent, UiSink};
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
    ui_tx: UiSink,
    help_open: bool,
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
            conversation: Vec::new(),
        }
    }

    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        let tick = Duration::from_millis(50);
        loop {
            self.state.tick();

            while let Ok(ev) = self.ui_rx.try_recv() {
                self.apply_ui_event(ev);
            }

            terminal.draw(|f| self.render(f))?;

            if event::poll(tick)? {
                self.handle_event(event::read()?)?;
            }

            if self.state.status == AgentStatus::Idle && self.input.is_empty() && self.should_quit() {
                break;
            }
        }
        Ok(())
    }

    fn should_quit(&self) -> bool {
        false
    }

    fn render(&self, f: &mut Frame) {
        let area = f.area();
        let regions = crate::tui::panels::layout(area, true);

        render_header(f, regions[0], &self.state, &self.theme);
        render_sidebar(f, regions[1], &self.state, &self.theme, self.state.focus == Focus::Sidebar);
        render_conversation(f, regions[2], &self.state, &self.theme, self.state.focus == Focus::Conversation);
        render_input(
            f,
            regions[4],
            &self.input,
            &self.state,
            &self.theme,
            self.state.focus == Focus::Input,
        );
        render_status(f, regions[5], &self.state, &self.theme);

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
            std::process::exit(0);
        }

        match self.state.focus {
            Focus::Input => self.handle_input_key(key),
            Focus::Conversation => self.handle_conversation_key(key),
            Focus::Sidebar => self.handle_sidebar_key(key),
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
                        if let Some(cmd) = text.parse::<SlashCommand>().ok().filter(|_| text.trim().starts_with('/')) {
                            self.handle_slash(cmd);
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
            (KeyModifiers::SHIFT, KeyCode::BackTab) => self.state.focus = Focus::Sidebar,
            (KeyModifiers::SHIFT, KeyCode::Tab) => self.state.focus = Focus::Sidebar,
            (KeyModifiers::NONE, KeyCode::F(1)) => self.help_open = !self.help_open,
            (_, KeyCode::Char(c)) => self.input.insert_char(c),
            _ => {}
        }
    }

    fn handle_conversation_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => self.state.focus = Focus::Sidebar,
            KeyCode::BackTab => self.state.focus = Focus::Input,
            KeyCode::PageUp => {
                self.state.scroll.conversation_offset =
                    self.state.scroll.conversation_offset.saturating_sub(5);
            }
            KeyCode::PageDown => {
                self.state.scroll.conversation_offset = self.state.scroll.conversation_offset.saturating_add(5);
            }
            KeyCode::Up => {
                self.state.scroll.conversation_offset =
                    self.state.scroll.conversation_offset.saturating_sub(1);
            }
            KeyCode::Down => {
                self.state.scroll.conversation_offset =
                    self.state.scroll.conversation_offset.saturating_add(1);
            }
            KeyCode::F(1) => self.help_open = !self.help_open,
            _ => {}
        }
    }

    fn handle_sidebar_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => self.state.focus = Focus::Input,
            KeyCode::BackTab => self.state.focus = Focus::Conversation,
            KeyCode::PageUp => {
                self.state.scroll.sidebar_offset = self.state.scroll.sidebar_offset.saturating_sub(5);
            }
            KeyCode::PageDown => {
                self.state.scroll.sidebar_offset = self.state.scroll.sidebar_offset.saturating_add(5);
            }
            KeyCode::F(1) => self.help_open = !self.help_open,
            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.state.scroll.conversation_offset =
                    self.state.scroll.conversation_offset.saturating_sub(3);
            }
            MouseEventKind::ScrollDown => {
                self.state.scroll.conversation_offset =
                    self.state.scroll.conversation_offset.saturating_add(3);
            }
            _ => {}
        }
    }

    fn handle_slash(&mut self, cmd: SlashCommand) {
        match cmd {
            SlashCommand::Help => {
                self.help_open = true;
            }
            SlashCommand::Unknown(s) => {
                self.help_open = true;
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: format!("Unknown command: {s} (press F1 for help)"),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            SlashCommand::Clear => {
                self.state.messages.clear();
                self.state.tool_events.clear();
                self.ui_tx
                    .send(UiEvent::AppendMessage {
                        role: MessageRole::System,
                        content: "Conversation cleared.".to_string(),
                    })
                    .ok();
            }
            SlashCommand::Theme => {
                self.theme_name = self.theme_name.next();
                self.theme = Theme::by_name(self.theme_name);
                let msg = format!("Theme → {}", self.theme_name.as_str());
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: msg,
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            SlashCommand::Tools => {
                let tools = self.registry.definitions();
                let body = tools
                    .iter()
                    .map(|d| format!("  • {} — {}", d.name, d.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: format!("Tools:\n{body}"),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            SlashCommand::Model => {
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: format!(
                        "Model: {} @ {}",
                        self.state.model, self.state.provider
                    ),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
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
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: format!("Providers:\n{body}"),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            SlashCommand::History => {
                let history = self.input.lines().first().map(|_| "see input history").unwrap_or("");
                let _ = history;
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: format!(
                        "Input history ({} entries): use ↑/↓ to recall",
                        self.input_history_len()
                    ),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            SlashCommand::Quit | SlashCommand::Exit => {
                std::process::exit(0);
            }
            SlashCommand::Save(path) => {
                let body = self
                    .state
                    .messages
                    .iter()
                    .map(|m| format!("[{:?}]\n{}\n", m.role, m.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                match std::fs::write(&path, body) {
                    Ok(_) => self.state.messages.push(Message {
                        role: MessageRole::System,
                        content: format!("Saved → {path}"),
                        timestamp: std::time::Instant::now(),
                        streaming: false,
                    }),
                    Err(e) => self.state.messages.push(Message {
                        role: MessageRole::Error,
                        content: format!("Save failed: {e}"),
                        timestamp: std::time::Instant::now(),
                        streaming: false,
                    }),
                }
            }
            SlashCommand::Load(_path) => {
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: "Load not yet implemented".to_string(),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            SlashCommand::Verify => {
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: "Verifying last code block…".to_string(),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            SlashCommand::Diff => {
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: "Diff not yet implemented".to_string(),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
        }
    }

    fn input_history_len(&self) -> usize {
        0
    }

    fn send_message(&mut self, text: String) {
        self.state.messages.push(Message {
            role: MessageRole::User,
            content: text.clone(),
            timestamp: std::time::Instant::now(),
            streaming: false,
        });
        self.conversation.push(LlmMessage::user(&text));

        let tx = self.ui_tx.clone();
        let provider = Arc::clone(&self.provider);
        let registry = Arc::clone(&self.registry);
        let system_prompt = self.system_prompt.clone();
        let model = self.model.clone();
        let max_tokens = self.max_tokens;
        let temperature = self.temperature;
        let max_rounds = self.state.max_rounds;
        let tool_count = self.tools.len();

        tokio::spawn(async move {
            let _ = tool_count;
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

            match agent.run(&text).await {
                Ok(response) => {
                    tx.send(UiEvent::AppendMessage {
                        role: MessageRole::Assistant,
                        content: response,
                    })
                    .ok();
                }
                Err(e) => {
                    tx.send(UiEvent::AppendMessage {
                        role: MessageRole::Error,
                        content: format!("Error: {e}"),
                    })
                    .ok();
                }
            }
            tx.send(UiEvent::StatusChanged("idle".to_string())).ok();
        });

        self.state.status = AgentStatus::Thinking;
    }

    fn apply_ui_event(&mut self, ev: UiEvent) {
        match ev {
            UiEvent::AppendMessage { role, content } => {
                self.state.messages.push(Message {
                    role,
                    content,
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
            UiEvent::StatusChanged(s) => {
                self.state.status = match s.as_str() {
                    "thinking" => AgentStatus::Thinking,
                    "streaming" => AgentStatus::Streaming,
                    "running" => AgentStatus::Running,
                    _ => AgentStatus::Idle,
                };
            }
            UiEvent::ClearConversation => {
                self.state.messages.clear();
                self.state.tool_events.clear();
            }
            UiEvent::SwitchTheme => {
                self.theme_name = self.theme_name.next();
                self.theme = Theme::by_name(self.theme_name);
            }
            UiEvent::ToolCallStarted(name) => {
                self.state.tool_events.push(crate::tui::state::ToolEvent {
                    name,
                    status: crate::tui::state::ToolStatus::Running,
                    arguments: String::new(),
                    result: None,
                    started_at: std::time::Instant::now(),
                });
            }
            UiEvent::ToolCallFinished { name, ok, output } => {
                if let Some(event) = self.state.tool_events.iter_mut().find(|e| e.name == name) {
                    event.status = if ok {
                        crate::tui::state::ToolStatus::Ok
                    } else {
                        crate::tui::state::ToolStatus::Failed
                    };
                    event.result = Some(output);
                }
            }
            UiEvent::VerificationDone(_v) => {
                self.state.messages.push(Message {
                    role: MessageRole::System,
                    content: "Verification complete".to_string(),
                    timestamp: std::time::Instant::now(),
                    streaming: false,
                });
            }
        }
    }
}

fn scan_workspace() -> Vec<FileEntry> {
    let mut entries = Vec::new();
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    collect(&root, 0, &mut entries, 0);
    entries
}

fn collect(path: &std::path::Path, depth: usize, out: &mut Vec<FileEntry>, limit: usize) {
    if out.len() > limit || depth > 4 {
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
        if name.starts_with('.') || name == "target" || name == "node_modules" {
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

    let mut lines: Vec<Line> = vec![Line::from(format!(
        " ⚡ Nexus Agent — Help "
    ))];
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

#[allow(dead_code)]
fn _force_link() {}

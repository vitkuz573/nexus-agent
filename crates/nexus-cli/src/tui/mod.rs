//! Full-terminal TUI for Nexus Agent.
//!
//! The TUI takes over the entire terminal — no scrolling, no echo.
//! Multiple panels render simultaneously: header, conversation, sidebar,
//! input, status bar. Streaming responses render token-by-token.

pub mod app;
pub mod command;
pub mod input;
pub mod panels;
pub mod state;
pub mod stream;
pub mod theme;

use anyhow::Result;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use nexus_client::provider::LlmProvider;
use nexus_config::provider::ProviderConfig;
use nexus_tools::registry::ToolRegistry;
use ratatui::prelude::*;
use std::io::stdout;
use std::sync::Arc;

use crate::tui::app::App;

/// Launch the full-screen TUI.
///
/// Sets up raw mode and the alternate screen, then runs the event loop.
/// On exit (success or error), restores the terminal to its original state.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    provider: Arc<LlmProvider>,
    registry: Arc<ToolRegistry>,
    tools: Vec<nexus_client::provider::ToolSchema>,
    system_prompt: String,
    prov_config: ProviderConfig,
    max_rounds: usize,
) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new(
        provider,
        registry,
        tools,
        system_prompt,
        prov_config,
        max_rounds,
    );
    let result = app.run(&mut terminal).await;

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
}

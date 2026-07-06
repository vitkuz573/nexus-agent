mod commands;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use nexus_config::settings::Settings;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "nexus",
    about = "⚡ Nexus Agent — full-terminal AI coding agent",
    version,
    long_about = "An open-source, blazing-fast, extensible coding agent for your terminal.\n\n\
                  Run without a subcommand to launch the full-screen TUI."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Set log level (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info")]
    log_level: String,

    /// Provider name to use (default: first configured)
    #[arg(short, long, global = true)]
    provider: Option<String>,

    /// Max tool rounds per request
    #[arg(short = 'r', long, global = true, default_value = "20")]
    max_rounds: usize,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a provider configuration
    Init {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        url: String,

        #[arg(short, long)]
        key: String,

        #[arg(short, long)]
        model: String,

        #[arg(long, default_value = "4096")]
        max_tokens: u32,

        #[arg(long, default_value = "0.7")]
        temperature: f32,
    },

    /// Launch the TUI (default if no subcommand given)
    Chat,

    /// Execute a single prompt and exit (non-interactive)
    Run {
        prompt: Vec<String>,
        #[arg(short = 'R', long)]
        raw: bool,
    },

    /// List configured providers
    Providers,

    /// Remove a provider
    Remove {
        name: String,
    },

    /// Show current config path and contents
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(&cli.log_level, cli.verbose)?;

    match cli.command {
        None => launch_tui(cli.provider.as_deref(), cli.max_rounds).await,
        Some(Commands::Chat) => launch_tui(cli.provider.as_deref(), cli.max_rounds).await,
        Some(Commands::Init { name, url, key, model, max_tokens, temperature }) => {
            commands::cmd_init(&name, &url, &key, &model, max_tokens, temperature)
        }
        Some(Commands::Run { prompt, raw }) => {
            commands::cmd_run(&prompt.join(" "), cli.provider.as_deref(), raw).await
        }
        Some(Commands::Providers) => commands::cmd_providers(),
        Some(Commands::Remove { name }) => commands::cmd_remove(&name),
        Some(Commands::Config) => commands::cmd_config(),
    }
}

fn init_logging(level: &str, verbose: bool) -> Result<()> {
    let filter = if verbose {
        format!("nexus_agent=debug,{}", level)
    } else {
        format!("nexus_agent={}", level)
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&filter)))
        .with_target(false)
        .with_ansi(true)
        .init();

    Ok(())
}

async fn launch_tui(provider: Option<&str>, max_rounds: usize) -> Result<()> {
    let settings = Settings::load()?;
    let prov_name = provider.unwrap_or("default");
    let prov_config = settings.get_provider(prov_name)?.clone();

    let prov = std::sync::Arc::new(nexus_client::provider::LlmProvider::new(
        &prov_config.base_url,
        &prov_config.api_key,
    ));

    let mut registry = nexus_tools::registry::ToolRegistry::new();
    commands::register_tools(&mut registry);
    let registry = std::sync::Arc::new(registry);

    let tools: Vec<nexus_client::provider::ToolSchema> = registry
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

    tui::run(
        prov,
        registry,
        tools,
        settings.system_prompt.clone(),
        prov_config.clone(),
        max_rounds,
    )
    .await
}

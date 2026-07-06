//! Non-TUI subcommands: init, run, providers, remove, config.

use anyhow::Result;
use console::style;
use nexus_config::settings::Settings;
use nexus_config::provider::ProviderConfig;

pub fn cmd_init(name: &str, url: &str, key: &str, model: &str, max_tokens: u32, temperature: f32) -> Result<()> {
    let mut settings = Settings::load().unwrap_or_default();
    settings.add_provider(ProviderConfig {
        name: name.to_string(),
        base_url: url.to_string(),
        api_key: key.to_string(),
        model: model.to_string(),
        max_tokens,
        temperature,
    });
    settings.save()?;

    println!(
        "{} Provider '{}' added → model: {}",
        style("✓").green().bold(),
        style(name).cyan().bold(),
        style(model).yellow()
    );
    Ok(())
}

pub async fn cmd_run(prompt: &str, provider: Option<&str>, raw: bool) -> Result<()> {
    let settings = Settings::load()?;
    let prov_name = provider.unwrap_or("default");
    let prov_config = settings.get_provider(prov_name)?;
    let prov = std::sync::Arc::new(nexus_client::provider::LlmProvider::new(
        &prov_config.base_url,
        &prov_config.api_key,
    ));

    let mut registry = nexus_tools::registry::ToolRegistry::new();
    register_tools(&mut registry);
    let registry = std::sync::Arc::new(registry);

    let mut agent = nexus_core::Agent::new(
        prov,
        registry,
        settings.system_prompt.clone(),
        prov_config.model.clone(),
        20,
        Some(prov_config.max_tokens),
        Some(prov_config.temperature),
    );

    let result = agent.run(prompt).await?;
    if raw {
        println!("{result}");
    } else {
        println!("\n{} {}\n", style("▸").blue().bold(), result);
    }
    Ok(())
}

pub fn cmd_providers() -> Result<()> {
    let settings = Settings::load()?;
    if settings.providers.is_empty() {
        println!("{}", style("No providers configured.").dim());
        println!("Run: nexus init --name <name> --url <url> --key <key> --model <model>");
        return Ok(());
    }

    println!("\n{}", style("Configured Providers:").bold().underlined());
    for (i, p) in settings.providers.iter().enumerate() {
        println!(
            "  {}. {} → {} [{}]",
            style(i + 1).cyan(),
            style(&p.name).green().bold(),
            style(&p.model).yellow(),
            &p.base_url,
        );
    }
    println!();
    Ok(())
}

pub fn cmd_remove(name: &str) -> Result<()> {
    let mut settings = Settings::load()?;
    let before = settings.providers.len();
    settings.providers.retain(|p| p.name != name);
    if settings.providers.len() == before {
        println!("{} Provider '{}' not found.", style("✗").red().bold(), name);
    } else {
        settings.save()?;
        println!("{} Provider '{}' removed.", style("✓").green().bold(), name);
    }
    Ok(())
}

pub fn cmd_config() -> Result<()> {
    let path = Settings::config_path()?;
    println!("Config: {}", style(path.display()).cyan());
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        println!("\n{content}");
    } else {
        println!("(not yet created)");
    }
    Ok(())
}

pub fn register_tools(registry: &mut nexus_tools::ToolRegistry) {
    use nexus_tools::tools::*;
    registry.register(Box::new(BashTool));
    registry.register(Box::new(ReadFileTool));
    registry.register(Box::new(WriteFileTool));
    registry.register(Box::new(ListDirTool));
    registry.register(Box::new(GrepTool));
    registry.register(Box::new(nexus_cognitive::ThinkTool));
    registry.register(Box::new(nexus_cognitive::VerifyCodeTool));
    registry.register(Box::new(nexus_cognitive::AnalyzeRisksTool));
    registry.register(Box::new(nexus_cognitive::SearchCodeTool));
    registry.register(Box::new(nexus_cognitive::RecallMemoryTool));
}

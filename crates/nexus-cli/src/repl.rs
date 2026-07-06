use anyhow::Result;
use console::style;
use nexus_config::provider::ProviderConfig;
use std::io::{self, Write};

pub async fn run(
    agent: &mut nexus_core::Agent<'_>,
    provider: &ProviderConfig,
) -> Result<()> {
    print_banner(provider);

    let mut line_num: u64 = 0;

    loop {
        line_num += 1;
        print_prompt(line_num);
        flush_stdout()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            println!("\n{}", style("Goodbye!").dim());
            break;
        }
        let input = input.trim();

        match input {
            "" => continue,
            "/quit" | "/exit" | "/q" => {
                println!("{}", style("Goodbye!").dim());
                break;
            }
            "/clear" | "/cls" => {
                agent.clear_context();
                clear_screen();
                print_banner(provider);
                continue;
            }
            "/help" | "/h" | "/?" => {
                print_help();
                continue;
            }
            "/providers" => {
                println!("Connected: {} ({})", style(&provider.name).cyan(), style(&provider.model).yellow());
                continue;
            }
            "/history" => {
                let ctx = agent.context();
                println!("Messages: {} | Rounds: {}/{}",
                    style(ctx.messages.len()).cyan(),
                    style(ctx.round).yellow(),
                    style(ctx.max_rounds).dim()
                );
                continue;
            }
            _ if input.starts_with('/') => {
                println!("{} Unknown command: {}", style("✗").red().bold(), input);
                println!("Type {} for available commands.", style("/help").dim());
                continue;
            }
            _ => {}
        }

        let mut spinner = start_spinner("thinking");
        let result = agent.run(input).await;
        stop_spinner(&mut spinner);

        match result {
            Ok(response) => {
                println!();
                print_response(&response);
                println!();
            }
            Err(e) => {
                println!("{} {}", style("Error:").red().bold(), e);
            }
        }
    }

    Ok(())
}

fn print_banner(provider: &ProviderConfig) {
    println!();
    println!("  {} {}", style("⚡").yellow().bold(), style("Nexus Agent").bold().dim());
    println!("  {} {} • {} max_tokens: {} • temp: {}",
        style("→").dim(),
        style(&provider.name).cyan(),
        style(&provider.model).yellow(),
        style(provider.max_tokens).dim(),
        style(format!("{:.1}", provider.temperature)).dim(),
    );
    println!("  {}", style("─────────────────────────────────────────").dim());
    println!("  {} for commands, {} to exit",
        style("/help").green(),
        style("/quit").red()
    );
    println!();
}

fn print_prompt(line_num: u64) {
    print!("  {} {} ",
        style(format!("{:>3}", line_num)).dim(),
        style("›").green().bold(),
    );
}

fn print_response(text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    for line in &lines {
        println!("  {} {}", style("▸").blue().bold(), line);
    }
}

fn print_help() {
    println!();
    println!("  {} Commands:", style("📖").cyan().bold());
    println!("    {} Show this help", style("/help").green());
    println!("    {} Clear conversation history", style("/clear").green());
    println!("    {} Show connected provider", style("/providers").green());
    println!("    {} Show message count & rounds", style("/history").green());
    println!("    {} Exit the agent", style("/quit").red());
    println!();
    println!("  {} Tips:", style("💡").yellow().bold());
    println!("    • Multi-line input: paste directly");
    println!("    • Tool calls are automatic");
    println!("    • Use Ctrl+C to cancel, Ctrl+D to exit");
    println!();
}

fn flush_stdout() -> Result<()> {
    io::stdout().flush()?;
    Ok(())
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    flush_stdout().ok();
}

struct Spinner {
    handle: Option<std::thread::JoinHandle<()>>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

fn start_spinner(text: &str) -> Spinner {
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    let msg = text.to_string();

    let handle = std::thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        while r.load(std::sync::atomic::Ordering::Relaxed) {
            print!("\r  {} {} ", style(frames[i % frames.len()]).cyan(), style(&msg).dim());
            flush_stdout().ok();
            i += 1;
            std::thread::sleep(std::time::Duration::from_millis(80));
        }
        print!("\r{}\r", " ".repeat(40));
        flush_stdout().ok();
    });

    Spinner {
        handle: Some(handle),
        running,
    }
}

fn stop_spinner(spinner: &mut Spinner) {
    spinner.running.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Some(handle) = spinner.handle.take() {
        handle.join().ok();
    }
}

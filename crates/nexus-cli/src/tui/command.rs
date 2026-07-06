//! Slash-command catalog for the TUI.
//!
//! `/help`, `/clear`, `/tools`, `/theme`, etc. Commands are dispatched
//! from the App's input handler.

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Help,
    Clear,
    Tools,
    Theme,
    Model,
    Providers,
    Quit,
    Exit,
    Save(String),
    Load(String),
    Verify,
    Diff,
    History,
    Unknown(String),
}

impl SlashCommand {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            SlashCommand::Help => "help",
            SlashCommand::Clear => "clear",
            SlashCommand::Tools => "tools",
            SlashCommand::Theme => "theme",
            SlashCommand::Model => "model",
            SlashCommand::Providers => "providers",
            SlashCommand::Quit => "quit",
            SlashCommand::Exit => "exit",
            SlashCommand::Save(_) => "save",
            SlashCommand::Load(_) => "load",
            SlashCommand::Verify => "verify",
            SlashCommand::Diff => "diff",
            SlashCommand::History => "history",
            SlashCommand::Unknown(_) => "unknown",
        }
    }
}

impl FromStr for SlashCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if !trimmed.starts_with('/') {
            return Err(());
        }
        let body = &trimmed[1..];
        let (cmd, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        let rest = rest.trim();

        match cmd {
            "help" | "?" => Ok(SlashCommand::Help),
            "clear" | "cls" | "c" => Ok(SlashCommand::Clear),
            "tools" => Ok(SlashCommand::Tools),
            "theme" => Ok(SlashCommand::Theme),
            "model" | "m" => Ok(SlashCommand::Model),
            "providers" => Ok(SlashCommand::Providers),
            "quit" | "q" => Ok(SlashCommand::Quit),
            "exit" => Ok(SlashCommand::Exit),
            "save" => Ok(SlashCommand::Save(rest.to_string())),
            "load" => Ok(SlashCommand::Load(rest.to_string())),
            "verify" => Ok(SlashCommand::Verify),
            "diff" => Ok(SlashCommand::Diff),
            "history" => Ok(SlashCommand::History),
            _ => Ok(SlashCommand::Unknown(trimmed.to_string())),
        }
    }
}

pub const COMMAND_HELP: &[(&str, &str)] = &[
    ("/help, /?", "Show this help"),
    ("/clear, /c", "Clear conversation"),
    ("/tools", "List available tools"),
    ("/theme", "Cycle through themes"),
    ("/model, /m", "Show current model"),
    ("/providers", "List configured providers"),
    ("/verify", "Re-run verification on last code"),
    ("/diff", "Show last diff"),
    ("/history", "Show input history"),
    ("/save <file>", "Save conversation to file"),
    ("/load <file>", "Load conversation from file"),
    ("/quit, /q, /exit", "Exit Nexus"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        assert_eq!("/help".parse::<SlashCommand>().unwrap(), SlashCommand::Help);
        assert_eq!("/?".parse::<SlashCommand>().unwrap(), SlashCommand::Help);
    }

    #[test]
    fn test_parse_clear() {
        assert_eq!("/clear".parse::<SlashCommand>().unwrap(), SlashCommand::Clear);
        assert_eq!("/c".parse::<SlashCommand>().unwrap(), SlashCommand::Clear);
    }

    #[test]
    fn test_parse_with_args() {
        let cmd = "/save foo.txt".parse::<SlashCommand>().unwrap();
        assert_eq!(cmd, SlashCommand::Save("foo.txt".to_string()));
    }

    #[test]
    fn test_parse_unknown() {
        let cmd = "/nonsense".parse::<SlashCommand>().unwrap();
        assert!(matches!(cmd, SlashCommand::Unknown(_)));
    }
}

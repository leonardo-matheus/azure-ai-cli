mod config;
mod client;
mod tools;
mod ui;
mod chat;
mod input;
mod i18n;

use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-v" => {
                println!("aicli v1.0.0");
                return Ok(());
            }
            "--config" | "-c" => {
                config::setup_config_interactive().await?;
                return Ok(());
            }
            _ => {}
        }
    }

    let config = match config::load_config() {
        Ok(c) => c,
        Err(_) => {
            println!("\x1b[33m⚠ No configuration found. Running setup...\x1b[0m\n");
            config::setup_config_interactive().await?
        }
    };

    chat::run(config).await
}

fn print_help() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════╗
║                    AICLI - Azure AI CLI                       ║
║                  By Leonardo M. Silva                         ║
╚═══════════════════════════════════════════════════════════════╝

Usage: aicli [OPTIONS]

Options:
  -h, --help      Show this help message
  -v, --version   Show version
  -c, --config    Configure API settings

Commands (inside chat):
  /help           Show available commands
  /exit, /quit    Exit the CLI
  /clear          Clear conversation history
  /model          List and switch models
  /model <name>   Switch to specific model
  /add-model      Add a new model
  /config         Show current configuration
  /history        Show conversation history

Features:
  • TAB completion for commands (/)
  • TAB completion for file paths (@)
  • Multiple model support
  • Automatic tool execution

Environment Variables:
  AZURE_API_KEY       API key for Azure AI Foundry
  AZURE_ENDPOINT      Azure AI endpoint URL
  AZURE_DEPLOYMENT    Model deployment name

Config file location: ~/.aicli/config.toml
"#);
}

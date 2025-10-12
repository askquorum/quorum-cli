// ============================================================================
// File: src/main.rs
// Entry point and CLI handling
// ============================================================================

mod config;
mod models;
mod orchestrator;
mod llm_client;
mod search_client;
mod markdown;

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::orchestrator::DebateOrchestrator;

/// Command-line arguments for the debate orchestrator
#[derive(Parser, Debug)]
#[command(name = "llm-debate")]
#[command(about = "Orchestrate debates between multiple LLMs", long_about = None)]
struct Args {
    /// Path to the JSON configuration file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    /// Path where the markdown output file will be saved
    #[arg(short, long, default_value = "debate_output.md")]
    output: PathBuf,

    /// Enable verbose output (shows search queries and debug info)
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Load configuration from JSON file
    let config_content = fs::read_to_string(&args.config)
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;
    let config: Config = serde_json::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;

    // Create and run the debate orchestrator
    let mut orchestrator = DebateOrchestrator::new(config, args.verbose);
    orchestrator.run_debate().await?;

    // Export results to markdown
    orchestrator.export_to_markdown(&args.output)?;
    println!("\n{} Debate exported to: {}",
        "✓".green().bold(),
        args.output.display().to_string().bright_cyan());

    // Display final statistics
    println!("\n{} Total tokens used: {}",
        "ℹ".blue().bold(),
        orchestrator.get_total_tokens().to_string().bright_yellow());

    Ok(())
}

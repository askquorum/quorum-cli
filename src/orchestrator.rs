// ============================================================================
// File: src/orchestrator.rs
// Main debate orchestration logic
// ============================================================================

use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;
use tiktoken_rs::p50k_base;

use crate::config::{Config, DebateRules, Participant};
use crate::models::Message;
use crate::llm_client::LLMClient;
use crate::markdown::MarkdownExporter;

pub struct DebateOrchestrator {
    config: Config,
    llm_client: LLMClient,
    conversation_history: Vec<Message>,
    total_tokens_used: usize,
    current_round: usize,
    #[allow(dead_code)]
    verbose: bool,
}

impl DebateOrchestrator {
    pub fn new(config: Config, verbose: bool) -> Self {
        // Validate configuration
        config.validate().expect("Invalid configuration");

        // Initialize LLM client
        let llm_client = LLMClient::new(
            config.openrouter_api_key.clone(),
            config.searxng_url.clone(),
            config.searxng_api_key.clone(),
            config.debate_rules.enable_search,
            config.debate_rules.search_limit_per_turn,
            verbose,
        );

        // Initialize conversation with context
        let mut conversation_history = Vec::new();
        let search_instructions = if config.debate_rules.enable_search {
            format!(
                "\n\nWEB SEARCH CAPABILITY:\nYou can request web searches by including 'SEARCH: <query>' on its own line in your response.\n\
                You can request up to {} search(es) per turn.\n\
                The search results will be provided, and you can then give your final response.",
                config.debate_rules.search_limit_per_turn
            )
        } else {
            String::new()
        };

        conversation_history.push(Message {
            role: "system".to_string(),
            content: format!(
                "DEBATE RULES AND CONTEXT:\n{}\n\nTOPIC: {}\n\nDEBATE CONTEXT:\n{}{}",
                Self::format_rules(&config.debate_rules),
                config.topic,
                config.context,
                search_instructions
            ),
        });

        Self {
            config,
            llm_client,
            conversation_history,
            total_tokens_used: 0,
            current_round: 0,
            verbose,
        }
    }

    pub async fn run_debate(&mut self) -> Result<()> {
        self.print_header();

        let total_turns = self.config.debate_rules.rounds * self.config.participants.len();
        let progress_bar = self.create_progress_bar(total_turns);

        for round in 1..=self.config.debate_rules.rounds {
            self.current_round = round;
            self.print_round_header(round);

            for participant in self.config.participants.clone() {
                if !self.check_token_limit()? {
                    println!("\n{} Token limit approaching, ending debate early", "⚠".yellow());
                    break;
                }

                progress_bar.set_message(format!("{} is thinking...", participant.name));

                let response = self.get_participant_response(&participant).await?;
                self.display_response(&participant, &response);

                progress_bar.inc(1);
            }
        }

        progress_bar.finish_with_message("Debate completed!");
        Ok(())
    }

    pub fn export_to_markdown(&self, path: &PathBuf) -> Result<()> {
        let exporter = MarkdownExporter::new(
            &self.config,
            &self.conversation_history,
            self.total_tokens_used,
        );
        exporter.export(path)
    }

    pub fn get_total_tokens(&self) -> usize {
        self.total_tokens_used
    }

    async fn get_participant_response(&mut self, participant: &Participant) -> Result<(String, usize, f32)> {
        // Prepare messages for this participant
        let mut messages = self.conversation_history.clone();
        if let Some(system_prompt) = &participant.system_prompt {
            messages.insert(1, Message {
                role: "system".to_string(),
                content: format!("You are {}. {}", participant.name, system_prompt),
            });
        }

        // Get LLM response
        let start = Instant::now();
        let (response, tokens) = self.llm_client
            .get_response(participant, messages, self.config.debate_rules.max_tokens_per_turn)
            .await?;
        let duration = start.elapsed().as_secs_f32();

        // Add to conversation history
        self.conversation_history.push(Message {
            role: "assistant".to_string(),
            content: format!("[{}]: {}", participant.name, response),
        });

        self.total_tokens_used += tokens;

        Ok((response, tokens, duration))
    }

    fn check_token_limit(&self) -> Result<bool> {
        let history_tokens = self.count_tokens(
            &self.conversation_history.iter()
                .map(|m| m.content.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        )?;

        Ok(history_tokens + self.config.debate_rules.max_tokens_per_turn
            <= self.config.debate_rules.max_total_tokens)
    }

    fn count_tokens(&self, text: &str) -> Result<usize> {
        let bpe = p50k_base()?;
        let tokens = bpe.encode_with_special_tokens(text);
        Ok(tokens.len())
    }

    fn format_rules(rules: &DebateRules) -> String {
        format!(
            "- Maximum tokens per turn: {}\n\
             - Total rounds: {}\n\
             - Web search enabled: {}\n\
             - Maximum searches per turn: {}",
            rules.max_tokens_per_turn,
            rules.rounds,
            rules.enable_search,
            rules.search_limit_per_turn
        )
    }

    fn print_header(&self) {
        println!("{}", "\n═══════════════════════════════════════".bright_blue());
        println!("{}", "       LLM DEBATE ORCHESTRATOR".bright_white().bold());
        println!("{}", "═══════════════════════════════════════".bright_blue());
        println!("\n{}: {}", "Topic".green().bold(), self.config.topic);
        println!("{}: {}",
            "Participants".green().bold(),
            self.config.participants.iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", "));
        println!("{}: {}\n", "Rounds".green().bold(), self.config.debate_rules.rounds);
    }

    fn print_round_header(&self, round: usize) {
        println!("\n{} Round {}/{} {}",
            "►".yellow().bold(),
            round,
            self.config.debate_rules.rounds,
            "◄".yellow().bold());
        println!("{}", "─".repeat(40).bright_black());
    }

    fn display_response(&self, participant: &Participant, response: &(String, usize, f32)) {
        let (content, tokens, duration) = response;

        println!("\n{} {} {}",
            "●".bright_cyan(),
            participant.name.bright_white().bold(),
            format!("({:.1}s, {} tokens)", duration, tokens).bright_black());
        println!("{}", "─".repeat(40).bright_black());

        for line in content.lines() {
            if line.starts_with("[Search:") {
                println!("{}", line.bright_black().italic());
            } else {
                println!("{}", line);
            }
        }
    }

    fn create_progress_bar(&self, total: usize) -> ProgressBar {
        let progress_bar = ProgressBar::new(total as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▓▒░ ")
        );
        progress_bar
    }
}

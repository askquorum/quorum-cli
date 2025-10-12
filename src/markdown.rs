// ============================================================================
// File: src/markdown.rs
// Markdown export functionality
// ============================================================================

use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::models::Message;

pub struct MarkdownExporter<'a> {
    config: &'a Config,
    conversation_history: &'a Vec<Message>,
    total_tokens: usize,
}

impl<'a> MarkdownExporter<'a> {
    pub fn new(
        config: &'a Config,
        conversation_history: &'a Vec<Message>,
        total_tokens: usize,
    ) -> Self {
        Self {
            config,
            conversation_history,
            total_tokens,
        }
    }

    pub fn export(&self, path: &PathBuf) -> Result<()> {
        let mut content = String::new();

        self.write_header(&mut content);
        self.write_participants(&mut content);
        self.write_context(&mut content);
        self.write_debate_transcript(&mut content);

        fs::write(path, content)?;
        Ok(())
    }

    fn write_header(&self, content: &mut String) {
        content.push_str(&format!("# LLM Debate: {}\n\n", self.config.topic));
        content.push_str(&format!("**Date**: {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        content.push_str(&format!("**Total Tokens Used**: {}\n\n", self.total_tokens));
    }

    fn write_participants(&self, content: &mut String) {
        content.push_str("## Participants\n\n");
        for participant in &self.config.participants {
            content.push_str(&format!("- **{}**: {} (temp: {})\n",
                participant.name,
                participant.model,
                participant.temperature));
        }
        content.push_str("\n");
    }

    fn write_context(&self, content: &mut String) {
        content.push_str("## Context\n\n");
        content.push_str(&self.config.context);
        content.push_str("\n\n");
    }

    fn write_debate_transcript(&self, content: &mut String) {
        content.push_str("## Debate\n\n");

        let mut current_round = 0;
        let mut turn_in_round = 0;
        let participants_count = self.config.participants.len();

        for (i, message) in self.conversation_history.iter().enumerate() {
            if i == 0 {
                continue; // Skip system message
            }

            if message.role == "assistant" {
                // New round header
                if turn_in_round == 0 {
                    current_round += 1;
                    content.push_str(&format!("### Round {}\n\n", current_round));
                }

                // Extract and format participant response
                if let Some(pos) = message.content.find("]: ") {
                    let name = &message.content[1..pos];
                    let text = &message.content[pos + 3..];

                    content.push_str(&format!("#### {}\n\n", name));

                    // Format search results and main content
                    for line in text.lines() {
                        if line.starts_with("[Search:") {
                            content.push_str(&format!("> {}\n", line));
                        } else {
                            content.push_str(&format!("{}\n", line));
                        }
                    }
                    content.push_str("\n---\n\n");
                }

                turn_in_round = (turn_in_round + 1) % participants_count;
            }
        }
    }
}

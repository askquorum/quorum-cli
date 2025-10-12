// ============================================================================
// File: src/config.rs
// Configuration structures and validation
// ============================================================================

use serde::{Deserialize, Serialize};

/// Main configuration structure loaded from config.json
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// OpenRouter API key for accessing LLMs
    pub openrouter_api_key: String,

    /// Base URL for your SearXNG instance (e.g., "http://localhost:8888")
    pub searxng_url: String,

    /// Optional API key for SearXNG (if your instance requires authentication)
    pub searxng_api_key: Option<String>,

    /// Rules governing the debate format and limits
    pub debate_rules: DebateRules,

    /// List of LLM participants in the debate
    pub participants: Vec<Participant>,

    /// The main topic or question being debated
    pub topic: String,

    /// Additional context and framing for the debate
    pub context: String,
}

/// Debate rules and constraints
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DebateRules {
    /// Maximum total tokens for the entire debate
    pub max_total_tokens: usize,

    /// Maximum tokens per individual LLM response
    pub max_tokens_per_turn: usize,

    /// Number of complete rounds in the debate
    pub rounds: usize,

    /// Whether LLMs can use web search
    pub enable_search: bool,

    /// Maximum number of searches per LLM turn
    pub search_limit_per_turn: usize,
}

/// Individual debate participant configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Participant {
    /// Display name for this participant
    pub name: String,

    /// OpenRouter model identifier (e.g., "anthropic/claude-3-opus-20240229")
    pub model: String,

    /// Optional system prompt to define the participant's role/perspective
    pub system_prompt: Option<String>,

    /// Temperature setting for response generation (0.0-1.0)
    pub temperature: f32,
}

impl Config {
    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.openrouter_api_key.is_empty() {
            return Err(anyhow::anyhow!("OpenRouter API key is required"));
        }

        if self.participants.is_empty() {
            return Err(anyhow::anyhow!("At least one participant is required"));
        }

        if self.debate_rules.rounds == 0 {
            return Err(anyhow::anyhow!("At least one round is required"));
        }

        for participant in &self.participants {
            if participant.temperature < 0.0 || participant.temperature > 2.0 {
                return Err(anyhow::anyhow!(
                    "Temperature for {} must be between 0.0 and 2.0",
                    participant.name
                ));
            }
        }

        Ok(())
    }
}

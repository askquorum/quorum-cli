// ============================================================================
// File: src/llm_client.rs
// OpenRouter API client for LLM interactions
// ============================================================================

use anyhow::{Result, anyhow};
use reqwest::Client;
use colored::*;

use crate::config::Participant;
use crate::models::{Message, OpenRouterRequest, OpenRouterResponse};
use crate::search_client::SearchClient;

pub struct LLMClient {
    client: Client,
    api_key: String,
    search_client: SearchClient,
    enable_search: bool,
    search_limit: usize,
    verbose: bool,
}

impl LLMClient {
    pub fn new(
        api_key: String,
        searxng_url: String,
        searxng_api_key: Option<String>,
        enable_search: bool,
        search_limit: usize,
        verbose: bool,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key,
            search_client: SearchClient::new(searxng_url, searxng_api_key),
            enable_search,
            search_limit,
            verbose,
        }
    }

    pub async fn get_response(
        &self,
        participant: &Participant,
        messages: Vec<Message>,
        max_tokens: usize,
    ) -> Result<(String, usize)> {
        // Initial API request
        let (content, mut total_tokens) = self.make_api_call(participant, &messages, max_tokens).await?;

        // Check if search is enabled and response contains search requests
        if self.enable_search {
            let search_requests = self.extract_search_queries(&content);

            if !search_requests.is_empty() {
                // Execute searches
                let search_results = self.execute_searches(&search_requests).await?;

                // Make follow-up call with search results
                let mut follow_up_messages = messages.clone();
                follow_up_messages.push(Message {
                    role: "assistant".to_string(),
                    content: content.clone(),
                });
                follow_up_messages.push(Message {
                    role: "user".to_string(),
                    content: format!("Here are the search results:\n\n{}\n\nPlease provide your response based on this information.", search_results),
                });

                let (final_content, follow_up_tokens) = self.make_api_call(participant, &follow_up_messages, max_tokens).await?;
                total_tokens += follow_up_tokens;

                return Ok((final_content, total_tokens));
            }
        }

        Ok((content, total_tokens))
    }

    async fn make_api_call(
        &self,
        participant: &Participant,
        messages: &[Message],
        max_tokens: usize,
    ) -> Result<(String, usize)> {
        let request = OpenRouterRequest {
            model: participant.model.clone(),
            messages: messages.to_vec(),
            temperature: participant.temperature,
            max_tokens,
        };

        if self.verbose {
            println!("  {} Calling model: {} (temp: {}, max_tokens: {})",
                "→".yellow(),
                participant.model.cyan(),
                participant.temperature,
                max_tokens
            );
        }

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!(
                "API error for model '{}': HTTP {}\nResponse: {}",
                participant.model,
                status,
                error_text
            ));
        }

        // Try to parse the response, with better error handling
        let response_text = response.text().await?;
        let response_data: OpenRouterResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!(
                "Failed to parse response from model '{}': {}\nRaw response: {}",
                participant.model,
                e,
                response_text
            ))?;

        let choice = response_data.choices.first()
            .ok_or_else(|| anyhow!(
                "Model '{}' returned no choices. Response may be empty.",
                participant.model
            ))?;

        let content = choice.message.content.clone()
            .unwrap_or_default();

        if content.is_empty() {
            return Err(anyhow!(
                "Model '{}' returned empty content. This model may not be compatible with the current API format.",
                participant.model
            ));
        }

        let tokens_used = response_data.usage
            .map(|u| u.total_tokens)
            .unwrap_or_else(|| Self::estimate_tokens(&content));

        Ok((content, tokens_used))
    }

    fn extract_search_queries(&self, content: &str) -> Vec<String> {
        let mut queries = Vec::new();

        for line in content.lines() {
            if let Some(query) = line.strip_prefix("SEARCH:") {
                let trimmed = query.trim();
                if !trimmed.is_empty() {
                    queries.push(trimmed.to_string());
                    if queries.len() >= self.search_limit {
                        break;
                    }
                }
            }
        }

        queries
    }

    async fn execute_searches(&self, queries: &[String]) -> Result<String> {
        let mut all_results = String::new();

        for query in queries {
            if self.verbose {
                println!("  {} Searching: {}", "→".yellow(), query.cyan());
            }

            let results = self.search_client.search(query).await?;
            all_results.push_str(&format!("Search query: {}\n{}\n", query, results));
        }

        Ok(all_results)
    }

    fn estimate_tokens(text: &str) -> usize {
        // Rough estimation: ~4 characters per token
        text.len() / 4
    }
}

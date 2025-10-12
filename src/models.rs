// ============================================================================
// File: src/models.rs
// API request and response models
// ============================================================================

use serde::{Deserialize, Serialize};

/// Message structure for conversation history
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,  // "system", "user", or "assistant"
    pub content: String,
}

/// Request structure for OpenRouter API
#[derive(Debug, Serialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: usize,
}

/// Response structure from OpenRouter API
#[derive(Debug, Deserialize)]
pub struct OpenRouterResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// Individual response choice
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
}

/// Message in API response
#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub content: Option<String>,
}

/// Token usage information
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Response from SearXNG search API
#[derive(Debug, Deserialize)]
pub struct SearXNGResponse {
    pub results: Vec<SearchResult>,
}

/// Individual search result
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

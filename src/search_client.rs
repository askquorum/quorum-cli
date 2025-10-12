// ============================================================================
// File: src/search_client.rs
// SearXNG search client
// ============================================================================

use anyhow::Result;
use reqwest::Client;

use crate::models::SearXNGResponse;

pub struct SearchClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl SearchClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }

    pub async fn search(&self, query: &str) -> Result<String> {
        let url = format!("{}/search", self.base_url);
        let params = [
            ("q", query),
            ("format", "json"),
            ("limit", "5"),
        ];

        let mut request = self.client
            .get(&url)
            .query(&params);

        // Add API key header if provided
        if let Some(api_key) = &self.api_key {
            request = request.header("X-API-Token", api_key);
        }

        let http_response = request.send().await?;

        // Check HTTP status
        if !http_response.status().is_success() {
            let status = http_response.status();
            let error_text = http_response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Search API error: HTTP {}\nURL: {}\nResponse: {}",
                status,
                url,
                error_text
            ));
        }

        // Get the raw response text
        let response_text = http_response.text().await?;

        // Try to parse as JSON with better error handling
        let response: SearXNGResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!(
                "Failed to parse search response as JSON: {}\nURL: {}\nRaw response (first 500 chars): {}",
                e,
                url,
                response_text.chars().take(500).collect::<String>()
            ))?;

        // Check if we got any results
        if response.results.is_empty() {
            return Ok("No search results found.".to_string());
        }

        // Format search results for inclusion in LLM context
        let mut results = String::new();
        for (i, result) in response.results.iter().take(3).enumerate() {
            results.push_str(&format!(
                "{}. {}\n   URL: {}\n   {}\n\n",
                i + 1,
                result.title,
                result.url,
                Self::truncate(&result.content, 200)
            ));
        }

        Ok(results)
    }

    fn truncate(text: &str, max_chars: usize) -> String {
        text.chars().take(max_chars).collect()
    }
}

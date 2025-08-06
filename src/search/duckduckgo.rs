use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{SearchResult, SearchResults};

#[derive(Debug, Serialize, Deserialize)]
pub struct DuckDuckGoResponse {
    #[serde(rename = "Abstract")]
    pub abstract_text: String,
    #[serde(rename = "AbstractText")]
    pub abstract_text_alt: String,
    #[serde(rename = "AbstractSource")]
    pub abstract_source: String,
    #[serde(rename = "AbstractURL")]
    pub abstract_url: String,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "Heading")]
    pub heading: String,
    #[serde(rename = "Answer")]
    pub answer: String,
    #[serde(rename = "AnswerType")]
    pub answer_type: String,
    #[serde(rename = "Definition")]
    pub definition: String,
    #[serde(rename = "DefinitionSource")]
    pub definition_source: String,
    #[serde(rename = "DefinitionURL")]
    pub definition_url: String,
    #[serde(rename = "RelatedTopics")]
    pub related_topics: Vec<DuckDuckGoRelatedTopic>,
    #[serde(rename = "Results")]
    pub results: Vec<DuckDuckGoResult>,
    #[serde(rename = "Type")]
    pub result_type: String,
    #[serde(rename = "Redirect")]
    pub redirect: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuckDuckGoRelatedTopic {
    #[serde(rename = "Result")]
    pub result: Option<String>,
    #[serde(rename = "Icon")]
    pub icon: Option<DuckDuckGoIcon>,
    #[serde(rename = "FirstURL")]
    pub first_url: Option<String>,
    #[serde(rename = "Text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuckDuckGoResult {
    #[serde(rename = "Result")]
    pub result: String,
    #[serde(rename = "FirstURL")]
    pub first_url: String,
    #[serde(rename = "Icon")]
    pub icon: Option<DuckDuckGoIcon>,
    #[serde(rename = "Text")]
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuckDuckGoIcon {
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "Height")]
    pub height: Option<serde_json::Value>,
    #[serde(rename = "Width")]
    pub width: Option<serde_json::Value>,
}

pub struct DuckDuckGoProvider {
    pub url: String,
    pub headers: HashMap<String, String>,
}

impl DuckDuckGoProvider {
    pub fn new(url: String, headers: HashMap<String, String>) -> Self {
        Self { url, headers }
    }

    pub async fn search(&self, query: &str, count: Option<usize>) -> Result<SearchResults> {
        let client = reqwest::Client::new();

        // Build query parameters for DuckDuckGo Instant Answer API
        let params = vec![
            ("q", query.to_string()),
            ("format", "json".to_string()),
            ("no_redirect", "1".to_string()),
            ("no_html", "1".to_string()),
            ("skip_disambig", "1".to_string()),
        ];

        crate::debug_log!(
            "DuckDuckGo: Making GET request to {} with params: {:?}",
            self.url,
            params
        );

        let mut request = client.get(&self.url).query(&params);

        // Add custom headers if provided
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;

        let status = response.status();
        crate::debug_log!("DuckDuckGo: Received response with status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            crate::debug_log!("DuckDuckGo: Error response: {}", error_text);
            anyhow::bail!(
                "DuckDuckGo request failed with status {}: {}",
                status,
                error_text
            );
        }

        let response_text = response.text().await?;
        crate::debug_log!(
            "DuckDuckGo: Response body length: {} bytes",
            response_text.len()
        );

        let ddg_response: DuckDuckGoResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse DuckDuckGo response: {}", e))?;

        crate::debug_log!("DuckDuckGo: Successfully parsed response");

        let mut results = Vec::new();
        let max_results = count.unwrap_or(10);

        // Add abstract/answer as first result if available
        if !ddg_response.abstract_text.is_empty() && !ddg_response.abstract_url.is_empty() {
            let title = if !ddg_response.heading.is_empty() {
                ddg_response.heading.clone()
            } else {
                format!("About {}", query)
            };

            results.push(SearchResult {
                title,
                url: ddg_response.abstract_url.clone(),
                snippet: ddg_response.abstract_text.clone(),
                published_date: None,
                author: Some(ddg_response.abstract_source.clone()),
                score: None,
            });
        }

        // Add definition if available
        if !ddg_response.definition.is_empty() && !ddg_response.definition_url.is_empty() {
            results.push(SearchResult {
                title: format!("Definition: {}", query),
                url: ddg_response.definition_url.clone(),
                snippet: ddg_response.definition.clone(),
                published_date: None,
                author: Some(ddg_response.definition_source.clone()),
                score: None,
            });
        }

        // Add answer if available
        if !ddg_response.answer.is_empty() {
            results.push(SearchResult {
                title: format!("Answer: {}", query),
                url: format!("https://duckduckgo.com/?q={}", urlencoding::encode(query)),
                snippet: ddg_response.answer.clone(),
                published_date: None,
                author: Some("DuckDuckGo".to_string()),
                score: None,
            });
        }

        // Add results from Results array
        for result in ddg_response
            .results
            .iter()
            .take(max_results.saturating_sub(results.len()))
        {
            if !result.text.is_empty() && !result.first_url.is_empty() {
                // Extract title from the result text (usually the first part before " - ")
                let title = if let Some(dash_pos) = result.text.find(" - ") {
                    result.text[..dash_pos].to_string()
                } else {
                    result.text.clone()
                };

                let snippet = if let Some(dash_pos) = result.text.find(" - ") {
                    result.text[dash_pos + 3..].to_string()
                } else {
                    String::new()
                };

                results.push(SearchResult {
                    title,
                    url: result.first_url.clone(),
                    snippet,
                    published_date: None,
                    author: None,
                    score: None,
                });
            }
        }

        // Add related topics if we need more results
        if results.len() < max_results {
            for topic in ddg_response
                .related_topics
                .iter()
                .take(max_results.saturating_sub(results.len()))
            {
                if let (Some(text), Some(url)) = (&topic.text, &topic.first_url) {
                    if !text.is_empty() && !url.is_empty() {
                        // Extract title from the topic text
                        let title = if let Some(dash_pos) = text.find(" - ") {
                            text[..dash_pos].to_string()
                        } else {
                            text.clone()
                        };

                        let snippet = if let Some(dash_pos) = text.find(" - ") {
                            text[dash_pos + 3..].to_string()
                        } else {
                            String::new()
                        };

                        results.push(SearchResult {
                            title,
                            url: url.clone(),
                            snippet,
                            published_date: None,
                            author: None,
                            score: None,
                        });
                    }
                }
            }
        }

        crate::debug_log!(
            "DuckDuckGo: Successfully extracted {} results",
            results.len()
        );

        Ok(SearchResults {
            query: query.to_string(),
            provider: "DuckDuckGo".to_string(),
            results,
            total_results: None,  // DuckDuckGo API doesn't provide total count
            search_time_ms: None, // API doesn't provide timing info
        })
    }
}

/// Search function that matches the interface used by other providers
pub async fn search(
    provider_config: &super::SearchProviderConfig,
    query: &str,
    count: Option<usize>,
) -> anyhow::Result<super::SearchResults> {
    let provider =
        DuckDuckGoProvider::new(provider_config.url.clone(), provider_config.headers.clone());

    provider.search(query, count).await
}

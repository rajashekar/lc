use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{SearchResult, SearchResults};

#[derive(Debug, Serialize, Deserialize)]
pub struct JinaSearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JinaDirectResponse {
    pub data: Vec<JinaDirectResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JinaDirectResult {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub content: String,
}

pub struct JinaProvider {
    pub url: String,
    pub headers: HashMap<String, String>,
}

impl JinaProvider {
    pub fn new(url: String, headers: HashMap<String, String>) -> Self {
        Self { url, headers }
    }

    pub async fn search(&self, query: &str, count: Option<usize>) -> Result<SearchResults> {
        let client = reqwest::Client::new();
        
        // Build query parameters
        let params = vec![("q", query.to_string())];
        
        crate::debug_log!("Jina: Making GET request to {} with params: {:?}", self.url, params);
        
        let mut request = client
            .get(&self.url)
            .query(&params);
        
        // Check if full content reading is enabled
        let use_full_content = self.headers.contains_key("X-Engine") &&
                              self.headers.get("X-Engine").map_or(false, |v| v == "direct");
        
        // Add headers
        for (name, value) in &self.headers {
            if name == "Authorization" {
                // For Authorization header, use Bearer format
                request = request.header(name, format!("Bearer {}", value));
                crate::debug_log!("Jina: Added Authorization header with Bearer token");
            } else {
                request = request.header(name, value);
                crate::debug_log!("Jina: Added header {}: {}", name, value);
            }
        }
        
        // Check if we should request JSON format
        let want_json = self.headers.contains_key("Accept") &&
                       self.headers.get("Accept").map_or(false, |v| v.contains("application/json"));
        
        if use_full_content {
            crate::debug_log!("Jina: Using X-Engine: direct for full content reading");
        }
        
        if want_json {
            crate::debug_log!("Jina: Requesting JSON format");
        } else {
            crate::debug_log!("Jina: Requesting default text format");
        }
        
        let response = request.send().await?;
        
        let status = response.status();
        crate::debug_log!("Jina: Received response with status: {}", status);
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            crate::debug_log!("Jina: Error response: {}", error_text);
            anyhow::bail!("Jina request failed with status {}: {}", status, error_text);
        }
        
        let response_text = response.text().await?;
        crate::debug_log!("Jina: Response body length: {} bytes", response_text.len());
        
        let mut results = Vec::new();
        let max_results = count.unwrap_or(10);
        
        if want_json {
            // Parse JSON response
            crate::debug_log!("Jina: Parsing JSON response");
            
            if use_full_content {
                // When using X-Engine: direct, Jina returns a different JSON structure
                crate::debug_log!("Jina: Parsing direct engine JSON response");
                let direct_response: JinaDirectResponse = serde_json::from_str(&response_text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Jina direct JSON response: {}", e))?;
                
                for (index, result) in direct_response.data.iter().enumerate() {
                    if index >= max_results {
                        break;
                    }
                    
                    let search_result = SearchResult {
                        title: result.title.clone(),
                        url: result.url.clone(),
                        snippet: if !result.content.is_empty() {
                            // With X-Engine: direct, content contains the full page content
                            result.content.clone()
                        } else if !result.description.is_empty() {
                            result.description.clone()
                        } else {
                            "No content available".to_string()
                        },
                        published_date: None,
                        author: None,
                        score: None,
                    };
                    
                    results.push(search_result);
                }
            } else {
                // Standard JSON response format
                let jina_results: Vec<JinaSearchResult> = serde_json::from_str(&response_text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse Jina JSON response: {}", e))?;
                
                for (index, result) in jina_results.iter().enumerate() {
                    if index >= max_results {
                        break;
                    }
                    
                    let search_result = SearchResult {
                        title: result.title.clone(),
                        url: result.url.clone(),
                        snippet: if !result.description.is_empty() {
                            result.description.clone()
                        } else {
                            result.content.clone()
                        },
                        published_date: None,
                        author: None,
                        score: None,
                    };
                    
                    results.push(search_result);
                }
            }
        } else {
            // Parse text response format
            crate::debug_log!("Jina: Parsing text response");
            let lines: Vec<&str> = response_text.lines().collect();
            let mut current_result: Option<(String, String, String)> = None; // (title, url, description)
            
            for line in lines {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                // Parse format like: [1] Title: Title text
                if let Some(title_match) = line.strip_prefix("[") {
                    if let Some(end_bracket) = title_match.find("] Title: ") {
                        let title = title_match[end_bracket + 9..].to_string();
                        if let Some((prev_title, prev_url, prev_desc)) = current_result.take() {
                            // Save previous result
                            if !prev_title.is_empty() && !prev_url.is_empty() && results.len() < max_results {
                                results.push(SearchResult {
                                    title: prev_title,
                                    url: prev_url,
                                    snippet: prev_desc,
                                    published_date: None,
                                    author: None,
                                    score: None,
                                });
                            }
                        }
                        current_result = Some((title, String::new(), String::new()));
                        continue;
                    }
                }
                
                // Parse format like: [1] URL Source: https://example.com
                if let Some(url_match) = line.strip_prefix("[") {
                    if let Some(end_bracket) = url_match.find("] URL Source: ") {
                        let url = url_match[end_bracket + 13..].to_string();
                        if let Some((title, _, desc)) = current_result.take() {
                            current_result = Some((title, url, desc));
                        }
                        continue;
                    }
                }
                
                // Parse format like: [1] Description: Description text
                if let Some(desc_match) = line.strip_prefix("[") {
                    if let Some(end_bracket) = desc_match.find("] Description: ") {
                        let description = desc_match[end_bracket + 15..].to_string();
                        if let Some((title, url, _)) = current_result.take() {
                            current_result = Some((title, url, description));
                        }
                        continue;
                    }
                }
            }
            
            // Don't forget the last result
            if let Some((title, url, desc)) = current_result {
                if !title.is_empty() && !url.is_empty() && results.len() < max_results {
                    results.push(SearchResult {
                        title,
                        url,
                        snippet: desc,
                        published_date: None,
                        author: None,
                        score: None,
                    });
                }
            }
        }
        
        crate::debug_log!("Jina: Successfully extracted {} results", results.len());
        
        Ok(SearchResults {
            query: query.to_string(),
            provider: "Jina".to_string(),
            results,
            total_results: None, // Jina doesn't provide total count
            search_time_ms: None, // Jina doesn't provide timing info
        })
    }
}

/// Search function that matches the interface used by other providers
pub async fn search(
    provider_config: &super::SearchProviderConfig,
    query: &str,
    count: Option<usize>,
) -> anyhow::Result<super::SearchResults> {
    let provider = JinaProvider::new(
        provider_config.url.clone(),
        provider_config.headers.clone(),
    );
    
    provider.search(query, count).await
}
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{SearchResult, SearchResults};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiRequest {
    pub engine: String,
    pub q: String,
    pub num: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiResponse {
    pub organic_results: Option<Vec<SerpApiOrganicResult>>,
    pub answer_box: Option<SerpApiAnswerBox>,
    pub knowledge_graph: Option<SerpApiKnowledgeGraph>,
    pub search_metadata: Option<SerpApiSearchMetadata>,
    pub search_parameters: Option<SerpApiSearchParameters>,
    pub search_information: Option<SerpApiSearchInformation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiOrganicResult {
    pub position: Option<u32>,
    pub title: Option<String>,
    pub link: Option<String>,
    pub displayed_link: Option<String>,
    pub snippet: Option<String>,
    pub date: Option<String>,
    pub cached_page_link: Option<String>,
    pub related_pages_link: Option<String>,
    // SerpApi sometimes returns sitelinks as an object or array, so we'll skip it for now
    #[serde(skip_deserializing)]
    pub sitelinks: Option<Vec<SerpApiSitelink>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiSitelink {
    pub title: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiAnswerBox {
    #[serde(rename = "type")]
    pub answer_type: Option<String>,
    pub title: Option<String>,
    pub answer: Option<String>,
    pub snippet: Option<String>,
    pub link: Option<String>,
    pub displayed_link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiKnowledgeGraph {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub kg_type: Option<String>,
    pub description: Option<String>,
    pub source: Option<SerpApiKnowledgeGraphSource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiKnowledgeGraphSource {
    pub name: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiSearchMetadata {
    pub id: Option<String>,
    pub status: Option<String>,
    pub json_endpoint: Option<String>,
    pub created_at: Option<String>,
    pub processed_at: Option<String>,
    pub google_url: Option<String>,
    pub raw_html_file: Option<String>,
    pub total_time_taken: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiSearchParameters {
    pub engine: Option<String>,
    pub q: Option<String>,
    pub google_domain: Option<String>,
    pub hl: Option<String>,
    pub gl: Option<String>,
    pub device: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerpApiSearchInformation {
    pub organic_results_state: Option<String>,
    pub query_displayed: Option<String>,
    pub total_results: Option<u64>,
    pub time_taken_displayed: Option<f64>,
}

pub struct SerpApiProvider {
    pub url: String,
    pub headers: HashMap<String, String>,
}

impl SerpApiProvider {
    pub fn new(url: String, headers: HashMap<String, String>) -> Self {
        Self { url, headers }
    }

    pub async fn search(&self, query: &str, count: Option<usize>) -> Result<SearchResults> {
        let client = reqwest::Client::new();
        
        // Build query parameters
        let mut params = vec![
            ("engine", "google".to_string()),
            ("q", query.to_string()),
        ];
        
        if let Some(num) = count {
            params.push(("num", num.to_string()));
        }
        
        // Add API key from headers
        if let Some(api_key) = self.headers.get("api_key") {
            params.push(("api_key", api_key.clone()));
        }
        
        crate::debug_log!("SerpApi: Making GET request to {} with params: {:?}", self.url, params);
        
        let response = client
            .get(&self.url)
            .query(&params)
            .send()
            .await?;
        
        let status = response.status();
        crate::debug_log!("SerpApi: Received response with status: {}", status);
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            crate::debug_log!("SerpApi: Error response: {}", error_text);
            anyhow::bail!("SerpApi request failed with status {}: {}", status, error_text);
        }
        
        let response_text = response.text().await?;
        crate::debug_log!("SerpApi: Response body length: {} bytes", response_text.len());
        
        let serpapi_response: SerpApiResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse SerpApi response: {}", e))?;
        
        crate::debug_log!("SerpApi: Successfully parsed response");
        
        // Convert to our standard format
        let mut results = Vec::new();
        
        if let Some(organic_results) = serpapi_response.organic_results {
            crate::debug_log!("SerpApi: Processing {} organic results", organic_results.len());
            
            for result in organic_results {
                if let (Some(title), Some(url)) = (result.title, result.link) {
                    let search_result = SearchResult {
                        title,
                        url,
                        snippet: result.snippet.unwrap_or_default(),
                        published_date: result.date,
                        author: None,
                        score: None,
                    };
                    results.push(search_result);
                }
            }
        }
        
        crate::debug_log!("SerpApi: Converted {} results to standard format", results.len());
        
        // Extract total results and search time from metadata if available
        let total_results = serpapi_response.search_information
            .and_then(|info| info.total_results);
        let search_time_ms = serpapi_response.search_metadata
            .and_then(|meta| meta.total_time_taken)
            .map(|time| (time * 1000.0) as u64);
        
        Ok(SearchResults {
            query: query.to_string(),
            provider: "SerpApi".to_string(),
            results,
            total_results,
            search_time_ms,
        })
    }
}

/// Search function that matches the interface used by other providers
pub async fn search(
    provider_config: &super::SearchProviderConfig,
    query: &str,
    count: Option<usize>,
) -> anyhow::Result<super::SearchResults> {
    let provider = SerpApiProvider::new(
        provider_config.url.clone(),
        provider_config.headers.clone(),
    );
    
    provider.search(query, count).await
}
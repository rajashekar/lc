use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResults {
    pub query: String,
    pub provider: String,
    pub results: Vec<SearchResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_results: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_time_ms: Option<u64>,
}

impl SearchResults {
    pub fn new(query: String, provider: String) -> Self {
        Self {
            query,
            provider,
            results: Vec::new(),
            total_results: None,
            search_time_ms: None,
        }
    }

    pub fn add_result(&mut self, result: SearchResult) {
        self.results.push(result);
    }

    #[allow(dead_code)]
    pub fn set_total_results(&mut self, total: u64) {
        self.total_results = Some(total);
    }

    pub fn set_search_time(&mut self, time_ms: u64) {
        self.search_time_ms = Some(time_ms);
    }
}

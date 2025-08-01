use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchProviderType {
    Brave,
    // Future providers
    // DuckDuckGo,
    // Serper,
    // SerpApi,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchProviderConfig {
    pub url: String,
    pub provider_type: SearchProviderType,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl SearchProviderConfig {
    pub fn new(url: String, provider_type: SearchProviderType) -> Self {
        Self {
            url,
            provider_type,
            headers: HashMap::new(),
        }
    }
}

pub trait SearchProvider {
    fn name(&self) -> &str;
    fn search(&self, query: &str, count: Option<usize>) -> impl std::future::Future<Output = anyhow::Result<super::SearchResults>> + Send;
}
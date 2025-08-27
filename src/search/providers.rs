use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchProviderType {
    Brave,
    Exa,
    Serper,
    SerpApi,
    DuckDuckGo,
    Jina,
    Tavily,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchProviderConfig {
    pub url: String,
    pub provider_type: SearchProviderType,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl SearchProviderConfig {
    #[allow(dead_code)]
    pub fn new(url: String, provider_type: SearchProviderType) -> Self {
        Self {
            url,
            provider_type,
            headers: HashMap::new(),
        }
    }
}

impl SearchProviderType {
    /// Auto-detect provider type from URL
    pub fn detect_from_url(url: &str) -> anyhow::Result<Self> {
        let url_lower = url.to_lowercase();

        if url_lower.contains("api.search.brave.com") {
            Ok(SearchProviderType::Brave)
        } else if url_lower.contains("api.exa.ai") || url_lower.contains("exa.ai") {
            Ok(SearchProviderType::Exa)
        } else if url_lower.contains("google.serper.dev") || url_lower.contains("serper.dev") {
            Ok(SearchProviderType::Serper)
        } else if url_lower.contains("serpapi.com") {
            Ok(SearchProviderType::SerpApi)
        } else if url_lower.contains("duckduckgo.com") || url_lower.contains("api.duckduckgo.com") {
            Ok(SearchProviderType::DuckDuckGo)
        } else if url_lower.contains("jina.ai") || url_lower.contains("s.jina.ai") {
            Ok(SearchProviderType::Jina)
        } else if url_lower.contains("api.tavily.com") || url_lower.contains("tavily.com") {
            Ok(SearchProviderType::Tavily)
        } else {
            anyhow::bail!(
                "Cannot auto-detect provider type from URL '{}'. \
                Supported providers:\n\
                - Brave: api.search.brave.com\n\
                - Exa: api.exa.ai\n\
                - Serper: google.serper.dev\n\
                - SerpApi: serpapi.com\n\
                - DuckDuckGo: api.duckduckgo.com\n\
                - Jina: s.jina.ai\n\
                - Tavily: api.tavily.com",
                url
            )
        }
    }

    /// Get the correct API key header name for this provider type
    pub fn api_key_header(&self) -> &'static str {
        match self {
            SearchProviderType::Brave => "X-Subscription-Token",
            SearchProviderType::Exa => "x-api-key",
            SearchProviderType::Serper => "X-API-KEY",
            SearchProviderType::SerpApi => "api_key",
            SearchProviderType::DuckDuckGo => "", // No API key required
            SearchProviderType::Jina => "Authorization",
            SearchProviderType::Tavily => "Authorization",
        }
    }
}

#[allow(dead_code)]
pub trait SearchProvider {
    fn name(&self) -> &str;
    fn search(
        &self,
        query: &str,
        count: Option<usize>,
    ) -> impl std::future::Future<Output = anyhow::Result<super::SearchResults>> + Send;
}

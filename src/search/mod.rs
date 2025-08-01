pub mod config;
pub mod providers;
pub mod brave;
pub mod search_result;

use anyhow::Result;

pub use config::SearchConfig;
pub use providers::{SearchProviderType, SearchProviderConfig};
pub use search_result::{SearchResult, SearchResults};

/// Main search interface
pub struct SearchEngine {
    config: SearchConfig,
}

impl SearchEngine {
    pub fn new() -> Result<Self> {
        let config = SearchConfig::load()?;
        Ok(Self { config })
    }

    pub async fn search(
        &self,
        provider_name: &str,
        query: &str,
        count: Option<usize>,
    ) -> Result<SearchResults> {
        let provider_config = self.config.get_provider(provider_name)?;
        
        match provider_config.provider_type {
            SearchProviderType::Brave => {
                brave::search(provider_config, query, count).await
            }
            // Future providers will be added here
        }
    }

    pub fn format_results_json(&self, results: &SearchResults) -> Result<String> {
        Ok(serde_json::to_string_pretty(results)?)
    }

    pub fn format_results_markdown(&self, results: &SearchResults) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# Search Results for: {}\n\n", results.query));
        output.push_str(&format!("Provider: {} | Total Results: {}\n\n", 
            results.provider, results.results.len()));
        
        for (i, result) in results.results.iter().enumerate() {
            output.push_str(&format!("## {}. {}\n", i + 1, result.title));
            output.push_str(&format!("**URL:** {}\n\n", result.url));
            output.push_str(&format!("{}\n\n", result.snippet));
            
            if let Some(published) = &result.published_date {
                output.push_str(&format!("*Published: {}*\n\n", published));
            }
            
            output.push_str("---\n\n");
        }
        
        output
    }

    pub fn extract_context_for_llm(&self, results: &SearchResults, max_results: usize) -> String {
        let mut context = String::new();
        context.push_str("Web search results:\n\n");
        
        for (i, result) in results.results.iter().take(max_results).enumerate() {
            context.push_str(&format!("{}. **{}**\n", i + 1, result.title));
            context.push_str(&format!("   URL: {}\n", result.url));
            context.push_str(&format!("   {}\n\n", result.snippet));
        }
        
        context
    }
}
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{SearchProviderConfig, SearchResult, SearchResults};

#[derive(Debug, Serialize)]
struct SerperRequest {
    q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    num: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct SerperResponse {
    #[serde(default)]
    organic: Vec<SerperOrganicResult>,
    #[serde(rename = "searchParameters")]
    #[allow(dead_code)]
    search_parameters: Option<SerperSearchParameters>,
    #[serde(rename = "topStories", default)]
    #[allow(dead_code)]
    top_stories: Vec<SerperTopStory>,
    #[serde(rename = "relatedSearches", default)]
    #[allow(dead_code)]
    related_searches: Vec<SerperRelatedSearch>,
    #[serde(rename = "peopleAlsoAsk", default)]
    #[allow(dead_code)]
    people_also_ask: Vec<SerperPeopleAlsoAsk>,
    #[serde(default)]
    #[allow(dead_code)]
    credits: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct SerperOrganicResult {
    title: String,
    link: String,
    snippet: String,
    #[serde(default)]
    position: i32,
    #[serde(rename = "date")]
    published_date: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    sitelinks: Vec<SerperSitelink>,
}

#[derive(Debug, Deserialize)]
struct SerperSitelink {
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    link: String,
}

#[derive(Debug, Deserialize)]
struct SerperSearchParameters {
    #[allow(dead_code)]
    q: String,
}

#[derive(Debug, Deserialize)]
struct SerperTopStory {
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    link: String,
    #[allow(dead_code)]
    date: Option<String>,
    #[allow(dead_code)]
    source: Option<String>,
    #[serde(rename = "imageUrl")]
    #[allow(dead_code)]
    image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerperRelatedSearch {
    #[allow(dead_code)]
    query: String,
}

#[derive(Debug, Deserialize)]
struct SerperPeopleAlsoAsk {
    #[allow(dead_code)]
    question: String,
    #[allow(dead_code)]
    snippet: String,
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    link: String,
}

pub async fn search(
    config: &SearchProviderConfig,
    query: &str,
    count: Option<usize>,
) -> Result<SearchResults> {
    let client = reqwest::Client::new();
    
    let request_body = SerperRequest {
        q: query.to_string(),
        num: count,
    };
    
    // Handle URL construction to avoid duplication
    let url = if config.url.ends_with("/search") {
        config.url.clone()
    } else {
        format!("{}/search", config.url.trim_end_matches('/'))
    };
    
    let mut request = client
        .post(&url)
        .json(&request_body);
    
    // Add headers
    for (key, value) in &config.headers {
        request = request.header(key, value);
    }
    
    let response = request.send().await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Serper API error ({}): {}", status, error_text);
    }
    
    let serper_response: SerperResponse = response.json().await?;
    
    // Convert Serper results to our common format
    let results: Vec<SearchResult> = serper_response.organic
        .into_iter()
        .map(|result| SearchResult {
            title: result.title,
            url: result.link,
            snippet: result.snippet,
            score: Some(1.0 - (result.position as f32 * 0.1)), // Simple scoring based on position
            published_date: result.published_date,
            author: None,
        })
        .collect();
    
    Ok(SearchResults {
        query: query.to_string(),
        provider: "serper".to_string(),
        results,
        total_results: None,
        search_time_ms: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serper_response_parsing() {
        let json_response = r#"{
            "searchParameters": {
                "q": "apple inc",
                "type": "search",
                "engine": "google"
            },
            "organic": [
                {
                    "title": "Apple",
                    "link": "https://www.apple.com/",
                    "snippet": "Discover the innovative world of Apple and shop everything iPhone, iPad, Apple Watch, Mac, and Apple TV, plus explore accessories, entertainment, ...",
                    "position": 1
                },
                {
                    "title": "Apple Inc. - Wikipedia",
                    "link": "https://en.wikipedia.org/wiki/Apple_Inc.",
                    "snippet": "Apple Inc. is an American multinational corporation and technology company headquartered in Cupertino, California, in Silicon Valley.",
                    "position": 2,
                    "date": "2024-01-15"
                }
            ]
        }"#;
        
        let response: SerperResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.organic.len(), 2);
        assert_eq!(response.organic[0].title, "Apple");
        assert_eq!(response.organic[0].position, 1);
        assert_eq!(response.organic[1].published_date, Some("2024-01-15".to_string()));
    }
}
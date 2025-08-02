use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::{SearchProviderConfig, SearchResult, SearchResults};

#[derive(Debug, Serialize)]
struct ExaSearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_results: Option<usize>,
    contents: ExaContentsRequest,
}

#[derive(Debug, Serialize)]
struct ExaContentsRequest {
    text: bool,
}

#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    title: String,
    url: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
    author: Option<String>,
    score: Option<f64>,
}

pub async fn search(
    provider_config: &SearchProviderConfig,
    query: &str,
    count: Option<usize>,
) -> Result<SearchResults> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let request_body = ExaSearchRequest {
        query: query.to_string(),
        num_results: count,
        contents: ExaContentsRequest { text: true },
    };

    // The provider_config.url should be the complete search endpoint URL
    // For Exa, it should be https://api.exa.ai/search
    let base_url = provider_config.url.trim_end_matches('/');
    let url = if base_url.ends_with("/search") {
        // URL already includes the endpoint path
        base_url.to_string()
    } else {
        // URL is just the base, append the endpoint
        format!("{}/search", base_url)
    };
    let mut request = client
        .post(&url)
        .json(&request_body);

    // Add headers
    for (name, value) in &provider_config.headers {
        request = request.header(name, value);
    }

    let start_time = std::time::Instant::now();
    let response = request.send().await?;
    let search_time_ms = start_time.elapsed().as_millis() as u64;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Exa search API error ({}): {}", status, error_text);
    }

    let exa_response: ExaSearchResponse = response.json().await?;
    
    let mut results = SearchResults::new(query.to_string(), "exa".to_string());
    results.set_search_time(search_time_ms);

    for exa_result in exa_response.results {
        // Use text content as snippet, or title if text is not available
        let snippet = exa_result.text.unwrap_or_else(|| exa_result.title.clone());

        let search_result = SearchResult {
            title: exa_result.title,
            url: exa_result.url,
            snippet,
            published_date: exa_result.published_date,
            author: exa_result.author,
            score: exa_result.score.map(|s| s as f32),
        };

        results.add_result(search_result);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exa_response_parsing() {
        let json_response = r#"{
            "results": [
                {
                    "title": "Understanding AI Safety",
                    "url": "https://example.com/ai-safety",
                    "text": "AI safety is a critical field of research...",
                    "publishedDate": "2024-01-15",
                    "author": "Jane Doe",
                    "score": 0.95
                }
            ]
        }"#;

        let response: ExaSearchResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Understanding AI Safety");
        assert_eq!(response.results[0].score, Some(0.95));
    }
}
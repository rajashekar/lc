use super::{SearchProviderConfig, SearchResult, SearchResults};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct TavilySearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_parameters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chunks_per_source: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_answer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_raw_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_image_descriptions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_favicon: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TavilySearchResponse {
    #[allow(dead_code)]
    query: String,
    #[serde(default)]
    #[allow(dead_code)]
    answer: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    images: Option<Vec<String>>,
    results: Vec<TavilyResult>,
    #[serde(default)]
    #[allow(dead_code)]
    auto_parameters: Option<TavilyAutoParameters>,
    #[serde(default)]
    #[allow(dead_code)]
    response_time: Option<f64>,
    #[serde(default)]
    #[allow(dead_code)]
    request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f64,
    #[serde(default)]
    #[allow(dead_code)]
    raw_content: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    favicon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TavilyAutoParameters {
    #[serde(default)]
    #[allow(dead_code)]
    topic: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    search_depth: Option<String>,
}

pub async fn search(
    provider_config: &SearchProviderConfig,
    query: &str,
    count: Option<usize>,
) -> Result<SearchResults> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let request_body = TavilySearchRequest {
        query: query.to_string(),
        auto_parameters: Some(false),
        topic: Some("general".to_string()),
        search_depth: Some("basic".to_string()),
        chunks_per_source: Some(3),
        max_results: count,
        time_range: None,
        days: Some(7),
        start_date: None,
        end_date: None,
        include_answer: Some(true),
        include_raw_content: Some(true),
        include_images: Some(false),
        include_image_descriptions: Some(false),
        include_favicon: Some(false),
        include_domains: Some(Vec::new()),
        exclude_domains: Some(Vec::new()),
        country: None,
    };

    // The provider_config.url should be the complete search endpoint URL
    // For Tavily, it should be https://api.tavily.com/search
    let base_url = provider_config.url.trim_end_matches('/');
    let url = if base_url.ends_with("/search") {
        // URL already includes the endpoint path
        base_url.to_string()
    } else {
        // URL is just the base, append the endpoint
        format!("{}/search", base_url)
    };

    let mut request = client.post(&url).json(&request_body);

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
        anyhow::bail!("Tavily search API error ({}): {}", status, error_text);
    }

    let tavily_response: TavilySearchResponse = response.json().await?;

    let mut results = SearchResults::new(query.to_string(), "tavily".to_string());
    results.set_search_time(search_time_ms);

    for tavily_result in tavily_response.results {
        let search_result = SearchResult {
            title: tavily_result.title,
            url: tavily_result.url,
            snippet: tavily_result.content,
            published_date: None, // Tavily doesn't provide published date in the basic response
            author: None,         // Tavily doesn't provide author information
            score: Some(tavily_result.score as f32),
        };

        results.add_result(search_result);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tavily_response_parsing() {
        let json_response = r#"{
            "query": "Who is Leo Messi?",
            "answer": "Lionel Messi, born in 1987, is an Argentine footballer widely regarded as one of the greatest players of his generation.",
            "images": [],
            "results": [
                {
                    "title": "Lionel Messi Facts | Britannica",
                    "url": "https://www.britannica.com/facts/Lionel-Messi",
                    "content": "Lionel Messi, an Argentine footballer, is widely regarded as one of the greatest football players of his generation.",
                    "score": 0.81025416,
                    "raw_content": null,
                    "favicon": "https://britannica.com/favicon.png"
                }
            ],
            "auto_parameters": {
                "topic": "general",
                "search_depth": "basic"
            },
            "response_time": 1.67,
            "request_id": "123e4567-e89b-12d3-a456-426614174111"
        }"#;

        let response: TavilySearchResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.query, "Who is Leo Messi?");
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Lionel Messi Facts | Britannica");
        assert_eq!(response.results[0].score, 0.81025416);
        assert!(response.answer.is_some());
    }

    #[test]
    fn test_tavily_request_serialization() {
        let request = TavilySearchRequest {
            query: "test query".to_string(),
            auto_parameters: Some(false),
            topic: Some("general".to_string()),
            search_depth: Some("basic".to_string()),
            chunks_per_source: Some(3),
            max_results: Some(5),
            time_range: None,
            days: Some(7),
            start_date: None,
            end_date: None,
            include_answer: Some(true),
            include_raw_content: Some(true),
            include_images: Some(false),
            include_image_descriptions: Some(false),
            include_favicon: Some(false),
            include_domains: Some(Vec::new()),
            exclude_domains: Some(Vec::new()),
            country: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test query"));
        assert!(json.contains("\"auto_parameters\":false"));
        assert!(json.contains("\"max_results\":5"));
    }
}

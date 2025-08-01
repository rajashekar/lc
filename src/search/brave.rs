use anyhow::Result;
use serde::Deserialize;
use super::{SearchProviderConfig, SearchResult, SearchResults};

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    #[allow(dead_code)]
    query: Query,
    web: Option<WebResults>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Query {
    original: String,
}

#[derive(Debug, Deserialize)]
struct WebResults {
    results: Vec<BraveWebResult>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResult {
    title: String,
    url: String,
    description: String,
    #[serde(rename = "age")]
    published_date: Option<String>,
    #[serde(rename = "extra_snippets")]
    extra_snippets: Option<Vec<String>>,
}

pub async fn search(
    provider_config: &SearchProviderConfig,
    query: &str,
    count: Option<usize>,
) -> Result<SearchResults> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut url = reqwest::Url::parse(&provider_config.url)?;
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("count", &count.unwrap_or(5).to_string());

    let mut request = client.get(url);

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
        anyhow::bail!("Brave search API error ({}): {}", status, error_text);
    }

    let brave_response: BraveSearchResponse = response.json().await?;
    
    let mut results = SearchResults::new(query.to_string(), "brave".to_string());
    results.set_search_time(search_time_ms);

    if let Some(web_results) = brave_response.web {
        for brave_result in web_results.results {
            let mut snippet = brave_result.description.clone();
            
            // Append extra snippets if available
            if let Some(extra) = &brave_result.extra_snippets {
                if !extra.is_empty() {
                    snippet.push_str(" ");
                    snippet.push_str(&extra.join(" "));
                }
            }

            let search_result = SearchResult {
                title: brave_result.title,
                url: brave_result.url,
                snippet,
                published_date: brave_result.published_date,
                author: None,
                score: None,
            };

            results.add_result(search_result);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brave_response_parsing() {
        let json_response = r#"{
            "query": {
                "original": "rust programming"
            },
            "web": {
                "results": [
                    {
                        "title": "Rust Programming Language",
                        "url": "https://www.rust-lang.org/",
                        "description": "A language empowering everyone to build reliable and efficient software.",
                        "age": "2023-12-01"
                    }
                ]
            }
        }"#;

        let response: BraveSearchResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.query.original, "rust programming");
        assert!(response.web.is_some());
        
        let web_results = response.web.unwrap();
        assert_eq!(web_results.results.len(), 1);
        assert_eq!(web_results.results[0].title, "Rust Programming Language");
    }
}
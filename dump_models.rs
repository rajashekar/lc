use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio;

// Import the existing modules
mod config;
mod provider;

use config::Config;
use provider::OpenAIClient;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Dumping /models for each provider...");
    
    // Load configuration
    let config = Config::load()?;
    
    // Create models directory if it doesn't exist
    fs::create_dir_all("models")?;
    
    let mut successful_dumps = 0;
    let mut total_providers = 0;
    
    for (provider_name, provider_config) in &config.providers {
        total_providers += 1;
        
        // Skip providers without API keys
        if provider_config.api_key.is_none() {
            println!("⚠️  Skipping {} (no API key)", provider_name);
            continue;
        }
        
        println!("📡 Fetching models from {}...", provider_name);
        
        // Create client with custom headers
        let client = OpenAIClient::new_with_headers(
            provider_config.endpoint.clone(),
            provider_config.api_key.clone().unwrap(),
            provider_config.models_path.clone(),
            provider_config.chat_path.clone(),
            provider_config.headers.clone(),
        );
        
        // Make raw request to get full JSON response
        match fetch_raw_models_response(&client, provider_config).await {
            Ok(raw_response) => {
                // Save raw response to file
                let filename = format!("models/{}.json", provider_name);
                match fs::write(&filename, &raw_response) {
                    Ok(_) => {
                        println!("✅ Saved {} models data to {}", provider_name, filename);
                        successful_dumps += 1;
                    }
                    Err(e) => {
                        println!("❌ Failed to save {} models data: {}", provider_name, e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to fetch models from {}: {}", provider_name, e);
            }
        }
    }
    
    println!("\n📊 Summary:");
    println!("   Total providers: {}", total_providers);
    println!("   Successful dumps: {}", successful_dumps);
    println!("   Models data saved to: ./models/");
    
    if successful_dumps > 0 {
        println!("\n🎉 Model data collection complete!");
        println!("   Next step: Analyze the JSON files to extract metadata patterns");
    }
    
    Ok(())
}

async fn fetch_raw_models_response(client: &OpenAIClient, provider_config: &config::ProviderConfig) -> Result<String> {
    let http_client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    
    let url = format!("{}{}", provider_config.endpoint.trim_end_matches('/'), provider_config.models_path);
    
    let mut req = http_client
        .get(&url)
        .header("Authorization", format!("Bearer {}", provider_config.api_key.as_ref().unwrap()))
        .header("Content-Type", "application/json");
    
    // Add custom headers
    for (name, value) in &provider_config.headers {
        req = req.header(name, value);
    }
    
    let response = req.send().await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("API request failed with status {}: {}", status, text);
    }
    
    let response_text = response.text().await?;
    
    // Pretty print the JSON for better readability
    match serde_json::from_str::<Value>(&response_text) {
        Ok(json_value) => {
            Ok(serde_json::to_string_pretty(&json_value)?)
        }
        Err(_) => {
            // If it's not valid JSON, return as-is
            Ok(response_text)
        }
    }
}
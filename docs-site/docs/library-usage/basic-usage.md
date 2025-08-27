# Basic Usage

Learn how to use LC as a library in your Rust code with practical examples.

## Your First LC Program

Here's a complete example that sends a chat message to an LLM:

```rust
use lc_cli::{Config, chat, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration (reads from ~/.config/lc/config.toml)
    let mut config = Config::load()?;
    
    // Create an authenticated client for your default provider
    let provider_name = config.default_provider.as_ref()
        .ok_or("No default provider configured")?;
    let client = chat::create_authenticated_client(&mut config, provider_name).await?;
    
    // Create a chat request
    let request = ChatRequest {
        messages: vec![
            Message::user("Hello! Can you help me with Rust programming?".to_string())
        ],
        model: "gpt-4".to_string(),
        max_tokens: Some(150),
        temperature: Some(0.7),
        stream: None,
        tools: None,
    };
    
    // Send the request and get response
    let response = client.chat(&request).await?;
    println!("AI Response: {}", response);
    
    Ok(())
}
```

## Core Components

### 1. Configuration Management

```rust
use lc_cli::Config;

// Load from default location (~/.config/lc/config.toml)
let config = Config::load()?;

// Load from specific file
let config = Config::load_from_path("/path/to/config.toml")?;

// Access provider settings
if let Some(openai_config) = config.providers.get("openai") {
    println!("OpenAI API key configured: {}", openai_config.api_key.is_some());
}
```

### 2. Creating Clients

```rust
use lc_cli::{Config, chat};

// Create authenticated client for a specific provider
let mut config = Config::load()?;
let client = chat::create_authenticated_client(&mut config, "openai").await?;

// Or use the default provider
let provider_name = config.default_provider.as_ref()
    .ok_or("No default provider configured")?;
let client = chat::create_authenticated_client(&mut config, provider_name).await?;
```

### 3. Building Chat Requests

```rust
use lc_cli::{ChatRequest, Message};

let request = ChatRequest {
    messages: vec![
        Message::assistant("You are a helpful coding assistant.".to_string()),
        Message::user("Explain async/await in Rust".to_string())
    ],
    model: "gpt-4".to_string(),
    max_tokens: Some(500),
    temperature: Some(0.3),
    stream: None,
    tools: None,
};
```

## Common Patterns

### Error Handling

```rust
use lc_cli::{Config, chat, ChatRequest, Message};
use anyhow::Result;

async fn chat_with_error_handling() -> Result<String> {
    let mut config = Config::load()?;
    
    let provider_name = config.default_provider.as_ref()
        .ok_or_else(|| anyhow::anyhow!("No default provider configured"))?;
    
    let client = chat::create_authenticated_client(&mut config, provider_name).await?;
    
    let request = ChatRequest {
        messages: vec![Message::user("Hello!".to_string())],
        model: "gpt-4".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        stream: None,
        tools: None,
    };
    
    match client.chat(&request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            eprintln!("Chat request failed: {}", e);
            Err(e)
        }
    }
}
```

### Multiple Providers

```rust
use lc_cli::{Config, chat, ChatRequest, Message};

async fn compare_responses(prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load()?;
    
    // Create clients for different providers
    let openai_client = chat::create_authenticated_client(&mut config, "openai").await?;
    let anthropic_client = chat::create_authenticated_client(&mut config, "anthropic").await?;
    
    let message = Message::user(prompt.to_string());
    
    // Send to OpenAI
    let openai_request = ChatRequest {
        messages: vec![message.clone()],
        model: "gpt-4".to_string(),
        max_tokens: Some(150),
        temperature: Some(0.7),
        stream: None,
        tools: None,
    };
    let openai_response = openai_client.chat(&openai_request).await?;
    
    // Send to Anthropic
    let anthropic_request = ChatRequest {
        messages: vec![message],
        model: "claude-3-sonnet-20240229".to_string(),
        max_tokens: Some(150),
        temperature: Some(0.7),
        stream: None,
        tools: None,
    };
    let anthropic_response = anthropic_client.chat(&anthropic_request).await?;
    
    println!("OpenAI: {}", openai_response);
    println!("Anthropic: {}", anthropic_response);
    
    Ok(())
}
```

### Streaming Responses

```rust
use lc_cli::{Config, chat, ChatRequest, Message};

async fn stream_chat() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load()?;
    let provider_name = config.default_provider.as_ref()
        .ok_or("No default provider configured")?;
    let client = chat::create_authenticated_client(&mut config, provider_name).await?;
    
    let request = ChatRequest {
        messages: vec![
            Message::user("Write a short story about a robot".to_string())
        ],
        model: "gpt-4".to_string(),
        max_tokens: Some(500),
        temperature: Some(0.7),
        stream: Some(true),
        tools: None,
    };
    
    // Use the streaming method (outputs directly to stdout)
    client.chat_stream(&request).await?;
    
    Ok(())
}
```

## CLI vs Library Comparison

| Task | CLI Command | Library Code |
|------|-------------|--------------|
| **Simple Chat** | `lc "Hello"` | `client.chat(&request).await` |
| **Specify Model** | `lc -m gpt-4 "Hello"` | `request.model = "gpt-4".to_string()` |
| **System Prompt** | `lc -s "You are helpful" "Hello"` | Add system message to `request.messages` |
| **Temperature** | `lc -t 0.7 "Hello"` | `request.temperature = Some(0.7)` |
| **Max Tokens** | `lc --max-tokens 100 "Hello"` | `request.max_tokens = Some(100)` |

## Next Steps

- [**Configuration**](configuration.md) - Set up providers and customize settings
- [**Advanced Features**](advanced-features.md) - Vector databases, sessions, and templates

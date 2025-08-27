# Library Usage Overview

LC (LLM Client) can be used not only as a command-line tool but also as a Rust library in your own projects. This allows you to integrate LLM capabilities directly into your applications without needing to shell out to the CLI.

## What You Get

When you use LC as a library, you get programmatic access to:

- **Configuration Management** - Load and manage provider configurations
- **Multiple LLM Providers** - OpenAI, Anthropic, Google Gemini, and more
- **Chat Functionality** - Send chat requests and handle responses
- **Vector Database** - Store and search embeddings
- **Session Management** - Maintain conversation history
- **Template Processing** - Use dynamic templates for prompts

## Library vs CLI

| Feature | CLI Usage | Library Usage |
|---------|-----------|---------------|
| **Execution** | `lc -m gpt-4 "Hello"` | `client.send_chat_request(request).await` |
| **Configuration** | Config files + flags | Programmatic config loading |
| **Output** | Terminal text | Structured data types |
| **Integration** | Shell scripts | Native Rust code |
| **Error Handling** | Exit codes | Result types |

## Use Cases

### Web Applications
Build web APIs that use LLMs internally:
```rust
// In your web handler
use lc_cli::{Config, OpenAIClient, ChatRequest, Message};

let config = Config::load()?;
let client = OpenAIClient::new(&config)?;
let request = ChatRequest {
    messages: vec![Message::user("Summarize this text".to_string())],
    model: "gpt-4".to_string(),
    ..Default::default()
};
let response = client.chat(&request).await?;
Json(response)
```

### Desktop Applications
Create GUI applications with LLM features:
```rust
// In your desktop app
use lc_cli::{Config, OpenAIClient, ChatRequest, Message};

let config = Config::load()?;
let client = OpenAIClient::new(&config)?;
let request = ChatRequest {
    messages: vec![Message::user(format!("Complete this code: {}", user_code))],
    model: "gpt-4".to_string(),
    ..Default::default()
};
let suggestion = client.chat(&request).await?;
display_suggestion(suggestion);
```

### Automation Scripts
Process files and data with LLM assistance:
```rust
// Batch processing
use lc_cli::{Config, OpenAIClient, ChatRequest, Message};

let config = Config::load()?;
let client = OpenAIClient::new(&config)?;

for file in files {
    let content = std::fs::read_to_string(file)?;
    let request = ChatRequest {
        messages: vec![Message::user(format!("Summarize this file: {}", content))],
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    let summary = client.chat(&request).await?;
    save_summary(summary);
}
```

### Integration with Existing Systems
Add LLM capabilities to existing Rust applications:
```rust
// In your existing service
use lc_cli::{Config, OpenAIClient, ChatRequest, Message};

let config = Config::load()?;
let client = OpenAIClient::new(&config)?;
let request = ChatRequest {
    messages: vec![Message::user(format!("Analyze this data: {:?}", metrics))],
    model: "gpt-4".to_string(),
    ..Default::default()
};
let analysis = client.chat(&request).await?;
update_dashboard(analysis);
```

## Getting Started

1. [**Installation**](installation.md) - Add LC to your Cargo.toml
2. [**Basic Usage**](basic-usage.md) - Your first LC library code
3. [**Configuration**](configuration.md) - Set up providers and models
4. [**Advanced Features**](advanced-features.md) - Vector DB, sessions, templates

## Architecture

LC library is structured as modules that you can import selectively:

```rust
use lc_cli::{
    Config,           // Configuration management
    OpenAIClient,     // LLM provider clients
    ChatRequest,      // Request/response types
    Database,         // Chat history storage
    VectorDB,         // Vector database operations
};
```

Each module is designed to work independently, so you can use only what you need for your specific use case.

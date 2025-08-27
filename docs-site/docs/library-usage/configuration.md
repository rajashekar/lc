# Configuration

Learn how to configure LC library for different providers and customize settings.

## Configuration Loading

LC library uses the same configuration system as the CLI tool. Configuration is stored in TOML format.

### Default Configuration Path

```rust
use lc_cli::Config;

// Loads from ~/.config/lc/config.toml (Linux/macOS)
// or %APPDATA%/lc/config.toml (Windows)
let config = Config::load()?;
```

### Custom Configuration Path

```rust
use lc_cli::Config;

let config = Config::load_from_path("/path/to/your/config.toml")?;
```

### Programmatic Configuration

```rust
use lc_cli::{Config, ProviderConfig};
use std::collections::HashMap;

let mut providers = HashMap::new();
providers.insert("openai".to_string(), ProviderConfig {
    api_key: Some("your-api-key".to_string()),
    base_url: Some("https://api.openai.com/v1".to_string()),
    model: Some("gpt-4".to_string()),
    ..Default::default()
});

let config = Config {
    providers,
    default_provider: Some("openai".to_string()),
    ..Default::default()
};
```

## Provider Setup

### OpenAI Configuration

```rust
use lc_cli::{Config, ProviderConfig};

// In your config.toml or programmatically:
let openai_config = ProviderConfig {
    api_key: Some("sk-your-openai-key".to_string()),
    base_url: Some("https://api.openai.com/v1".to_string()),
    model: Some("gpt-4".to_string()),
    max_tokens: Some(2000),
    temperature: Some(0.7),
    ..Default::default()
};
```

### Anthropic Configuration

```rust
let anthropic_config = ProviderConfig {
    api_key: Some("your-anthropic-key".to_string()),
    base_url: Some("https://api.anthropic.com".to_string()),
    model: Some("claude-3-sonnet-20240229".to_string()),
    max_tokens: Some(4000),
    temperature: Some(0.3),
    ..Default::default()
};
```

### Google Gemini Configuration

```rust
let gemini_config = ProviderConfig {
    api_key: Some("your-gemini-key".to_string()),
    base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
    model: Some("gemini-pro".to_string()),
    ..Default::default()
};
```

## Configuration File Format

Create a `config.toml` file with your provider settings:

```toml
default_provider = "openai"

[providers.openai]
api_key = "sk-your-openai-key"
base_url = "https://api.openai.com/v1"
model = "gpt-4"
max_tokens = 2000
temperature = 0.7

[providers.anthropic]
api_key = "your-anthropic-key"
base_url = "https://api.anthropic.com"
model = "claude-3-sonnet-20240229"
max_tokens = 4000
temperature = 0.3

[providers.gemini]
api_key = "your-gemini-key"
base_url = "https://generativelanguage.googleapis.com/v1beta"
model = "gemini-pro"
```

## Environment Variables

You can also use environment variables for sensitive data:

```rust
use std::env;
use lc_cli::ProviderConfig;

let openai_config = ProviderConfig {
    api_key: env::var("OPENAI_API_KEY").ok(),
    base_url: Some("https://api.openai.com/v1".to_string()),
    model: Some("gpt-4".to_string()),
    ..Default::default()
};
```

## Configuration Validation

```rust
use lc_cli::{Config, error::LcError};

async fn validate_config() -> Result<(), LcError> {
    let config = Config::load()?;
    
    // Check if default provider is configured
    if let Some(default_provider) = &config.default_provider {
        if let Some(provider_config) = config.providers.get(default_provider) {
            if provider_config.api_key.is_none() {
                return Err(LcError::Config(
                    format!("No API key configured for provider: {}", default_provider)
                ));
            }
            println!("✅ Provider {} is properly configured", default_provider);
        } else {
            return Err(LcError::Config(
                format!("Default provider {} not found in configuration", default_provider)
            ));
        }
    } else {
        return Err(LcError::Config("No default provider configured".to_string()));
    }
    
    Ok(())
}
```

## Dynamic Provider Selection

```rust
use lc_cli::{Config, OpenAIClient, provider::AnthropicClient};

async fn use_specific_provider(provider_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = Config::load()?;
    
    let response = match provider_name {
        "openai" => {
            let client = OpenAIClient::new(&config)?;
            let request = create_chat_request("gpt-4", "Hello!");
            client.send_chat_request(request).await?
        },
        "anthropic" => {
            let client = AnthropicClient::new(&config)?;
            let request = create_chat_request("claude-3-sonnet-20240229", "Hello!");
            client.send_chat_request(request).await?
        },
        _ => return Err(format!("Unsupported provider: {}", provider_name).into()),
    };
    
    Ok(response.content)
}

fn create_chat_request(model: &str, content: &str) -> ChatRequest {
    ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: content.to_string(),
        }],
        model: model.to_string(),
        ..Default::default()
    }
}
```

## Configuration Best Practices

### 1. Keep API Keys Secure

```rust
// ❌ Don't hardcode API keys
let config = ProviderConfig {
    api_key: Some("sk-hardcoded-key".to_string()),
    ..Default::default()
};

// ✅ Use environment variables or config files
let config = ProviderConfig {
    api_key: std::env::var("OPENAI_API_KEY").ok(),
    ..Default::default()
};
```

### 2. Use Default Values

```rust
use lc_cli::{Config, ProviderConfig};

let config = Config {
    providers: {
        let mut providers = std::collections::HashMap::new();
        providers.insert("openai".to_string(), ProviderConfig {
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            model: Some("gpt-4".to_string()),
            // Use defaults for other fields
            ..Default::default()
        });
        providers
    },
    default_provider: Some("openai".to_string()),
    ..Default::default()
};
```

### 3. Handle Missing Configuration Gracefully

```rust
use lc_cli::{Config, error::LcError};

async fn robust_config_loading() -> Result<Config, LcError> {
    match Config::load() {
        Ok(config) => {
            // Validate that we have at least one provider configured
            if config.providers.is_empty() {
                return Err(LcError::Config("No providers configured".to_string()));
            }
            Ok(config)
        },
        Err(_) => {
            // Create a minimal default configuration
            println!("No config found, using environment variables");
            let mut providers = std::collections::HashMap::new();
            
            if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                providers.insert("openai".to_string(), ProviderConfig {
                    api_key: Some(api_key),
                    model: Some("gpt-4".to_string()),
                    ..Default::default()
                });
            }
            
            if providers.is_empty() {
                return Err(LcError::Config("No API keys found in environment".to_string()));
            }
            
            Ok(Config {
                providers,
                default_provider: Some("openai".to_string()),
                ..Default::default()
            })
        }
    }
}
```

## Next Steps

- [**Advanced Features**](advanced-features.md) - Vector databases, sessions, and templates
- [**Basic Usage**](basic-usage.md) - Go back to basic examples

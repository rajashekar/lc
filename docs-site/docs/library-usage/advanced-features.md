# Advanced Features

Explore advanced LC library features including vector databases, session management, and template processing.

## Vector Database Operations

LC includes built-in vector database functionality for embeddings and semantic search.

### Basic Vector Operations

```rust
use lc_cli::{VectorDB, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let vector_db = VectorDB::new(&config).await?;
    
    // Store a document with embedding
    let document = "Rust is a systems programming language focused on safety and performance.";
    let doc_id = vector_db.store_document("rust_intro", document).await?;
    
    // Search for similar documents
    let query = "What is Rust programming language?";
    let results = vector_db.search(query, 5).await?;
    
    for result in results {
        println!("Score: {:.3} - {}", result.score, result.content);
    }
    
    Ok(())
}
```

### Batch Operations

```rust
use lc_cli::VectorDB;

async fn batch_store_documents() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let vector_db = VectorDB::new(&config).await?;
    
    let documents = vec![
        ("rust_memory", "Rust uses ownership system for memory management"),
        ("rust_concurrency", "Rust provides safe concurrency with async/await"),
        ("rust_performance", "Rust offers zero-cost abstractions for performance"),
    ];
    
    for (id, content) in documents {
        vector_db.store_document(id, content).await?;
    }
    
    println!("Stored {} documents", documents.len());
    Ok(())
}
```

## Session Management

Maintain conversation history and context across multiple interactions.

### Creating and Managing Sessions

```rust
use lc_cli::{Database, ChatMessage};

async fn session_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new()?;
    
    // Create a new session
    let session_id = db.create_session("My Rust Learning Session")?;
    
    // Add messages to the session
    db.add_chat_entry(&session_id, 
        "What is ownership in Rust?", 
        "Ownership is Rust's unique approach to memory management...",
        "gpt-4"
    )?;
    
    // Retrieve conversation history
    let history = db.get_chat_history(&session_id)?;
    
    for entry in history {
        println!("Q: {}", entry.question);
        println!("A: {}", entry.response);
        println!("---");
    }
    
    Ok(())
}
```

### Continuing Conversations

```rust
use lc_cli::{Database, OpenAIClient, ChatRequest, Message};

async fn continue_conversation(session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let client = OpenAIClient::new(&config)?;
    let db = Database::new()?;
    
    // Get conversation history
    let history = db.get_chat_history(session_id)?;
    
    // Convert to messages format
    let mut messages = Vec::new();
    for entry in history {
        messages.push(Message {
            role: "user".to_string(),
            content: entry.question,
        });
        messages.push(Message {
            role: "assistant".to_string(),
            content: entry.response,
        });
    }
    
    // Add new user message
    messages.push(Message {
        role: "user".to_string(),
        content: "Can you give me a code example?".to_string(),
    });
    
    let request = ChatRequest {
        messages,
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    
    let response = client.send_chat_request(request).await?;
    
    // Save the new exchange
    db.add_chat_entry(session_id, 
        "Can you give me a code example?", 
        &response.content,
        "gpt-4"
    )?;
    
    println!("Response: {}", response.content);
    Ok(())
}
```

## Template Processing

Use dynamic templates for consistent prompt formatting.

### Basic Template Usage

```rust
use lc_cli::template_processor::TemplateProcessor;
use std::collections::HashMap;

async fn template_example() -> Result<(), Box<dyn std::error::Error>> {
    let template = r#"
You are a {{role}} assistant specializing in {{domain}}.

User Question: {{question}}

Please provide a {{response_type}} response with:
- Clear explanations
- {{#if include_examples}}Code examples{{/if}}
- Best practices

{{#if context}}
Context: {{context}}
{{/if}}
"#;

    let mut variables = HashMap::new();
    variables.insert("role".to_string(), "helpful".to_string());
    variables.insert("domain".to_string(), "Rust programming".to_string());
    variables.insert("question".to_string(), "How do I handle errors in Rust?".to_string());
    variables.insert("response_type".to_string(), "detailed".to_string());
    variables.insert("include_examples".to_string(), "true".to_string());
    variables.insert("context".to_string(), "Beginner level".to_string());
    
    let processor = TemplateProcessor::new();
    let rendered = processor.render(template, &variables)?;
    
    println!("Rendered template:\n{}", rendered);
    Ok(())
}
```

### Template with File Loading

```rust
use lc_cli::template_processor::TemplateProcessor;
use std::collections::HashMap;

async fn load_template_from_file() -> Result<(), Box<dyn std::error::Error>> {
    let processor = TemplateProcessor::new();
    
    // Load template from file
    let template_content = std::fs::read_to_string("templates/code_review.txt")?;
    
    let mut variables = HashMap::new();
    variables.insert("code".to_string(), "fn main() { println!(\"Hello\"); }".to_string());
    variables.insert("language".to_string(), "Rust".to_string());
    variables.insert("focus".to_string(), "performance and safety".to_string());
    
    let rendered = processor.render(&template_content, &variables)?;
    
    // Use rendered template in chat request
    let config = Config::load()?;
    let client = OpenAIClient::new(&config)?;
    
    let request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: rendered,
        }],
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    
    let response = client.send_chat_request(request).await?;
    println!("Code review: {}", response.content);
    
    Ok(())
}
```

## File Processing

Handle different file types and extract content for LLM processing.

### Text File Processing

```rust
use lc_cli::readers::TextReader;

async fn process_text_files() -> Result<(), Box<dyn std::error::Error>> {
    let reader = TextReader::new();
    
    let files = vec!["src/main.rs", "README.md", "Cargo.toml"];
    
    for file_path in files {
        match reader.read_file(file_path).await {
            Ok(content) => {
                println!("File: {}", file_path);
                println!("Content length: {} characters", content.len());
                
                // Process with LLM
                let config = Config::load()?;
                let client = OpenAIClient::new(&config)?;
                
                let request = ChatRequest {
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: format!("Summarize this {} file:\n\n{}", file_path, content),
                    }],
                    model: "gpt-4".to_string(),
                    max_tokens: Some(200),
                    ..Default::default()
                };
                
                let response = client.send_chat_request(request).await?;
                println!("Summary: {}", response.content);
                println!("---");
            },
            Err(e) => eprintln!("Failed to read {}: {}", file_path, e),
        }
    }
    
    Ok(())
}
```

## Combining Features

Here's an example that combines multiple advanced features:

```rust
use lc_cli::{Config, OpenAIClient, VectorDB, Database, template_processor::TemplateProcessor};
use std::collections::HashMap;

async fn advanced_rag_system() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let client = OpenAIClient::new(&config)?;
    let vector_db = VectorDB::new(&config).await?;
    let db = Database::new()?;
    let template_processor = TemplateProcessor::new();
    
    // 1. Store knowledge in vector database
    let documents = vec![
        "Rust ownership prevents memory leaks and data races",
        "Rust borrowing allows multiple readers or one writer",
        "Rust lifetimes ensure references are valid",
    ];
    
    for (i, doc) in documents.iter().enumerate() {
        vector_db.store_document(&format!("rust_concept_{}", i), doc).await?;
    }
    
    // 2. User asks a question
    let user_question = "How does Rust prevent memory issues?";
    
    // 3. Search for relevant context
    let search_results = vector_db.search(user_question, 3).await?;
    let context = search_results.iter()
        .map(|r| r.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    
    // 4. Use template to format prompt
    let template = r#"
You are a Rust expert. Answer the user's question using the provided context.

Context:
{{context}}

Question: {{question}}

Provide a comprehensive answer with examples if relevant.
"#;
    
    let mut variables = HashMap::new();
    variables.insert("context".to_string(), context);
    variables.insert("question".to_string(), user_question.to_string());
    
    let prompt = template_processor.render(template, &variables)?;
    
    // 5. Send to LLM
    let request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    
    let response = client.send_chat_request(request).await?;
    
    // 6. Save to session
    let session_id = db.create_session("RAG Session")?;
    db.add_chat_entry(&session_id, user_question, &response.content, "gpt-4")?;
    
    println!("Answer: {}", response.content);
    
    Ok(())
}
```

## Performance Optimization

### Connection Pooling

```rust
use lc_cli::{Config, OpenAIClient};
use std::sync::Arc;

struct LLMService {
    client: Arc<OpenAIClient>,
}

impl LLMService {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Arc::new(OpenAIClient::new(config)?);
        Ok(Self { client })
    }
    
    pub async fn process_batch(&self, prompts: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut tasks = Vec::new();
        
        for prompt in prompts {
            let client = Arc::clone(&self.client);
            let task = tokio::spawn(async move {
                let request = ChatRequest {
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: prompt,
                    }],
                    model: "gpt-4".to_string(),
                    ..Default::default()
                };
                
                client.send_chat_request(request).await
            });
            tasks.push(task);
        }
        
        let mut results = Vec::new();
        for task in tasks {
            let response = task.await??;
            results.push(response.content);
        }
        
        Ok(results)
    }
}
```

## Error Handling Patterns

```rust
use lc_cli::{Config, OpenAIClient, error::LcError};

#[derive(Debug)]
enum AppError {
    Config(String),
    Network(String),
    LLM(String),
}

impl From<LcError> for AppError {
    fn from(err: LcError) -> Self {
        match err {
            LcError::Config(msg) => AppError::Config(msg),
            LcError::Network(msg) => AppError::Network(msg),
            _ => AppError::LLM(err.to_string()),
        }
    }
}

async fn robust_llm_call(prompt: &str) -> Result<String, AppError> {
    let config = Config::load()
        .map_err(|e| AppError::Config(format!("Failed to load config: {}", e)))?;
    
    let client = OpenAIClient::new(&config)?;
    
    let request = ChatRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        model: "gpt-4".to_string(),
        ..Default::default()
    };
    
    match client.send_chat_request(request).await {
        Ok(response) => Ok(response.content),
        Err(e) => {
            // Retry logic could go here
            Err(AppError::LLM(format!("LLM request failed: {}", e)))
        }
    }
}
```

## Next Steps

- Check out the [CLI documentation](../commands/overview.md) to understand all available features
- Explore the [provider-specific guides](../providers/openai.md) for detailed configuration
- Review the [troubleshooting guide](../troubleshooting.md) for common issues

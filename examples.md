# lc Usage Examples

This document provides comprehensive examples of using the `lc` CLI tool.

## Setup Examples

### Adding OpenAI Provider
```bash
# Add OpenAI provider
lc providers add -n openai -e https://api.openai.com/v1

# Set API key
lc keys add openai
# Enter your OpenAI API key when prompted: sk-...

# List available models
lc providers models -n openai
```

### Adding Vercel v0.dev Provider
```bash
# Add Vercel provider
lc providers add -n vercel -e https://api.v0.dev/v1

# Set API key (from your Vercel account)
lc keys add vercel
# Enter: v1:your_token_here

# Test with v0 model
lc -m v0-1.0-md "Create a simple React button component"
```

### Adding Local Provider (Ollama)
```bash
# Add local Ollama instance
lc providers add -n local -e http://localhost:11434/v1

# Set dummy API key (Ollama doesn't require real keys)
lc keys add local
# Enter: dummy

# Use local model
lc -m llama2 "Explain machine learning in simple terms"
```

## Chat Examples

### Basic Chat
```bash
# Simple question
lc -m gpt-3.5-turbo "What is the capital of France?"

# Longer prompt
lc -m gpt-4 "Explain the difference between REST and GraphQL APIs, including pros and cons of each"

# Code generation
lc -m gpt-4 "Write a Python function to calculate the Fibonacci sequence"
```

### Session Management
```bash
# Start a new conversation
lc -m gpt-4 "I'm learning Rust. Can you explain ownership?"

# Continue the conversation
lc -c -m gpt-4 "Can you give me a practical example?"

# Continue further
lc -c -m gpt-4 "What about borrowing and lifetimes?"

# Start a completely new session
lc -m gpt-4 "Now let's talk about something else entirely"
```

### Using Specific Chat IDs
```bash
# Get current session ID
lc -c

# Use a specific chat ID to continue an old conversation
lc --cid a1b2c3d4 -m gpt-4 "Let's continue our previous discussion about databases"

# Start a new session with a specific ID
lc --cid new-project-chat -m gpt-4 "I'm starting a new web project. What framework should I use?"
```

## History and Logging Examples

### Viewing Chat History
```bash
# Show the last response
lc -r

# Show current session details
lc -c

# Show all chat logs (detailed view)
lc logs show

# Show logs in minimal table format
lc logs show --minimal
```

### Example Log Output
```bash
$ lc logs show --minimal
┌──────────┬─────────────────┬──────────────────────────────────────────────────┬──────────────┐
│ Chat ID  │ Model           │ Question                                         │ Time         │
├──────────┼─────────────────┼──────────────────────────────────────────────────┼──────────────┤
│ a1b2c3d4 │ gpt-4           │ What is the capital of France?                   │ 01-15 14:30  │
│ a1b2c3d4 │ gpt-4           │ What about Germany?                              │ 01-15 14:31  │
│ e5f6g7h8 │ gpt-3.5-turbo   │ Explain machine learning                         │ 01-15 14:25  │
└──────────┴─────────────────┴──────────────────────────────────────────────────┴──────────────┘
```

## Advanced Usage Examples

### Working with Different Providers
```bash
# Use OpenAI for general questions
lc -m gpt-4 "What's the weather like today?"

# Use Vercel v0 for code generation
lc -m v0-1.0-md "Create a responsive navbar component"

# Use local model for privacy-sensitive tasks
lc -m llama2 "Help me write a personal email"
```

### Scripting with lc
```bash
#!/bin/bash

# Script to generate code documentation
MODEL="gpt-4"
FILE="$1"

if [ -z "$FILE" ]; then
    echo "Usage: $0 <source-file>"
    exit 1
fi

echo "Generating documentation for $FILE..."

# Read file content and ask for documentation
CONTENT=$(cat "$FILE")
PROMPT="Please generate comprehensive documentation for this code:\n\n$CONTENT"

lc -m "$MODEL" "$PROMPT" > "${FILE}.md"
echo "Documentation saved to ${FILE}.md"
```

### Batch Processing
```bash
# Process multiple questions in sequence
questions=(
    "What is Rust?"
    "How does memory management work in Rust?"
    "What are the main benefits of using Rust?"
)

for question in "${questions[@]}"; do
    echo "Q: $question"
    lc -m gpt-3.5-turbo "$question"
    echo "---"
done
```

## Configuration Examples

### Multiple Provider Setup
```bash
# Set up multiple providers for different use cases
lc providers add -n openai -e https://api.openai.com/v1
lc providers add -n anthropic -e https://api.anthropic.com/v1
lc providers add -n local -e http://localhost:11434/v1

# Set keys for each
lc keys add openai
lc keys add anthropic
lc keys add local

# Use different providers for different tasks
lc -m gpt-4 "Creative writing task"
lc -m claude-3 "Analysis task"
lc -m llama2 "Local processing task"
```

### Custom Endpoint Examples
```bash
# Azure OpenAI
lc providers add -n azure -e https://your-resource.openai.azure.com/openai/deployments/your-deployment

# OpenRouter
lc providers add -n openrouter -e https://openrouter.ai/api/v1

# Hugging Face Inference API
lc providers add -n huggingface -e https://api-inference.huggingface.co/models/your-model/v1
```

## Troubleshooting Examples

### Common Error Scenarios
```bash
# Error: No providers configured
$ lc -m gpt-4 "Hello"
Error: No providers configured. Add one with 'lc providers add'

# Solution:
lc providers add -n openai -e https://api.openai.com/v1
lc keys add openai

# Error: Model not found
$ lc -m nonexistent-model "Hello"
Error: Model 'nonexistent-model' not found

# Solution: List available models
lc providers models -n openai

# Error: API request failed
$ lc -m gpt-4 "Hello"
Error: API request failed with status 401: Invalid API key

# Solution: Reset API key
lc keys add openai
```

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug lc -m gpt-4 "Test message"

# This will show detailed HTTP requests, responses, and internal operations
```

## Performance Comparison Examples

### Startup Time Test
```bash
# Test Python llm tool
time llm -m gpt-3.5-turbo "Hello"
# Output: real 0m0.156s

# Test lc
time lc -m gpt-3.5-turbo "Hello"  
# Output: real 0m0.003s
```

### Memory Usage Test
```bash
# Monitor memory usage
/usr/bin/time -v lc -m gpt-4 "Explain quantum computing"
# Shows detailed memory statistics
```

This comprehensive set of examples should help users get started with `lc` and understand its full capabilities.
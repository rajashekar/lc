---
id: quick-start
title: Quick Start
sidebar_position: 2
---

# Quick Start

Get up and running with LLM Client in just a few minutes. This guide will walk you through the essential steps to start using `lc`.


## Step 1: Add a Provider

First, add an LLM provider. 

```bash
lc providers add <provider-name> <base-url>
```

We'll use OpenAI as an example:

```bash
lc providers add openai https://api.openai.com/v1
```

Or using the short alias:

```bash
lc p a openai https://api.openai.com/v1
```

### Other Popular Providers

```bash
# Anthropic Claude
lc providers add claude https://api.anthropic.com/v1 -c /messages

# OpenRouter (access to many models)
lc providers add openrouter https://openrouter.ai/api/v1

# Together AI (open source models)
lc providers add together https://api.together.xyz/v1

# Local Ollama
lc providers add ollama http://localhost:11434/v1
```

## Step 2: Set Your API Key

Add your API key for the provider:

```bash
lc keys add openai
# Enter your API key when prompted (input is hidden)
```

Or using the alias:

```bash
lc k a openai
```

## Step 3: Test Your Setup

Send your first prompt:

```bash
lc -p openai -m gpt-3.5-turbo "tell me a joke about AI"

# Or using the alias

lc -m openai:gpt-3.5-turbo "tell me a joke about AI"
```

## Step 4: Explore Models

List available models from your provider:

```bash
lc providers models openai
or
lc p m openai
```

Or use the models command for a richer view:

```bash
lc models
or 
lc m
```

## Step 5: Start Chatting

### Direct Prompts

Send one-off prompts with specific models:

```bash
# Use a specific model
lc -m openai:gpt-4 "Explain quantum computing in simple terms"

# Use a different provider and model
lc -p openrouter -m "anthropic/claude-3.5-sonnet" "Write a haiku about coding"
```

### Interactive Chat

Start an interactive chat session:

```bash
lc chat -m openai:gpt-4
```

In chat mode, you can:

- Type messages naturally
- Use `/exit` to quit
- Use `/clear` to start fresh
- Use `/model <name>` to switch models
- Use `/help` for more commands

## Setting Defaults (Optional)

Make your workflow faster by setting defaults:

```bash
# Set default provider
lc config set provider openai

# Set default model
lc config set model gpt-4

# Now you can just use:
lc "Your prompt here"
```

## Step 6: Add Search Integration (Optional)

Enhance your prompts with real-time web search results:

### Quick Setup with Free DuckDuckGo

```bash
# Add DuckDuckGo (no API key required!)
lc search provider add ddg https://api.duckduckgo.com/

# Test search
lc search query ddg "latest AI news" -f json

# Use in prompts
lc --use-search ddg "What are the latest developments in AI?"
```

### Setup with Premium Providers

```bash
# Add Brave Search (requires API key)
lc search provider add brave https://api.search.brave.com/res/v1/web/search
lc search provider set brave X-Subscription-Token YOUR_API_KEY

# Add Jina AI for advanced features (requires API key)
lc search provider add jina https://s.jina.ai/
lc search provider set jina Authorization YOUR_API_KEY

# Enable full content reading for research
lc search provider set jina X-Engine direct

# Use in prompts
lc --use-search brave "What's happening in tech today?"
lc --use-search jina "Comprehensive research on quantum computing"
```

## What's Next?

### Essential Features

- [Providers Command](/commands/providers) - Managing multiple LLM providers
- [Chat Command](/commands/chat) - Interactive conversations
- [FAQ](/faq) - View and manage chat history

### Advanced Features

- [Vector Database](/advanced/vector-database) - Store and search embeddings
- [RAG](/advanced/rag) - Enhance responses with your own data
- [Configuration Sync](/advanced/sync) - Sync settings across machines

### Quick Tips

1. **Use aliases for speed**: `lc p l` instead of `lc providers list`
2. **Pipe input**: `echo "translate: hello" | lc`
3. **Get last response**: `lc logs recent answer`
4. **Extract code**: `lc logs recent answer code`

## Common Commands Cheatsheet

```bash
# Providers
lc p l                    # List providers
lc p m openai            # List models from provider

# Direct chat
lc "prompt"              # Use defaults
lc -m openai:gpt-4 "prompt"     # Specify model

# Interactive
lc chat -m gpt-4         # Start chat session
lc c -m gpt-4           # Short alias

# Search integration
lc se p l                # List search providers
lc se q ddg "query"      # Direct search (free!)
lc --use-search ddg "prompt with search"  # Search + LLM

# Logs
lc l r                   # Recent logs
lc l r a                 # Last answer
lc l r a c               # Last answer's code blocks
```

Ready to explore more? Check out the [Command Reference](/commands/overview) for the complete list of available commands.

---
id: intro
title: Introduction
sidebar_position: 1
slug: /
---

<div align="center">
<h1>LLM Client (lc)</h1>
<img src="img/social-card.png" alt="LLM Client" width="450" />
</div>

A fast, Rust-based command-line tool for interacting with Large Language Models through OpenAI-compatible APIs. Built for speed, efficiency, and ease of use.

## Why LLM Client?

LLM Client was created to solve common pain points when working with LLMs:

- **⚡ Lightning Fast**: Near-zero cold start time compared to Python alternatives
- **🎯 Simple & Intuitive**: Clean CLI with short aliases for common operations
- **🔧 Universal Compatibility**: Works with any OpenAI-compatible API
- **💾 Built-in Intelligence**: Vector database and RAG support out of the box
- **🛠️ Extensible**: Model Context Protocol (MCP) support for tools and integrations
- **🔐 Secure by Design**: Encrypted configuration sync across machines

## Key Features

### Core Capabilities

- 🚀 **Fast startup** - ~3ms cold start vs ~150ms for Python alternatives
- 💬 **Direct prompts** - Send one-off prompts with simple commands
- 💾 **Session management** - Continue conversations with full history
- 📊 **SQLite logging** - All conversations stored locally
- 🔐 **Secure key storage** - API keys stored safely in user config

### Advanced Features

- 🧠 **Vector Database & RAG** - Built-in embeddings and similarity search
- 📚 **Smart File Processing** - Embed entire documents with intelligent chunking
- 🔍 **Web Search Integration** - 6 search providers including free DuckDuckGo and AI-powered Jina
- ☁️ **Configuration Sync** - Sync settings across machines with encryption
- 🤖 **Model Context Protocol** - Extend LLMs with tools for web access, file operations, and more

## Quick Example

```bash
# Add a provider
lc providers add openai https://api.openai.com/v1
or
lc p a openai https://api.openai.com/v1
or
lc providers install openai

# Set your API key
lc keys add openai
or
lc k a openai

# List available models for the provider
lc providers models openai
or
lc p m openai

# list all models
lc models
or 
lc m 

# to search for a specific model
lc models -q gpt-4

# set default provider and model
lc config set provider openai
lc config set model gpt-4

# Start chatting
lc "What is the capital of France?"

# Or use interactive mode
lc chat

# Or use a specific model
lc -m openai:gpt-4 "Explain quantum computing in simple terms"
or
lc -p openrouter -m "anthropic/claude-3.5-sonnet" "Write a haiku about coding"
```

## Next Steps

- [Installation Guide](/getting-started/installation) - Get lc up and running
- [Quick Start](/getting-started/quick-start) - Start using lc in minutes
- [Command Reference](/commands/overview) - Explore all available commands

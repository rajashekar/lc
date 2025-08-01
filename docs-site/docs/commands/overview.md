---
id: overview
title: Commands Overview
sidebar_position: 1
---

# Commands Overview

LLM Client provides a comprehensive set of commands organized by functionality. Most commands have short aliases for faster usage.

## Command Structure

```bash
lc [COMMAND] [SUBCOMMAND] [OPTIONS] [ARGS]
```

## Command Categories

### Core Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `lc "prompt"` | - | Send a direct prompt using defaults |
| `lc chat` | `lc c` | Start interactive chat session |
| `lc providers` | `lc p` | Manage LLM providers |
| `lc models` | `lc m` | List and filter available models |
| `lc keys` | `lc k` | Manage API keys |
| `lc config` | `lc co` | Configure defaults |
| `lc logs` | `lc l` | View and manage chat history |

### Advanced Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `lc embed` | `lc e` | Generate and store embeddings |
| `lc vectors` | `lc v` | Manage vector databases |
| `lc similar` | `lc s` | Search for similar content |
| `lc search` | `lc se` | Web search integration |
| `lc sync` | `lc sy` | Sync configuration to cloud |
| `lc mcp` | - | Manage MCP servers |
| `lc alias` | `lc a` | Manage model aliases |
| `lc templates` | `lc t` | Manage templates |
| `lc proxy` | `lc pr` | Run proxy server |
| `lc web-chat-proxy` | `lc w` | Web chat proxy |

## Direct Prompts

The simplest way to use lc:

```bash
# Using defaults
lc "What is the capital of France?"

# Specify provider
lc --provider openai "Explain recursion"

# Specify model
lc -m gpt-4 "Write a Python function"

# Specify both
lc --provider openrouter -m "claude-3.5-sonnet" "Explain quantum computing"

# With vector database context (RAG)
lc -v knowledge "What do you know about machine learning?"

# With MCP tools
lc -t fetch "What's the latest news about AI?"

# With web search
lc --use-search brave "What are the latest AI developments?"
```

## Global Options

These options work with most commands:

- `-p, --provider <PROVIDER>` - Specify provider
- `-m, --model <MODEL>` - Specify model
- `-s, --system <SYSTEM_PROMPT>` - Set system prompt
- `--max-tokens <MAX_TOKENS>` - Maximum number of tokens
- `--temperature <TEMPERATURE>` - Adjust response randomness
- `-a, --attach <ATTACHMENTS>` - Attach files
- `-t, --tools <TOOLS>` - Include MCP tools (comma-separated)
- `-v, --vectordb <VECTORDB>` - Use vector database for context
- `-d, --debug` - Enable debug mode
- `-c, --continue` - Continue previous session
- `--cid <CHAT_ID>` - Specify chat ID
- `--use-search <SEARCH>` - Use search results as context
- `-h, --help` - Show help information
- `-V, --version` - Show version

## Command Aliases

LLM Client uses intuitive aliases to speed up your workflow:

### Single Letter Aliases

- `c` → `chat`
- `p` → `providers`
- `m` → `models`
- `k` → `keys`
- `l` → `logs`
- `e` → `embed`
- `v` → `vectors`
- `s` → `similar`

### Two Letter Aliases

- `co` → `config`
- `sy` → `sync`
- `se` → `search`
- `pr` → `proxy`

### Subcommand Aliases

- `a` → `add`
- `r` → `remove` or `recent` or `refresh`
- `l` → `list`
- `s` → `show` or `setup` or `stats`
- `u` → `update`
- `d` → `delete` or `dump`
- `i` → `info`

## Examples

### Quick Provider Setup

```bash
# Long form
lc providers add openai https://api.openai.com/v1
lc keys add openai
lc providers models openai

# Short form (same result)
lc p a openai https://api.openai.com/v1
lc k a openai
lc p m openai
```

### Chat Workflow

```bash
# Start chat
lc c -m gpt-4

# View recent chats
lc l r

# Get last answer
lc l r a

# Extract code from last answer
lc l r a c
```

### Vector Database Workflow

```bash
# Create embeddings
lc e -m text-embedding-3-small -v docs "Important information"

# Search similar content
lc s -v docs "related query"

# Use in chat
lc c -v docs -m gpt-4
```

### MCP Tools Workflow

```bash
# Add MCP server
lc mcp add fetch "uvx mcp-server-fetch" --type stdio

# List functions
lc mcp functions fetch

# Use in prompt
lc -t fetch "Get current weather in Tokyo"

# Use in chat
lc c -m gpt-4 -t fetch
```

### Search Integration Workflow

```bash
# Add search provider
lc search provider add brave https://api.search.brave.com/res/v1/web/search
lc search provider set brave X-Subscription-Token YOUR_API_KEY

# Direct search
lc search query brave "latest AI news" -f json

# Use in prompts
lc --use-search brave "What's happening in AI today?"
```

## Getting Help

Every command has built-in help:

```bash
# General help
lc --help

# Command help
lc providers --help
lc p --help

# Subcommand help
lc providers add --help
lc p a --help
```

## Next Steps

Explore specific command documentation:

- [Providers Commands](/commands/providers)
- [Models Commands](/commands/models)
- [MCP Commands](/commands/mcp)
- [Chat Commands](/commands/chat)
- [Embedding Commands](/commands/embed)
- [Vector Commands](/commands/vectors)
- [Search Commands](/commands/search)

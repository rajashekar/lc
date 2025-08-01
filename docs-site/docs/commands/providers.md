---
id: providers
title: Provider Commands
sidebar_position: 2
---

# Provider Commands

Manage LLM providers and their configurations. Providers are API endpoints that serve language models.

## Command: `lc providers` (alias: `lc p`)

### Subcommands

#### Add Provider

Add a new provider to your configuration.

```bash
lc providers add <name> <endpoint> [OPTIONS]
lc p a <name> <endpoint> [OPTIONS]
```

**Options:**

- `-m, --models-path <PATH>` - Custom models endpoint (default: `/models`)
- `-c, --chat-path <PATH>` - Custom chat endpoint (default: `/chat/completions`)

**Examples:**

```bash
# Standard OpenAI-compatible provider
lc providers add openai https://api.openai.com/v1

# Provider with custom endpoints
lc providers add github https://models.github.ai \
  --models-path /catalog/models \
  --chat-path /inference/chat/completions

# Short form
lc p a together https://api.together.xyz/v1
```

#### List Providers

Show all configured providers.

```bash
lc providers list
lc p l
```

**Output example:**

```
Configured providers:
  • openai (https://api.openai.com/v1) [key set]
  • claude (https://api.anthropic.com/v1) [key set]
  • together (https://api.together.xyz/v1) [no key]
```

#### List Models

Show available models from a specific provider.

```bash
lc providers models <provider>
lc p m <provider>
```

**Example:**

```bash
lc providers models openai
# Output:
# Available models for openai:
#   • gpt-4-turbo-preview
#   • gpt-4
#   • gpt-3.5-turbo
#   • text-embedding-3-small
#   • text-embedding-3-large
```

#### Update Provider

Update a provider's endpoint URL.

```bash
lc providers update <name> <endpoint>
lc p u <name> <endpoint>
```

**Example:**

```bash
lc providers update openai https://api.openai.com/v1
```

#### Manage Headers

Add, list, or delete custom headers for providers.

**Add Header**

```bash
lc providers headers <provider> add <header> <value>
lc p h <provider> a <header> <value>
```

**List Headers**

```bash
lc providers headers <provider> list
lc p h <provider> l
```

**Delete Header**

```bash
lc providers headers <provider> delete <header>
lc p h <provider> d <header>
```

#### Set Token URL

Configure token URLs for providers requiring different endpoints for token handling.

```bash
lc providers token-url <provider> <url>
lc p t <provider> <url>
```

#### Remove Provider

Remove a provider from your configuration.

```bash
lc providers remove <name>
lc p r <name>
```

**Example:**

```bash
lc providers remove old-provider
```

**Example: Token URL Setup**

```bash
# Set a custom token URL for a provider
lc providers token-url custom-provider https://api.custom.com/auth/token
```

### Custom Headers

Some providers require additional headers beyond the standard Authorization header.

**Example: Anthropic Claude Setup**

```bash
# Add Claude provider
lc providers add claude https://api.anthropic.com/v1 -c /messages

# Add required headers
lc providers headers claude add x-api-key sk-ant-api03-...
lc providers headers claude add anthropic-version 2023-06-01

# Verify headers
lc providers headers claude list
```

## Common Provider Configurations

### OpenAI

```bash
lc providers add openai https://api.openai.com/v1
lc keys add openai
```

### Anthropic Claude

```bash
lc providers add claude https://api.anthropic.com/v1 -c /messages
lc providers headers claude add x-api-key <your-key>
lc providers headers claude add anthropic-version 2023-06-01
```

### OpenRouter

```bash
lc providers add openrouter https://openrouter.ai/api/v1
lc keys add openrouter
```

### Together AI

```bash
lc providers add together https://api.together.xyz/v1
lc keys add together
```

### GitHub Models

```bash
lc providers add github https://models.github.ai \
  -m /catalog/models \
  -c /inference/chat/completions
lc keys add github
```

### Local Ollama

```bash
lc providers add ollama http://localhost:11434/v1
# No API key needed for local providers
```

### Hugging Face Router

```bash
lc providers add hf https://router.huggingface.co/v1
lc keys add hf
```

## Provider Features

### Custom Endpoints

Some providers use non-standard paths for their endpoints:

- **Models Path**: Where to fetch available models (default: `/models`)
- **Chat Path**: Where to send chat requests (default: `/chat/completions`)

### Response Format Support

LLM Client automatically detects and handles multiple response formats:

1. **OpenAI Format** (most providers)
2. **Llama API Format** (Meta)
3. **Cohere Format**
4. **Anthropic Format** (Claude)

### Special Provider: Hugging Face Router

The HF router expands models with their available providers:

```bash
lc providers models hf
# Output shows:
# • Qwen/Qwen3-32B:groq
# • Qwen/Qwen3-32B:hyperbolic
# • meta-llama/Llama-3.3-70B-Instruct:together
```

Use the full `model:provider` format when prompting:

```bash
lc --provider hf -m "Qwen/Qwen3-32B:groq" "Hello"
```

## Troubleshooting

### "Provider not found"

- Check spelling: `lc providers list`
- Ensure provider is added: `lc providers add <name> <url>`

### "Invalid endpoint"

- Verify URL includes protocol: `https://` or `http://`
- Check if custom paths are needed: `-m` and `-c` flags

### "Authentication failed"

- Verify API key: `lc keys add <provider>`
- Check if custom headers are needed
- Some providers use `x-api-key` instead of `Authorization`

## See Also

- [Keys Command](keys.md)
- [Models Command](models.md)
- [Chat Command](chat.md)

## Next Steps

- [API Key Management](/commands/keys) - Secure key storage
- [Models Command](/commands/models) - Advanced model filtering
- [Provider Examples](/providers/openai) - Detailed provider guides

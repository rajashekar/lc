---
sidebar_position: 3
---

# Provider Commands

Manage LLM provider configurations and installations.

## Overview

The `providers` command (alias: `p`) allows you to install, update, and manage provider configurations from a centralized registry. Provider configurations are stored separately from API keys for security and shareability.

## Commands

### `lc providers install`

Install a provider from the registry.

**Aliases:** `lc p i`

**Usage:**
```bash
lc providers install <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Name of the provider to install

**Options:**
- `-f, --force` - Force reinstall even if already installed

**Examples:**
```bash
# Install OpenAI provider
lc providers install openai

# Force reinstall Anthropic provider
lc p i anthropic --force

# Install Google Gemini
lc p i gemini
```

### `lc providers upgrade`

Update installed providers to their latest versions.

**Aliases:** `lc p up`

**Usage:**
```bash
lc providers upgrade [name]
```

**Arguments:**
- `[name]` - Optional: specific provider to update (updates all if not specified)

**Examples:**
```bash
# Update all installed providers
lc providers upgrade

# Update only OpenAI provider
lc p up openai
```

### `lc providers uninstall`

Remove an installed provider configuration.

**Aliases:** `lc p un`

**Usage:**
```bash
lc providers uninstall <name>
```

**Arguments:**
- `<name>` - Name of the provider to uninstall

**Examples:**
```bash
# Uninstall a provider
lc providers uninstall openai

# Using short alias
lc p un anthropic
```

**Note:** This removes the provider configuration but preserves any API keys in `keys.toml`.

### `lc providers available`

List all providers available in the registry.

**Aliases:** `lc p av`

**Usage:**
```bash
lc providers available [OPTIONS]
```

**Options:**
- `--official` - Show only official providers
- `-t, --tag <tag>` - Filter by tag (e.g., chat, embeddings, vision)

**Examples:**
```bash
# List all available providers
lc providers available

# Show only official providers
lc p av --official

# Filter by chat capability
lc p av --tag chat

# Filter by embeddings support
lc p av --tag embeddings
```

### `lc providers list`

List all installed providers and their authentication status.

**Aliases:** `lc p l`

**Usage:**
```bash
lc providers list
```

**Examples:**
```bash
# List installed providers
lc providers list

# Using short alias
lc p l
```

**Output:**
```
Configured Providers:
  • openai - https://api.openai.com/v1 (API Key: ✓)
  • anthropic - https://api.anthropic.com/v1 (API Key: ✗)
  • gemini - https://generativelanguage.googleapis.com/v1beta (API Key: ✓)
```

### `lc providers add`

Manually add a custom provider (without using the registry).

**Aliases:** `lc p a`

**Usage:**
```bash
lc providers add <name> <url> [OPTIONS]
```

**Arguments:**
- `<name>` - Provider name
- `<url>` - Provider endpoint URL

**Options:**
- `-m, --models-path <path>` - Custom models endpoint path (default: /models)
- `-c, --chat-path <path>` - Custom chat completions endpoint path (default: /chat/completions)

**Examples:**
```bash
# Add a basic provider
lc providers add custom https://api.custom.com

# Add with custom paths
lc p a internal https://llm.internal.com \
  --models-path /v1/models \
  --chat-path /v1/chat
```

### `lc providers update`

Update an existing provider's endpoint URL.

**Aliases:** `lc p u`

**Usage:**
```bash
lc providers update <name> <url>
```

**Examples:**
```bash
# Update provider endpoint
lc providers update custom https://new-api.custom.com
```

### `lc providers remove`

Remove a manually added provider.

**Aliases:** `lc p r`

**Usage:**
```bash
lc providers remove <name>
```

**Examples:**
```bash
# Remove a provider
lc providers remove custom
```

### `lc providers models`

List available models for a provider.

**Aliases:** `lc p m`

**Usage:**
```bash
lc providers models <name> [OPTIONS]
```

**Arguments:**
- `<name>` - Provider name

**Options:**
- `-r, --refresh` - Refresh the models cache

**Examples:**
```bash
# List OpenAI models
lc providers models openai

# Refresh and list models
lc p m anthropic --refresh
```

### `lc providers headers`

Manage custom headers for a provider.

**Aliases:** `lc p h`

**Usage:**
```bash
lc providers headers <provider> <COMMAND>
```

**Subcommands:**

#### `add` - Add a custom header
```bash
lc providers headers <provider> add <name> <value>

# Example
lc p h custom add X-Custom-Auth "secret-token"
```

#### `delete` - Remove a custom header
```bash
lc providers headers <provider> delete <name>

# Example
lc p h custom delete X-Custom-Auth
```

#### `list` - List all custom headers
```bash
lc providers headers <provider> list

# Example
lc p h custom list
```

### `lc providers vars`

Manage provider variables for path templating.

**Aliases:** `lc p v`

**Usage:**
```bash
lc providers vars <provider> <COMMAND>
```

**Subcommands:**

#### `set` - Set a provider variable
```bash
lc providers vars <provider> set <key> <value>

# Example: Set project ID for Vertex AI
lc p v vertex_ai set project "my-project-id"
lc p v vertex_ai set location "us-central1"
```

#### `get` - Get a provider variable
```bash
lc providers vars <provider> get <key>

# Example
lc p v vertex_ai get project
```

#### `list` - List all provider variables
```bash
lc providers vars <provider> list

# Example
lc p v vertex_ai list
```

### `lc providers paths`

Manage API endpoint paths for a provider.

**Aliases:** `lc p path`

**Usage:**
```bash
lc providers paths <provider> <COMMAND>
```

**Subcommands:**

#### `add` - Add or update provider paths
```bash
lc providers paths <provider> add [OPTIONS]

# Options:
# -m, --models <path>     - Models endpoint path
# -c, --chat <path>       - Chat completions path
# -i, --images <path>     - Image generations path
# -e, --embeddings <path> - Embeddings path

# Example
lc p path openai add --images /v1/images/generations
```

#### `delete` - Reset provider paths to defaults
```bash
lc providers paths <provider> delete [OPTIONS]

# Options:
# -m, --models     - Reset models path
# -c, --chat       - Reset chat path
# -i, --images     - Reset images path
# -e, --embeddings - Reset embeddings path

# Example
lc p path openai delete --images
```

#### `list` - List all provider paths
```bash
lc providers paths <provider> list

# Example
lc p path openai list
```

## Provider Registry

The provider registry contains pre-configured providers that can be easily installed. The default registry includes:

### Official Providers
- **openai** - OpenAI's GPT models
- **anthropic** - Anthropic's Claude models
- **gemini** - Google's Gemini models

### Community Providers
- **groq** - High-speed inference
- **together** - Open-source models
- **ollama** - Local LLM inference
- **mistral** - Mistral AI models
- **cohere** - Cohere's language models
- **deepseek** - DeepSeek coding models
- **perplexity** - Models with web search
- **vertex_ai** - Google Cloud Vertex AI
- **amazon_bedrock** - AWS Bedrock

## Authentication

After installing a provider, you need to add its API key:

```bash
# Install provider
lc providers install openai

# Add API key
lc keys add openai
```

See [`lc keys`](./keys.md) for more information about key management.

## Custom Registry

You can use a custom provider registry by setting the environment variable:

```bash
# Use a remote registry
export LC_PROVIDER_REGISTRY_URL="https://your-domain.com/registry"

# Use a local registry
export LC_PROVIDER_REGISTRY_URL="file:///path/to/registry"

# Install from custom registry
lc providers install custom-provider
```

## File Locations

Provider configurations are stored in:
- **macOS**: `~/Library/Application Support/lc/providers/`
- **Linux**: `~/.local/share/lc/providers/`
- **Windows**: `%LOCALAPPDATA%\lc\providers\`

## Examples

### Complete Provider Setup

```bash
# 1. Browse available providers
lc providers available

# 2. Install a provider
lc providers install anthropic

# 3. Add API key
lc keys add anthropic

# 4. Verify installation
lc providers list

# 5. Test the provider
lc -p anthropic -m claude-3-opus "Hello!"
```

### Managing Multiple Providers

```bash
# Install multiple providers
lc p i openai
lc p i anthropic
lc p i gemini

# Add keys for each
lc k a openai
lc k a anthropic
lc k a gemini

# List all providers
lc p l

# Use different providers
lc -p openai "Query for GPT"
lc -p anthropic "Query for Claude"
lc -p gemini "Query for Gemini"
```

### Custom Provider Configuration

```bash
# Add custom provider
lc providers add internal https://llm.internal.com \
  --models-path /api/models \
  --chat-path /api/chat

# Add authentication header
lc providers headers internal add X-API-Key "internal-key"

# Set provider variables
lc providers vars internal set department "engineering"

# Test the provider
lc -p internal "Test query"
```

## See Also

- [`lc keys`](./keys.md) - Manage API keys
- [`lc config`](./config.md) - General configuration
- [Provider Management Guide](../advanced/provider-management.md) - Detailed provider management documentation

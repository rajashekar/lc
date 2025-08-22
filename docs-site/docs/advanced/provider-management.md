---
sidebar_position: 5
---

# Provider Management

The `lc` CLI includes a powerful provider management system that separates provider configurations from sensitive API keys, enabling secure sharing and version control of provider setups.

## Overview

The provider management system offers:
- **Centralized Provider Registry**: Browse and install pre-configured providers
- **Secure Key Management**: API keys stored separately in `keys.toml`
- **Version Control Friendly**: Share provider configs without exposing secrets
- **Automatic Updates**: Keep provider configurations up-to-date
- **Multiple Auth Types**: Support for API keys, OAuth, service accounts, and custom headers

## Installing Providers

### Browse Available Providers

View all available providers in the registry:

```bash
lc providers available
# or use the short alias
lc p av
```

Filter by official providers:
```bash
lc providers available --official
```

Filter by tags:
```bash
lc providers available --tag chat
```

### Install a Provider

Install a provider from the registry:

```bash
lc providers install openai
# or use the short alias
lc p i openai
```

Force reinstall (overwrites existing configuration):
```bash
lc providers install openai --force
```

### List Installed Providers

View all installed providers and their authentication status:

```bash
lc providers list
# or use the short alias
lc p l
```

Output example:
```
Configured Providers:
  • openai - https://api.openai.com/v1 (API Key: ✓)
  • anthropic - https://api.anthropic.com/v1 (API Key: ✗)
  • gemini - https://generativelanguage.googleapis.com/v1beta (API Key: ✗)
```

## Managing API Keys

API keys and other credentials are stored separately from provider configurations in `keys.toml`, located in your config directory.

### Add an API Key

```bash
lc keys add openai
# You'll be prompted to enter the API key securely
```

### List Providers with Keys

```bash
lc keys list
# or use the short alias
lc k l
```

### Remove an API Key

```bash
lc keys remove openai
# or use the short alias
lc k r openai
```

## Updating Providers

### Update a Specific Provider

```bash
lc providers upgrade openai
# or use the short alias
lc p up openai
```

### Update All Providers

```bash
lc providers upgrade
# or use the short alias
lc p up
```

## Uninstalling Providers

Remove an installed provider:

```bash
lc providers uninstall openai
# or use the short alias
lc p un openai
```

Note: This removes the provider configuration but preserves the API key in `keys.toml`.

## File Structure

Provider configurations are organized as follows:

```
~/Library/Application Support/lc/  # macOS
~/.local/share/lc/                 # Linux
%LOCALAPPDATA%/lc/                 # Windows
├── config.toml                    # Main configuration
├── keys.toml                      # API keys (secure, not for version control)
└── providers/                     # Provider configurations
    ├── openai.toml
    ├── anthropic.toml
    └── gemini.toml
```

## Authentication Types

The provider system supports various authentication methods:

### API Key
Most common authentication type:
```bash
lc keys add openai
```

### Service Account (Google Vertex AI)
For Google Cloud services:
```bash
lc keys add vertex_ai
# Paste base64-encoded service account JSON or provide file path
```

### OAuth Token
For OAuth-based providers:
```bash
lc keys add github
# Enter OAuth token
```

### Custom Headers
For providers requiring custom authentication headers:
```bash
lc providers headers <provider> add X-API-Key <value>
```

## Creating a Provider Registry

You can create your own provider registry for custom or internal providers:

### Registry Structure

```
provider-registry/
├── registry.json              # Registry metadata
└── providers/
    ├── custom-provider.toml
    └── internal-llm.toml
```

### Registry Format

`registry.json`:
```json
{
  "custom-provider": {
    "name": "Custom Provider",
    "version": "1.0.0",
    "description": "Internal LLM service",
    "official": false,
    "auth_type": "ApiKey",
    "tags": ["chat", "internal"]
  }
}
```

### Provider Configuration Format

`providers/custom-provider.toml`:
```toml
# Custom Provider Configuration
# Version: 1.0.0

endpoint = "https://api.internal.com"
models = []
models_path = "/v1/models"
chat_path = "/v1/chat/completions"

[headers]
Content-Type = "application/json"

[vars]
# Custom variables for path templating

[chat_templates.".*"]
# Request/response templates for chat endpoint
request = """
{
  "model": "{{ model }}",
  "messages": {{ messages | json }}
}
"""
```

### Using a Custom Registry

Set the registry URL environment variable:
```bash
export LC_PROVIDER_REGISTRY_URL="https://your-domain.com/provider-registry"
lc providers install custom-provider
```

Or use a local file path:
```bash
export LC_PROVIDER_REGISTRY_URL="file:///path/to/provider-registry"
lc providers install custom-provider
```

## Migration from Legacy Configuration

If you have existing provider configurations with embedded API keys, they will be automatically migrated to the new system:

1. API keys are extracted and moved to `keys.toml`
2. Provider configs are updated to remove sensitive data
3. Original functionality is preserved

The migration happens automatically when you run any `lc` command after updating.

## Security Best Practices

1. **Never commit `keys.toml`** to version control
2. **Add `keys.toml` to `.gitignore`** if sharing configurations
3. **Use environment variables** for CI/CD:
   ```bash
   export LC_OPENAI_API_KEY="sk-..."
   ```
4. **Set restrictive permissions** on `keys.toml` (automatically done on Unix systems)

## Troubleshooting

### Provider Not Found
```bash
Error: Provider 'xyz' not found in registry
```
Solution: Check available providers with `lc providers available`

### Authentication Failed
```bash
Error: No API key configured for provider 'openai'
```
Solution: Add the API key with `lc keys add openai`

### Version Conflicts
```bash
Warning: Provider 'openai' has updates available (v1.0.0 -> v1.1.0)
```
Solution: Update with `lc providers upgrade openai`

### Custom Registry Issues
```bash
Error: Failed to fetch registry from URL
```
Solution: Verify the registry URL and ensure it's accessible

## Examples

### Complete Setup Flow

```bash
# 1. Install a provider
lc providers install openai

# 2. Add API key
lc keys add openai
# Enter your API key when prompted

# 3. Verify installation
lc providers list

# 4. Test the provider
lc -m gpt-4 "Hello, world!"
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

# List all configured providers
lc p l

# Use different providers
lc -p openai -m gpt-4 "OpenAI query"
lc -p anthropic -m claude-3-opus "Anthropic query"
lc -p gemini -m gemini-pro "Google query"
```

### Sharing Configurations

```bash
# Export provider configs (without keys)
cp -r ~/Library/Application\ Support/lc/providers ./shared-configs/

# Import on another machine
cp -r ./shared-configs/* ~/Library/Application\ Support/lc/providers/

# Add keys on the new machine
lc keys add openai
lc keys add anthropic
```

## Related Commands

- [`lc providers`](../commands/providers.md) - Provider management commands
- [`lc keys`](../commands/keys.md) - API key management commands
- [`lc config`](../commands/config.md) - General configuration management
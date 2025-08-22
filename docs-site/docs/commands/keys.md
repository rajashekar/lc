---
sidebar_position: 4
---

# Keys Commands

Manage API keys and authentication credentials for providers.

## Overview

The `keys` command (alias: `k`) manages authentication credentials separately from provider configurations. This separation allows you to share provider configurations without exposing sensitive API keys.

All keys are stored in `keys.toml` in your configuration directory with restrictive permissions (0600 on Unix systems).

## Commands

### `lc keys add`

Add an API key or authentication credential for a provider.

**Aliases:** `lc k a`

**Usage:**
```bash
lc keys add <name>
```

**Arguments:**
- `<name>` - Provider name

**Examples:**
```bash
# Add API key for OpenAI
lc keys add openai
# Enter API key: [hidden input]

# Using short alias
lc k a anthropic
```

**Special Cases:**

#### Google Vertex AI / Service Accounts
For providers using Google service accounts:
```bash
lc keys add vertex_ai
# Options:
# 1. Paste base64-encoded service account JSON
# 2. Provide path to service account JSON file

# Example with base64:
cat service-account.json | base64 | lc keys add vertex_ai

# Example with file path:
lc keys add vertex_ai
# Enter: /path/to/service-account.json
```

### `lc keys list`

List all providers that have configured API keys.

**Aliases:** `lc k l`

**Usage:**
```bash
lc keys list
```

**Examples:**
```bash
# List providers with keys
lc keys list

# Using short alias
lc k l
```

**Output:**
```
API Key Status:
  • openai - ✓ Configured
  • anthropic - ✓ Configured
  • gemini - ✗ Missing
```

### `lc keys get`

Display the API key for a specific provider.

**Aliases:** `lc k g`

**Usage:**
```bash
lc keys get <name>
```

**Arguments:**
- `<name>` - Provider name

**Examples:**
```bash
# Get OpenAI API key
lc keys get openai

# Using short alias
lc k g anthropic
```

**Note:** This displays the actual API key in plain text. Use with caution.

### `lc keys remove`

Remove the API key for a provider.

**Aliases:** `lc k r`

**Usage:**
```bash
lc keys remove <name>
```

**Arguments:**
- `<name>` - Provider name

**Examples:**
```bash
# Remove API key
lc keys remove openai

# Using short alias
lc k r anthropic
```

## Key Storage

### File Location

Keys are stored in `keys.toml`:
- **macOS**: `~/Library/Application Support/lc/keys.toml`
- **Linux**: `~/.local/share/lc/keys.toml`
- **Windows**: `%LOCALAPPDATA%\lc\keys.toml`

### File Format

```toml
[api_keys]
openai = "sk-..."
anthropic = "sk-ant-..."
gemini = "AIza..."

[service_accounts]
vertex_ai = "{\"type\":\"service_account\",\"project_id\":\"...\"}"

[oauth_tokens]
github = "ghp_..."

[tokens]
custom_provider = "token-..."

[custom_headers]
[custom_headers.internal_llm]
X-API-Key = "internal-key"
X-Department = "engineering"
```

### Security

- File permissions are automatically set to 0600 (owner read/write only) on Unix systems
- Keys are never included in provider configuration files
- Keys are loaded at runtime when needed
- Never commit `keys.toml` to version control

## Authentication Types

### API Keys

Most common authentication type:
```bash
lc keys add openai
# Enter API key: sk-...
```

### Service Accounts

For Google Cloud services:
```bash
lc keys add vertex_ai
# Paste base64 service account JSON or file path
```

### OAuth Tokens

For OAuth-based authentication:
```bash
lc keys add github
# Enter OAuth token: ghp_...
```

### Custom Headers

Some providers require custom authentication headers:
```bash
# Add custom headers via provider commands
lc providers headers custom add X-API-Key "secret"
```

## Environment Variables

You can also provide API keys via environment variables:

```bash
# Format: LC_<PROVIDER>_API_KEY
export LC_OPENAI_API_KEY="sk-..."
export LC_ANTHROPIC_API_KEY="sk-ant-..."

# Service accounts (base64 encoded)
export LC_VERTEX_AI_SERVICE_ACCOUNT="eyJ0eXBlIjoi..."
```

Environment variables take precedence over keys.toml.

## Migration

If you have existing provider configurations with embedded API keys, they will be automatically migrated to `keys.toml`:

1. Run any `lc` command after updating
2. API keys are extracted from provider configs
3. Keys are moved to `keys.toml`
4. Provider configs are updated to remove sensitive data

## Examples

### Basic Setup

```bash
# 1. Install a provider
lc providers install openai

# 2. Add API key
lc keys add openai
# Enter your API key

# 3. Verify key is configured
lc keys list

# 4. Test the provider
lc -m gpt-4 "Hello!"
```

### Multiple Providers

```bash
# Add keys for multiple providers
lc keys add openai
lc keys add anthropic
lc keys add gemini

# List all configured keys
lc keys list

# Remove a key
lc keys remove gemini
```

### Service Account Setup

```bash
# Install Vertex AI provider
lc providers install vertex_ai

# Add service account (from file)
lc keys add vertex_ai
# Enter: /path/to/service-account.json

# Or use base64
cat sa.json | base64 | pbcopy  # Copy to clipboard
lc keys add vertex_ai
# Paste the base64 content

# Verify
lc keys list
```

### Using Environment Variables

```bash
# Set environment variable
export LC_OPENAI_API_KEY="sk-..."

# Use without adding to keys.toml
lc -p openai -m gpt-4 "Test"

# Environment variable takes precedence
lc keys add openai  # Add different key
# The environment variable will still be used
```

## Security Best Practices

1. **Never share `keys.toml`** - Keep it local and secure
2. **Add to .gitignore** - Prevent accidental commits:
   ```gitignore
   keys.toml
   **/keys.toml
   ```
3. **Use environment variables in CI/CD** - Don't store keys in repositories
4. **Rotate keys regularly** - Update keys periodically
5. **Use minimal permissions** - Create API keys with only necessary permissions
6. **Backup securely** - If backing up keys, use encrypted storage

## Troubleshooting

### Key Not Found
```
Error: No API key configured for provider 'openai'
```
**Solution:** Add the key with `lc keys add openai`

### Invalid Key Format
```
Error: Invalid API key format
```
**Solution:** Check the key format for your provider:
- OpenAI: Starts with `sk-`
- Anthropic: Starts with `sk-ant-`
- Google: Base64 JSON for service accounts

### Permission Denied
```
Error: Permission denied accessing keys.toml
```
**Solution:** Check file permissions:
```bash
chmod 600 ~/Library/Application\ Support/lc/keys.toml
```

### Service Account Issues
```
Error: Invalid service account JSON
```
**Solution:** Validate the JSON:
```bash
cat service-account.json | jq .
```

## See Also

- [`lc providers`](./providers.md) - Provider management
- [`lc config`](./config.md) - General configuration
- [Provider Management Guide](../advanced/provider-management.md) - Complete provider setup guide

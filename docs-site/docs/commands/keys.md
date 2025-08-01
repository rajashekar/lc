---
id: keys
title: Keys Command
sidebar_position: 4
---

# Keys Command

Manage API keys for LLM providers securely. The keys command provides encrypted storage and management of API credentials for all configured providers.

## Overview

The keys command handles secure storage of API keys using platform-specific encryption mechanisms. Keys are stored locally and encrypted at rest, ensuring your credentials remain secure while being easily accessible to the LLM client.

## Usage

```bash
# Add an API key for a provider
lc keys add <provider>

# List providers with keys
lc keys list

# Using aliases
lc k a openai
lc k l
```

## Subcommands

| Name     | Alias | Description                        |
|----------|-------|------------------------------------|
| `add`    | `a`   | Add API key for a provider         |
| `list`   | `l`   | List providers with API keys       |
| `get`    | `g`   | Get API key for a provider         |
| `remove` | `r`   | Remove API key for a provider      |

## Options

| Short | Long     | Description | Default |
|-------|----------|-------------|---------|
| `-h`  | `--help` | Print help  | False   |

## Examples

### Add API Keys

```bash
# Add OpenAI key (will prompt securely)
lc keys add openai

# Add multiple provider keys
lc keys add claude
lc keys add together
lc keys add openrouter

# Using aliases
lc k a anthropic
```

### List Configured Keys

```bash
lc keys list
# Output:
# Providers with API keys:
#   • openai
#   • claude
#   • together [key set]
#   • anthropic [no key]

# Short form
lc k l
```

### Get API Key (for scripts)

```bash
# Get key value (use with caution)
lc keys get openai

# Short form
lc k g openai
```

### Remove API Keys

```bash
lc keys remove old-provider
lc k r old-provider
```

### Real-world Workflow

```bash
# Set up multiple providers
lc providers add openai https://api.openai.com/v1
lc keys add openai

lc providers add claude https://api.anthropic.com/v1 -c /messages
lc keys add claude

lc providers add together https://api.together.xyz/v1
lc keys add together

# Verify setup
lc providers list
lc keys list
```

## Troubleshooting

### Common Issues

#### "Provider not found"

- **Error**: Provider must be added before setting keys
- **Solution**: Use `lc providers add <name> <url>` first

#### "Failed to store key"

- **Error**: Platform keychain access denied
- **Solution**: Grant keychain access when prompted
- **Alternative**: Check system keychain permissions

#### "Key not found"

- **Error**: API key not set for provider
- **Solution**: Use `lc keys add <provider>` to set the key

#### Platform-specific Issues

**macOS**:

- May prompt for keychain access
- Keys stored in macOS Keychain
- Requires security permissions

**Linux**:

- Uses secret-service or keyring
- May require gnome-keyring or similar
- Falls back to encrypted file storage

**Windows**:

- Uses Windows Credential Manager
- Requires appropriate user permissions

### Security Best Practices

1. **Never share keys**: Keys are personal credentials
2. **Use environment variables**: For CI/CD and scripts
3. **Rotate keys regularly**: Generate new keys periodically
4. **Remove unused keys**: Clean up old provider keys

```bash
# For scripts, prefer environment variables
export OPENAI_API_KEY="sk-..."
lc --provider openai "Hello world"

# Rather than storing in lc keys
```

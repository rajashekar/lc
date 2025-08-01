---
id: config
title: Config Command
sidebar_position: 5
---

# Config Command

Define and manage default configuration settings for the LLM Client including providers, models, and execution options.

## Overview

Configurations allow users to specify default values for frequently used settings to streamline LLM Client operations. You can set and get values for provider, model, system prompts, token limits, and temperature settings.

## Usage

```bash
# Set a default model
lc config set model gpt-4

# Retrieve the current provider
lc config get provider

# Set system prompt
lc config set system-prompt "You are a helpful assistant"

# Using aliases
lc co s model gpt-4
lc co g provider
```

## Subcommands

| Name   | Alias | Description                      |
|--------|-------|----------------------------------|
| `set`  | `s`   | Set configuration values         |
| `get`  | `g`   | Retrieve configuration values    |

### Set Subcommands

| Name            | Alias | Description                 |
|-----------------|-------|-----------------------------|
| `provider`      | `p`   | Set default provider        |
| `model`         | `m`   | Set default model           |
| `system-prompt` | `s`   | Set system prompt           |
| `max-tokens`    | `mt`  | Set max tokens              |
| `temperature`   | `te`  | Set temperature             |

### Get Subcommands

| Name            | Alias | Description                 |
|-----------------|-------|-----------------------------|
| `provider`      | `p`   | Get default provider        |
| `model`         | `m`   | Get default model           |
| `system-prompt` | `s`   | Get system prompt           |
| `max-tokens`    | `mt`  | Get max tokens              |
| `temperature`   | `te`  | Get temperature             |

## Options

| Short | Long     | Description | Default |
|-------|----------|-------------|---------|
| `-h`  | `--help` | Print help  | False   |

## Examples

### Setting Configuration Values

**Basic Configuration**

```bash
# Set default provider
lc config set provider openai
lc co s provider openai

# Set default model
lc config set model gpt-4
lc co s model gpt-4

# Set system prompt
lc config set system-prompt "You are a helpful assistant"
lc co s system-prompt "You are a helpful assistant"

# Set token limits
lc config set max-tokens 4096
lc co s max-tokens 4096

# Set temperature
lc config set temperature 0.7
lc co s temperature 0.7
```

**Getting Configuration Values**

```bash
# Get current provider
lc config get provider
lc co g provider

# Get current model
lc config get model
lc co g model

# Get all settings
lc config get provider && lc config get model && lc config get temperature
```

### Complete Setup Workflow

```bash
# Initial setup
lc providers add openai https://api.openai.com/v1
lc keys add openai

# Configure defaults
lc config set provider openai
lc config set model gpt-4
lc config set max-tokens 2048
lc config set temperature 0.5

# Verify configuration
lc config get provider
lc config get model

# Test with defaults
lc "Hello, world!"  # Uses configured defaults
```

### Environment-Specific Configurations

```bash
# Development environment
lc config set model gpt-3.5-turbo    # Faster, cheaper
lc config set temperature 0.8        # More creative

# Production environment
lc config set model gpt-4             # Higher quality
lc config set temperature 0.2        # More consistent
```

## Troubleshooting

### Common Issues

#### "Unable to save configuration"

- **Error**: Failed to update config settings
- **Solution**: Verify write permissions for config directory
- **Check**: `ls -la ~/.config/lc/`

#### "Invalid configuration value"

- **Error**: Unrecognized configuration option
- **Solution**: Use `lc config --help` to see valid options
- **Solution**: Check spelling and format of config names

#### "Configuration not found"

- **Error**: Config file doesn't exist
- **Solution**: Run any `lc config set` command to create initial config
- **Solution**: Check config directory exists: `~/.config/lc/`

### Best Practices

1. **Consistent Defaults**: Set reasonable defaults for your workflow
2. **Environment Variables**: Use env vars for sensitive or dynamic values
3. **Backup Configuration**: Keep backups of working configurations
4. **Team Standards**: Establish team-wide configuration standards

### Configuration File Management

```bash
# View current config file location
echo ~/.config/lc/config.json

# Backup configuration
cp ~/.config/lc/config.json ~/.config/lc/config.backup.json

# Restore from backup
cp ~/.config/lc/config.backup.json ~/.config/lc/config.json

# Reset to defaults (delete config file)
rm ~/.config/lc/config.json
```

### Security Considerations

- Configuration files may contain sensitive default values
- Secure access to configuration directory
- Review configurations before sharing
- Use environment variables for secrets instead of config files

```bash
# Secure config directory
chmod 700 ~/.config/lc/
chmod 600 ~/.config/lc/config.json
```

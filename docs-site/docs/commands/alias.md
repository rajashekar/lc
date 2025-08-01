---
id: alias
title: Alias Command
sidebar_position: 12
---

# Alias Command

Manage model aliases for simplified model selection. Aliases allow you to create short, memorable names for complex model specifications like `provider:model` combinations.

## Overview

Model aliases simplify the process of using models with long names or specific provider combinations. Instead of typing `openrouter:anthropic/claude-3.5-sonnet`, you can create an alias like `claude` and use it consistently.

## Usage

```bash
# Add a new alias
lc alias add <name> <target>

# Remove an alias
lc alias delete <name>

# List all aliases
lc alias list

# Using aliases
lc a a gpt4 "openai:gpt-4"
lc a d old-alias
lc a l
```

## Subcommands

| Name     | Alias | Description           |
|----------|-------|-----------------------|
| `add`    | `a`   | Add a new alias       |
| `delete` | `d`   | Remove an alias       |
| `list`   | `l`   | List all aliases      |

## Options

| Short | Long     | Description | Default |
|-------|----------|-------------|---------|
| `-h`  | `--help` | Print help  | False   |

## Examples

### Create Aliases

```bash
# Simple model alias
lc alias add gpt4 "gpt-4"

# Provider:model alias
lc alias add claude "anthropic:claude-3.5-sonnet"

# Complex alias with specific provider
lc alias add llama "openrouter:meta-llama/llama-3.1-405b-instruct"

# Using aliases for short names
lc a a fast "gpt-3.5-turbo"
lc a a smart "gpt-4"
lc a a cheap "claude-haiku"
```

### List Aliases

```bash
lc alias list
# Output:
# Configured aliases:
#   • gpt4 → gpt-4
#   • claude → anthropic:claude-3.5-sonnet
#   • fast → gpt-3.5-turbo
#   • smart → gpt-4

# Short form
lc a l
```

### Use Aliases

```bash
# Use alias in prompts
lc -m gpt4 "Hello world"

# Use alias in chat
lc chat -m claude

# Set as default
lc config set model fast
```

### Practical Aliases

```bash
# By use case
lc alias add coding "gpt-4"
lc alias add writing "claude-3.5-sonnet"
lc alias add analysis "gpt-4-turbo"

# By speed/cost
lc alias add fast "gpt-3.5-turbo"
lc alias add balanced "gpt-4"
lc alias add premium "gpt-4-turbo"

# By provider preference
lc alias add openai-best "openai:gpt-4"
lc alias add anthropic-best "anthropic:claude-3.5-sonnet"
```

## Troubleshooting

### Common Issues

#### "Alias already exists"

- **Error**: Trying to create alias with existing name
- **Solution**: Use `lc alias delete <name>` first, or choose different name

#### "Target model not found"

- **Error**: Alias points to non-existent model
- **Solution**: Verify model name with `lc models -q <name>`
- **Solution**: Check provider:model format is correct

#### "Circular alias reference"

- **Error**: Alias points to another alias creating a loop
- **Solution**: Aliases must point directly to models, not other aliases

### Best Practices

1. **Use meaningful names**: Choose names that reflect the model's purpose
2. **Consistent naming**: Develop a naming convention for your team
3. **Document aliases**: Keep track of what each alias represents
4. **Regular cleanup**: Remove unused or outdated aliases

### Naming Conventions

```bash
# By capability
lc alias add vision "gpt-4-vision-preview"
lc alias add tools "gpt-4-1106-preview"
lc alias add reasoning "o1-preview"

# By provider
lc alias add oai-fast "openai:gpt-3.5-turbo"
lc alias add ant-smart "anthropic:claude-3.5-sonnet"

# By cost tier
lc alias add tier1 "gpt-3.5-turbo"  # cheapest
lc alias add tier2 "gpt-4"          # balanced
lc alias add tier3 "gpt-4-turbo"    # premium
```

### Team Workflow

```bash
# Create team-standard aliases
lc alias add review "gpt-4"           # for code review
lc alias add docs "claude-3.5-sonnet" # for documentation
lc alias add debug "gpt-3.5-turbo"    # for quick debugging

# Share alias configurations
lc sync to aws  # backup aliases to cloud
```

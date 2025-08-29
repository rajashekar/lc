---
id: completions
title: Shell Completions
sidebar_position: 3
---

# Shell Completions

The `lc completions` command generates shell completion scripts that provide intelligent tab completion for all `lc` commands, options, and dynamic values like providers and models.

## Overview

Shell completions significantly enhance your productivity by:

- **Reducing typing**: Tab-complete commands, options, and values
- **Preventing errors**: See available options before executing
- **Discovering features**: Explore commands without reading documentation
- **Context awareness**: Completions adapt based on current command context
- **Alias support**: All command aliases work seamlessly

## Supported Shells

The completion system supports all major shells:

| Shell | Status | Installation |
|-------|---------|-------------|
| **Zsh** | ✅ Full support with rich features | Most advanced |
| **Bash** | ✅ Full support | Standard features |
| **Fish** | ✅ Full support | Fish-native completions |
| **PowerShell** | ✅ Full support | Windows/Cross-platform |
| **Elvish** | ✅ Full support | Modern shell features |

## Quick Setup

### Generate Completion Script

```bash
# Generate completion script for your shell
lc completions <shell>

# Examples for different shells
lc completions zsh     # For Zsh
lc completions bash    # For Bash
lc completions fish    # For Fish
lc completions pwsh    # For PowerShell
lc completions elvish  # For Elvish
```

### One-Time Setup

```bash
# Temporary setup (current session only)
source <(lc completions zsh)

# Or save and source
lc completions zsh > /tmp/lc_completion.zsh
source /tmp/lc_completion.zsh
```

## Permanent Installation

### Zsh

#### Option 1: Using eval (Recommended)
```bash
# Add to your ~/.zshrc
echo 'eval "$(lc completions zsh)"' >> ~/.zshrc
source ~/.zshrc
```

#### Option 2: Using completion directory
```bash
# Generate completion file
lc completions zsh > ~/.local/share/zsh/site-functions/_lc

# Add to fpath in ~/.zshrc (if not already there)
echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
source ~/.zshrc
```

#### Option 3: Oh My Zsh users
```bash
# Create completions directory if it doesn't exist
mkdir -p ~/.oh-my-zsh/custom/plugins/lc

# Generate completion
lc completions zsh > ~/.oh-my-zsh/custom/plugins/lc/_lc

# Add to plugins in ~/.zshrc
plugins=(... lc)
```

### Bash

#### Linux
```bash
# System-wide installation
sudo lc completions bash > /etc/bash_completion.d/lc

# User installation
mkdir -p ~/.local/share/bash-completion/completions
lc completions bash > ~/.local/share/bash-completion/completions/lc
```

#### macOS (with Homebrew bash-completion)
```bash
# If you have bash-completion v2
lc completions bash > $(brew --prefix)/etc/bash_completion.d/lc

# Or add to your ~/.bash_profile
echo 'eval "$(lc completions bash)"' >> ~/.bash_profile
source ~/.bash_profile
```

### Fish

```bash
# Generate and install completion
lc completions fish > ~/.config/fish/completions/lc.fish

# Reload Fish completions
fish_update_completions
```

### PowerShell

#### Windows PowerShell
```powershell
# Create profile if it doesn't exist
if (!(Test-Path $PROFILE)) { New-Item -Path $PROFILE -Type File -Force }

# Add completion to profile
Add-Content $PROFILE "lc completions pwsh | Out-String | Invoke-Expression"

# Reload profile
. $PROFILE
```

#### PowerShell Core (Cross-platform)
```bash
# Linux/macOS with PowerShell Core
pwsh -Command "lc completions pwsh >> \$PROFILE"
```

### Elvish

```bash
# Generate completion for Elvish
lc completions elvish > ~/.config/elvish/lib/lc-completion.elv

# Add to ~/.config/elvish/rc.elv
echo 'use ./lib/lc-completion' >> ~/.config/elvish/rc.elv
```

## Advanced Features

### Dynamic Provider Completion

The completion system dynamically reads your configured providers:

```bash
# Shows all configured providers
lc -p <TAB>
# Example output: anthropic  openai  openrouter  together

lc providers models <TAB>
# Shows same providers for the models subcommand
```

### Context-Aware Model Completion

Model completion adapts based on the selected provider:

```bash
# Shows all models in provider:model format
lc -m <TAB>
# Example: anthropic:claude-3-opus  openai:gpt-4  openai:gpt-3.5-turbo

# When provider is specified, shows only that provider's models
lc -p openai -m <TAB>
# Shows only: gpt-4  gpt-3.5-turbo  gpt-4-turbo

# Complex model names with colons are preserved
lc -m <TAB>
# Example: amazon:amazon.nova-pro-v1:0  bedrock:anthropic.claude-3-opus-20240229-v1:0
```

### Command Alias Support

All command aliases work perfectly with completion:

```bash
lc p <TAB>          # Same as: lc providers <TAB>
lc c <TAB>          # Same as: lc chat <TAB>  
lc k <TAB>          # Same as: lc keys <TAB>
lc co <TAB>         # Same as: lc config <TAB>

# Nested alias completion
lc p m <TAB>        # Same as: lc providers models <TAB>
lc p l <TAB>        # Same as: lc providers list <TAB>
```

### Multi-Level Command Completion

```bash
# Main commands
lc <TAB>
# Shows: chat, providers, keys, models, config, logs, etc.

# Subcommands
lc providers <TAB>
# Shows: list, add, remove, models, sync

# Options and flags
lc chat --<TAB>
# Shows: --model, --provider, --temperature, --max-tokens, etc.
```

## Completion Examples

### Basic Commands
```bash
lc <TAB>                    # All main commands
lc ch<TAB>                  # Completes to: chat
lc pro<TAB>                 # Completes to: providers
```

### Providers and Models
```bash
lc -p <TAB>                 # Shows configured providers
lc -p openai -m <TAB>       # Shows OpenAI models only
lc providers models <TAB>   # Shows providers for models command
```

### Configuration
```bash
lc config <TAB>             # Shows: get, set, list, reset
lc config set <TAB>         # Shows available config keys
lc keys <TAB>               # Shows: add, remove, list
```

### Advanced Features
```bash
lc search <TAB>             # Shows: query, provider
lc search provider <TAB>    # Shows: add, remove, list
lc vector-db <TAB>          # Shows: create, add, search
```

## Shell-Specific Features

### Zsh Features
- **Rich descriptions**: Shows detailed information about each option
- **Menu completion**: Navigate through options with arrow keys
- **Fuzzy matching**: Partial matches work intelligently
- **Color coding**: Syntax highlighting for different option types

### Bash Features
- **Standard completion**: Full compatibility with bash-completion
- **Filename completion**: Automatic path completion where appropriate
- **History integration**: Works with bash history and reverse search

### Fish Features
- **Inline descriptions**: Shows help text as you type
- **Substring matching**: Flexible matching algorithm
- **Syntax highlighting**: Real-time command validation
- **Auto-suggestions**: Predictive text based on history

### PowerShell Features
- **Parameter sets**: Context-aware parameter completion
- **Type validation**: Ensures correct parameter types
- **Pipeline integration**: Works with PowerShell objects
- **Tab expansion**: Standard PowerShell completion behavior

## Troubleshooting

### Completion Not Working

1. **Verify installation**:
   ```bash
   # Check if lc is in PATH
   which lc
   
   # Test basic completion generation
   lc completions zsh | head -n 10
   ```

2. **Re-source shell configuration**:
   ```bash
   # Zsh
   source ~/.zshrc
   
   # Bash
   source ~/.bashrc  # or ~/.bash_profile
   
   # Fish
   fish_update_completions
   ```

3. **Check shell compatibility**:
   ```bash
   # Verify shell version
   echo $SHELL
   zsh --version  # or bash --version
   ```

### No Provider/Model Completion

1. **Verify providers are configured**:
   ```bash
   lc providers list
   ```

2. **Check models are available**:
   ```bash
   lc models list
   ```

3. **Test completion generation**:
   ```bash
   # This should show your providers
   lc completions zsh | grep -A 10 "_lc_providers"
   ```

### Slow Completion

1. **Check for network issues**: Model listing might be slow if provider APIs are unresponsive
2. **Clear shell completion cache**: Some shells cache completions
3. **Use faster completion method**: Switch to eval-based setup if using file-based

### Permission Issues

```bash
# Fix directory permissions
mkdir -p ~/.local/share/zsh/site-functions
chmod 755 ~/.local/share/zsh/site-functions

# Fix completion file permissions
chmod 644 ~/.local/share/zsh/site-functions/_lc
```

## Performance Tips

1. **Use eval method**: `eval "$(lc completions zsh)"` is often faster than file-based completion
2. **Cache configuration**: The system caches provider and model lists for performance
3. **Enable completion caching**: Most shells support completion result caching
4. **Optimize shell startup**: Put completion setup near the end of your shell config

## Integration with Development Tools

### With Oh My Zsh
```bash
# Create custom plugin
mkdir -p ~/.oh-my-zsh/custom/plugins/lc-completions
echo 'eval "$(lc completions zsh)"' > ~/.oh-my-zsh/custom/plugins/lc-completions/lc-completions.plugin.zsh

# Add to plugins
plugins=(... lc-completions)
```

### With Prezto
```bash
# Add to ~/.zpreztorc
zstyle ':prezto:load' pmodule 'completion'

# Add completion to ~/.zshrc after Prezto initialization
eval "$(lc completions zsh)"
```

### With Starship Prompt
Shell completions work seamlessly with Starship and other prompt customizations.

## Command Reference

```bash
lc completions --help
```

**Usage**: `lc completions <SHELL>`

**Arguments**:
- `<SHELL>`: The shell to generate completions for
  - `bash` - Bourne Again Shell
  - `elvish` - Elvish shell  
  - `fish` - Fish shell
  - `pwsh` - PowerShell
  - `zsh` - Z shell

**Examples**:
```bash
lc completions zsh              # Generate Zsh completions
lc completions bash > lc.bash   # Save Bash completions to file
eval "$(lc completions fish)"   # Load Fish completions immediately
```

The completion system provides a professional, feature-rich experience that adapts to your workflow and helps you work more efficiently with the `lc` CLI tool.

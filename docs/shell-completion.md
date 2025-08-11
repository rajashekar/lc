# Shell Completion for lc CLI

The `lc` CLI tool supports comprehensive shell completion for all major shells, including dynamic completion for providers, models, and vector databases based on your current configuration.

## Features

- **Static completion**: All commands, subcommands, and flags
- **Command alias support**: All command aliases work with completion (e.g., `lc p <TAB>` shows providers subcommands)
- **Dynamic completion**: Context-aware completion for:
  - Providers (e.g., `lc -p g<TAB>` shows providers starting with "g")
  - Models (based on available providers)
  - Vector databases (from your configuration)
- **Multi-shell support**: Bash, Zsh, Fish, PowerShell, and Elvish

## Quick Setup

### Generate Completion Scripts

First, generate the completion script for your shell:

```bash
# For Bash
lc completions bash > ~/.local/share/bash-completion/completions/lc

# For Zsh
lc completions zsh > ~/.local/share/zsh/site-functions/_lc

# For Fish
lc completions fish > ~/.config/fish/completions/lc.fish

# For PowerShell
lc completions power-shell > lc.ps1

# For Elvish
lc completions elvish > lc.elv
```

### Shell-Specific Setup Instructions

#### Bash

1. Generate the completion script:
   ```bash
   lc completions bash > ~/.local/share/bash-completion/completions/lc
   ```

2. If the directory doesn't exist, create it:
   ```bash
   mkdir -p ~/.local/share/bash-completion/completions
   ```

3. Reload your shell or source your `.bashrc`:
   ```bash
   source ~/.bashrc
   ```

**Alternative method** (add to `~/.bashrc`):
```bash
# Add this line to your ~/.bashrc
eval "$(lc completions bash)"
```

#### Zsh

1. Generate the completion script:
   ```bash
   lc completions zsh > ~/.local/share/zsh/site-functions/_lc
   ```

2. If the directory doesn't exist, create it and add to fpath:
   ```bash
   mkdir -p ~/.local/share/zsh/site-functions
   echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> ~/.zshrc
   ```

3. Reload completions:
   ```bash
   autoload -U compinit && compinit
   ```

**Alternative method** (add to `~/.zshrc`):
```bash
# Add this line to your ~/.zshrc
eval "$(lc completions zsh)"
```

#### Fish

1. Generate the completion script:
   ```bash
   lc completions fish > ~/.config/fish/completions/lc.fish
   ```

2. If the directory doesn't exist, create it:
   ```bash
   mkdir -p ~/.config/fish/completions
   ```

3. Restart Fish or reload completions:
   ```bash
   fish -c "source ~/.config/fish/completions/lc.fish"
   ```

#### PowerShell

1. Generate the completion script:
   ```powershell
   lc completions power-shell > lc.ps1
   ```

2. Add to your PowerShell profile:
   ```powershell
   # Add this line to your PowerShell profile
   . path\to\lc.ps1
   ```

3. Find your profile location:
   ```powershell
   echo $PROFILE
   ```

#### Elvish

1. Generate the completion script:
   ```bash
   lc completions elvish > lc.elv
   ```

2. Add to your Elvish configuration:
   ```elvish
   # Add this line to your ~/.elvish/rc.elv
   eval (lc completions elvish)
   ```

## Usage Examples

Once set up, you can use tab completion for various scenarios:

### Basic Command Completion
```bash
lc <TAB>                    # Shows all available commands
lc p<TAB>                   # Completes to "providers"
lc providers <TAB>          # Shows provider subcommands (add, list, etc.)
```

### Command Alias Completion
```bash
lc p <TAB>                  # Shows providers subcommands (add, list, models, etc.)
lc p l<TAB>                 # Completes to "list"
lc k <TAB>                  # Shows keys subcommands (add, list, get, remove)
lc m <TAB>                  # Shows models subcommands (refresh, info, dump, etc.)
lc c <TAB>                  # Shows chat command options (-m, -p, --cid, etc.)
```

### Dynamic Provider Completion
```bash
lc -p <TAB>                 # Shows all configured providers
lc -p g<TAB>                # Shows providers starting with "g" (e.g., "groq", "gemini")
lc --provider o<TAB>        # Shows providers starting with "o" (e.g., "openai")
```

### Model and Vector Database Completion
```bash
lc -m <TAB>                 # Shows available models
lc -v <TAB>                 # Shows configured vector databases
lc --vectordb my<TAB>       # Shows vector databases starting with "my"
```

### Complex Command Completion
```bash
lc providers add <TAB>      # Shows required arguments
lc config set <TAB>         # Shows configuration options
lc sync configure <TAB>     # Shows sync configuration commands
```

## Dynamic Completion Details

The completion system provides intelligent, context-aware suggestions:

- **Provider completion**: Reads from your actual provider configuration
- **Model completion**: Based on cached model information from providers
- **Vector database completion**: From your vector database configurations
- **Command-specific completion**: Different completions based on the current command context

## Troubleshooting

### Completion Not Working

1. **Verify installation**:
   ```bash
   lc completions bash --help  # Should show help for completions command
   ```

2. **Check shell configuration**:
   - Ensure the completion script is in the correct location
   - Verify your shell's completion system is enabled
   - Try restarting your shell

3. **Test basic completion**:
   ```bash
   lc <TAB><TAB>  # Should show available commands
   ```

### Dynamic Completion Issues

1. **Provider completion not working**:
   - Ensure you have providers configured: `lc providers list`
   - Check that the `lc` command is in your PATH
   - Verify the completion script includes dynamic functions

2. **Slow completion**:
   - Dynamic completion queries your configuration, which may take a moment
   - This is normal for the first completion in a session

### Shell-Specific Issues

#### Bash
- Ensure `bash-completion` package is installed
- Check that `/etc/bash_completion` or similar is sourced in your `.bashrc`

#### Zsh
- Verify `compinit` is called in your `.zshrc`
- Check that the completion directory is in your `fpath`

#### Fish
- Fish automatically loads completions from `~/.config/fish/completions/`
- No additional configuration should be needed

## Advanced Configuration

### Custom Completion Locations

You can place completion scripts in custom locations:

```bash
# Custom location example
lc completions bash > /path/to/custom/location/lc_completion.bash
source /path/to/custom/location/lc_completion.bash
```

### Integration with Shell Frameworks

#### Oh My Zsh
```bash
# Create plugin directory
mkdir -p ~/.oh-my-zsh/custom/plugins/lc
lc completions zsh > ~/.oh-my-zsh/custom/plugins/lc/_lc

# Add to plugins in ~/.zshrc
plugins=(... lc)
```

#### Prezto
```bash
# Add to Prezto completions
lc completions zsh > ~/.zprezto/modules/completion/external/src/_lc
```

## Performance Notes

- Static completions are instant
- Dynamic completions may have a slight delay (typically <100ms) as they query your configuration
- Completion results are not cached between sessions to ensure accuracy

## Contributing

If you encounter issues with shell completion or have suggestions for improvements, please:

1. Check the existing issues on GitHub
2. Provide details about your shell, OS, and the specific completion problem
3. Include the output of `lc completions <your-shell> | head -20` for debugging

The completion system is designed to be robust and work across different environments, but edge cases may exist for specific shell configurations.
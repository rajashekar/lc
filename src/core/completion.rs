//! Shell completion support for the lc CLI
//!
//! This module provides both static completion generation and dynamic completion
//! support for values that depend on the current configuration.

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::{Cli, CompletionShell};
use crate::config::Config;

/// Generate shell completions for the specified shell
pub async fn generate_completions(shell: CompletionShell) -> Result<()> {
    let mut cmd = Cli::command();
    let shell_type = match shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Zsh => Shell::Zsh,
        CompletionShell::Fish => Shell::Fish,
        CompletionShell::PowerShell => Shell::PowerShell,
        CompletionShell::Elvish => Shell::Elvish,
    };

    // Generate basic completions
    generate(shell_type, &mut cmd, "lc", &mut io::stdout());

    // Add custom completion functions for dynamic values
    match shell {
        CompletionShell::Bash => generate_bash_dynamic_completions(),
        CompletionShell::Zsh => generate_zsh_dynamic_completions(),
        CompletionShell::Fish => generate_fish_dynamic_completions(),
        _ => {
            eprintln!(
                "Note: Dynamic completions for providers are not yet supported for {:?}",
                shell
            );
            eprintln!("Basic command completions have been generated.");
        }
    }

    Ok(())
}

/// Generate dynamic completion functions for Bash
fn generate_bash_dynamic_completions() {
    println!(
        r#"
# Dynamic completion functions for lc (Bash)
_lc_complete_providers() {{
    local providers
    providers=$(lc providers list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null || echo "")
    COMPREPLY=($(compgen -W "$providers" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

_lc_complete_models() {{
    local models provider
    # Check if a provider was specified with -p or --provider
    for ((i=1; i<COMP_CWORD; i++)); do
        if [[ "${{COMP_WORDS[i]}}" == "-p" || "${{COMP_WORDS[i]}}" == "--provider" ]]; then
            provider="${{COMP_WORDS[i+1]}}"
            break
        elif [[ "${{COMP_WORDS[i]}}" =~ ^--provider= ]]; then
            provider="${{COMP_WORDS[i]#--provider=}}"
            break
        elif [[ "${{COMP_WORDS[i]}}" =~ ^-p.+ ]]; then
            provider="${{COMP_WORDS[i]#-p}}"
            break
        fi
    done
    
    if [[ -n "$provider" ]]; then
        # Get models for specific provider (extract full model name including colons)
        models=$(lc providers models "$provider" 2>/dev/null | grep "  •" | awk -F' ' '{{gsub(/^  • /, "", $0); gsub(/ \(.*$/, "", $0); gsub(/ \[.*$/, "", $0); print $1}}' 2>/dev/null || echo "")
    else
        # Get all models in provider:model format
        models=$(lc models 2>/dev/null | awk '
            /^[a-zA-Z0-9_-]+:$/ {{ provider = substr($0, 1, length($0)-1) }}
            /^  •/ {{
                gsub(/^  • /, "")
                gsub(/ \(.*$/, "")
                gsub(/ \[.*$/, "")
                if (provider != "") print provider ":" $0
            }}
        ' 2>/dev/null || echo "")
    fi
    COMPREPLY=($(compgen -W "$models" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

_lc_complete_vectordbs() {{
    local vectordbs
    vectordbs=$(lc vectors list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null || echo "")
    COMPREPLY=($(compgen -W "$vectordbs" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

# Enhanced completion function with alias support
_lc_enhanced() {{
    local cur prev opts cmd
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    
    # Handle command aliases by expanding them
    if [[ COMP_CWORD -ge 1 ]]; then
        cmd="${{COMP_WORDS[1]}}"
        case "$cmd" in
            p)
                COMP_WORDS[1]="providers"
                ;;
            k)
                COMP_WORDS[1]="keys"
                ;;
            l)
                COMP_WORDS[1]="logs"
                ;;
            co)
                COMP_WORDS[1]="config"
                ;;
            c)
                COMP_WORDS[1]="chat"
                ;;
            m)
                COMP_WORDS[1]="models"
                ;;
            a)
                COMP_WORDS[1]="alias"
                ;;
            t)
                COMP_WORDS[1]="templates"
                ;;
            pr)
                COMP_WORDS[1]="proxy"
                ;;
            e)
                COMP_WORDS[1]="embed"
                ;;
            s)
                COMP_WORDS[1]="similar"
                ;;
            v)
                COMP_WORDS[1]="vectors"
                ;;
            w)
                COMP_WORDS[1]="web-chat-proxy"
                ;;
            sy)
                COMP_WORDS[1]="sync"
                ;;
            se)
                COMP_WORDS[1]="search"
                ;;
            img)
                COMP_WORDS[1]="image"
                ;;
            dump)
                COMP_WORDS[1]="dump-metadata"
                ;;
        esac
    fi
    
    case "$prev" in
        -p|--provider)
            _lc_complete_providers
            return 0
            ;;
        -m|--model)
            _lc_complete_models
            return 0
            ;;
        -v|--vectordb|--database)
            _lc_complete_vectordbs
            return 0
            ;;
    esac
    
    # Fall back to default completion
    _lc "$@"
}}

# Register the enhanced completion
complete -F _lc_enhanced lc

# Instructions for setup
# Add the above to your ~/.bashrc or ~/.bash_completion to enable dynamic completions
# Then run: source ~/.bashrc
"#
    );
}

/// Generate dynamic completion functions for Zsh
fn generate_zsh_dynamic_completions() {
    println!(
        r#"
# Dynamic completion functions for lc (Zsh)
_lc_providers() {{
    local providers
    providers=($(lc providers list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null || echo ""))
    _describe 'providers' providers
}}

_lc_models() {{
    local models provider
    # Check if a provider was specified with -p or --provider in the current command line
    local -a words
    words=(${{(z)BUFFER}})
    
    for ((i=1; i<=${{#words}}; i++)); do
        if [[ "${{words[i]}}" == "-p" || "${{words[i]}}" == "--provider" ]]; then
            provider="${{words[i+1]}}"
            break
        elif [[ "${{words[i]}}" =~ ^--provider= ]]; then
            provider="${{words[i]#--provider=}}"
            break
        elif [[ "${{words[i]}}" =~ ^-p.+ ]]; then
            provider="${{words[i]#-p}}"
            break
        fi
    done
    
    if [[ -n "$provider" ]]; then
        # Get models for specific provider (extract full model name including colons)
        models=($(lc providers models "$provider" 2>/dev/null | grep "  •" | awk -F' ' '{{gsub(/^  • /, "", $0); gsub(/ \(.*$/, "", $0); gsub(/ \[.*$/, "", $0); print $1}}' 2>/dev/null || echo ""))
        # For provider-specific models, just use the model names directly
        _describe 'models' models
    else
        # Get all models in provider:model format
        local raw_models
        raw_models=($(lc models 2>/dev/null | awk '
            /^[a-zA-Z0-9_-]+:$/ {{ provider = substr($0, 1, length($0)-1) }}
            /^  •/ {{
                gsub(/^  • /, "")
                gsub(/ \(.*$/, "")
                gsub(/ \[.*$/, "")
                if (provider != "") print provider ":" $0
            }}
        ' 2>/dev/null || echo ""))
        
        # Use compadd with proper display format: "provider -- model"
        local -a completions descriptions
        for model in $raw_models; do
            local provider_part="${{model%%:*}}"
            local model_part="${{model#*:}}"
            completions+=("$model")
            descriptions+=("$provider_part -- $model_part")
        done
        
        if [[ ${{#completions}} -gt 0 ]]; then
            compadd -d descriptions -a completions
        fi
    fi
}}

_lc_vectordbs() {{
    local vectordbs
    vectordbs=($(lc vectors list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null || echo ""))
    _describe 'vectordbs' vectordbs
}}

# Override the default completion to use our dynamic functions
# This replaces _default with our custom functions in the generated completion
if (( $+functions[_lc] )); then
    # Modify the existing _lc function to use our dynamic completions
    eval "$(declare -f _lc | sed \
        -e "s/:PROVIDER:_default/:PROVIDER:_lc_providers/g" \
        -e "s/:MODEL:_default/:MODEL:_lc_models/g" \
        -e "s/:VECTORDB:_default/:VECTORDB:_lc_vectordbs/g" \
        -e "s/:DATABASE:_default/:DATABASE:_lc_vectordbs/g")"
fi

# Custom wrapper function to handle command aliases
_lc_with_aliases() {{
    local context curcontext="$curcontext" state line
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    # First, let the original _lc function handle most of the work
    _lc "$@"
    ret=$?
    
    # If we're in a command context and have an alias, handle it specially
    if [[ $state == "lc" && -n $line[2] ]]; then
        case $line[2] in
            (p)
                # Redirect 'p' alias to 'providers' subcommand completion
                words=("providers" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-providers:"
                
                # Handle special case for 'lc p m <TAB>' (providers models command)
                if [[ ${{#words}} -ge 3 && "${{words[2]}}" == "m" ]]; then
                    # This is 'lc p m <TAB>' - should complete with provider names
                    _lc_providers
                    ret=$?
                elif [[ ${{#words}} -ge 3 && "${{words[2]}}" == "models" ]]; then
                    # This is 'lc p models <TAB>' - should complete with provider names
                    _lc_providers
                    ret=$?
                else
                    _lc__providers_commands
                    ret=$?
                fi
                ;;
            (k)
                # Redirect 'k' alias to 'keys' subcommand completion
                words=("keys" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-keys:"
                _lc__keys_commands
                ret=$?
                ;;
            (l)
                # Redirect 'l' alias to 'logs' subcommand completion
                words=("logs" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-logs:"
                _lc__logs_commands
                ret=$?
                ;;
            (co)
                # Redirect 'co' alias to 'config' subcommand completion
                words=("config" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-config:"
                _lc__config_commands
                ret=$?
                ;;
            (c)
                # Redirect 'c' alias to 'chat' subcommand completion
                words=("chat" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-chat:"
                # Chat command has no subcommands, so just complete its options
                _arguments "${{_arguments_options[@]}}" : \
                    '-m+[Model to use for the chat]:MODEL:_lc_models' \
                    '--model=[Model to use for the chat]:MODEL:_lc_models' \
                    '-p+[Provider to use for the chat]:PROVIDER:_lc_providers' \
                    '--provider=[Provider to use for the chat]:PROVIDER:_lc_providers' \
                    '--cid=[Chat ID to use or continue]:CHAT_ID:_default' \
                    '-t+[Include tools from MCP server(s)]:TOOLS:_default' \
                    '--tools=[Include tools from MCP server(s)]:TOOLS:_default' \
                    '-v+[Vector database name for RAG]:DATABASE:_lc_vectordbs' \
                    '--vectordb=[Vector database name for RAG]:DATABASE:_lc_vectordbs' \
                    '-d[Enable debug/verbose logging]' \
                    '--debug[Enable debug/verbose logging]' \
                    '*-i+[Attach image(s) to the chat]:IMAGES:_default' \
                    '*--image=[Attach image(s) to the chat]:IMAGES:_default' \
                    '-h[Print help]' \
                    '--help[Print help]'
                ret=$?
                ;;
            (m)
                # Redirect 'm' alias to 'models' subcommand completion
                words=("models" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-models:"
                _lc__models_commands
                ret=$?
                ;;
            (a)
                # Redirect 'a' alias to 'alias' subcommand completion
                words=("alias" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-alias:"
                _lc__alias_commands
                ret=$?
                ;;
            (t)
                # Redirect 't' alias to 'templates' subcommand completion
                words=("templates" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-templates:"
                _lc__templates_commands
                ret=$?
                ;;
            (pr)
                # Redirect 'pr' alias to 'proxy' subcommand completion
                words=("proxy" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-proxy:"
                # Proxy command has no subcommands, so just complete its options
                _arguments "${{_arguments_options[@]}}" : \
                    '-p+[Port to listen on]:PORT:_default' \
                    '--port=[Port to listen on]:PORT:_default' \
                    '--host=[Host to bind to]:HOST:_default' \
                    '--provider=[Filter by provider]:PROVIDER:_lc_providers' \
                    '-m+[Filter by specific model]:MODEL:_lc_models' \
                    '--model=[Filter by specific model]:MODEL:_lc_models' \
                    '-k+[API key for authentication]:API_KEY:_default' \
                    '--key=[API key for authentication]:API_KEY:_default' \
                    '-g[Generate a random API key]' \
                    '--generate-key[Generate a random API key]' \
                    '-h[Print help]' \
                    '--help[Print help]'
                ret=$?
                ;;
            (e)
                # Redirect 'e' alias to 'embed' subcommand completion
                words=("embed" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-embed:"
                # Embed command has no subcommands, so just complete its options
                _arguments "${{_arguments_options[@]}}" : \
                    '-m+[Model to use for embeddings]:MODEL:_lc_models' \
                    '--model=[Model to use for embeddings]:MODEL:_lc_models' \
                    '-p+[Provider to use for embeddings]:PROVIDER:_lc_providers' \
                    '--provider=[Provider to use for embeddings]:PROVIDER:_lc_providers' \
                    '-v+[Vector database name to store embeddings]:DATABASE:_lc_vectordbs' \
                    '--vectordb=[Vector database name to store embeddings]:DATABASE:_lc_vectordbs' \
                    '*-f+[Files to embed]:FILES:_files' \
                    '*--files=[Files to embed]:FILES:_files' \
                    '-d[Enable debug/verbose logging]' \
                    '--debug[Enable debug/verbose logging]' \
                    '-h[Print help]' \
                    '--help[Print help]' \
                    '::text -- Text to embed:_default'
                ret=$?
                ;;
            (s)
                # Redirect 's' alias to 'similar' subcommand completion
                words=("similar" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-similar:"
                # Similar command has no subcommands, so just complete its options
                _arguments "${{_arguments_options[@]}}" : \
                    '-m+[Model to use for embeddings]:MODEL:_lc_models' \
                    '--model=[Model to use for embeddings]:MODEL:_lc_models' \
                    '-p+[Provider to use for embeddings]:PROVIDER:_lc_providers' \
                    '--provider=[Provider to use for embeddings]:PROVIDER:_lc_providers' \
                    '-v+[Vector database name to search]:DATABASE:_lc_vectordbs' \
                    '--vectordb=[Vector database name to search]:DATABASE:_lc_vectordbs' \
                    '-l+[Number of similar results to return]:LIMIT:_default' \
                    '--limit=[Number of similar results to return]:LIMIT:_default' \
                    '-h[Print help]' \
                    '--help[Print help]' \
                    ':query -- Query text to find similar content:_default'
                ret=$?
                ;;
            (v)
                # Redirect 'v' alias to 'vectors' subcommand completion
                words=("vectors" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-vectors:"
                _lc__vectors_commands
                ret=$?
                ;;
            (w)
                # Redirect 'w' alias to 'web-chat-proxy' subcommand completion
                words=("web-chat-proxy" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-web-chat-proxy:"
                _lc__web_chat_proxy_commands
                ret=$?
                ;;
            (sy)
                # Redirect 'sy' alias to 'sync' subcommand completion
                words=("sync" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-sync:"
                _lc__sync_commands
                ret=$?
                ;;
            (se)
                # Redirect 'se' alias to 'search' subcommand completion
                words=("search" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-search:"
                _lc__search_commands
                ret=$?
                ;;
            (img)
                # Redirect 'img' alias to 'image' subcommand completion
                words=("image" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-image:"
                # Image command has no subcommands, so just complete its options
                _arguments "${{_arguments_options[@]}}" : \
                    '-m+[Model to use for image generation]:MODEL:_lc_models' \
                    '--model=[Model to use for image generation]:MODEL:_lc_models' \
                    '-p+[Provider to use for image generation]:PROVIDER:_lc_providers' \
                    '--provider=[Provider to use for image generation]:PROVIDER:_lc_providers' \
                    '-s+[Image size]:SIZE:_default' \
                    '--size=[Image size]:SIZE:_default' \
                    '-n+[Number of images to generate]:COUNT:_default' \
                    '--count=[Number of images to generate]:COUNT:_default' \
                    '-o+[Output directory for generated images]:OUTPUT:_directories' \
                    '--output=[Output directory for generated images]:OUTPUT:_directories' \
                    '-d[Enable debug/verbose logging]' \
                    '--debug[Enable debug/verbose logging]' \
                    '-h[Print help]' \
                    '--help[Print help]' \
                    ':prompt -- Text prompt for image generation:_default'
                ret=$?
                ;;
            (dump)
                # Redirect 'dump' alias to 'dump-metadata' subcommand completion
                words=("dump-metadata" "${{words[@]:2}}")
                (( CURRENT -= 1 ))
                curcontext="${{curcontext%:*:*}}:lc-command-dump-metadata:"
                # Dump-metadata command has no subcommands, so just complete its options
                _arguments "${{_arguments_options[@]}}" : \
                    '-l[List available cached metadata files]' \
                    '--list[List available cached metadata files]' \
                    '-h[Print help]' \
                    '--help[Print help]' \
                    '::provider -- Specific provider to dump:_lc_providers'
                ret=$?
                ;;
        esac
    fi
    
    return ret
}}

# Replace the main completion function with our alias-aware version
compdef _lc_with_aliases lc

# Instructions for setup
# Add the above to your ~/.zshrc or a file in your fpath to enable dynamic provider completion
# Then run: source ~/.zshrc
"#
    );
}

/// Generate dynamic completion functions for Fish
fn generate_fish_dynamic_completions() {
    println!(
        r#"
# Dynamic completion functions for lc (Fish)
function __lc_complete_providers
    lc providers list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null
end

# Add dynamic provider completion
complete -c lc -s p -l provider -f -a "(__lc_complete_providers)" -d "Provider to use"

# Instructions for setup
# Add the above to ~/.config/fish/completions/lc.fish to enable dynamic provider completion
# The file will be loaded automatically by Fish
"#
    );
}

/// Get list of available providers for completion
#[allow(dead_code)]
pub fn get_available_providers() -> Vec<String> {
    match Config::load() {
        Ok(config) => {
            let mut providers: Vec<String> = config.providers.keys().cloned().collect();
            providers.sort();
            providers
        }
        Err(_) => Vec::new(),
    }
}

/// Get list of available models for completion (simplified version)
#[allow(dead_code)]
pub fn get_available_models() -> Vec<String> {
    // For now, return common model names
    // In a full implementation, this would load from cache
    vec![
        "gpt-4".to_string(),
        "gpt-4-turbo".to_string(),
        "gpt-3.5-turbo".to_string(),
        "claude-3-sonnet".to_string(),
        "claude-3-haiku".to_string(),
        "gemini-pro".to_string(),
    ]
}

/// Get list of available vector databases for completion
#[allow(dead_code)]
pub fn get_available_vectordbs() -> Vec<String> {
    crate::vector_db::VectorDatabase::list_databases().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"gpt-4".to_string()));
    }
}

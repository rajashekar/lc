//! Shell completion commands

use crate::cli::{Cli, CompletionShell};
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Handle shell completion generation
pub async fn handle(shell: CompletionShell) -> Result<()> {
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
            eprintln!("Note: Dynamic completions for providers are not yet supported for {:?}", shell);
            eprintln!("Basic command completions have been generated.");
        }
    }
    
    Ok(())
}

/// Generate dynamic completion functions for Bash
fn generate_bash_dynamic_completions() {
    println!(r#"
# Dynamic completion functions for lc (Bash)
_lc_complete_providers() {{
    local providers
    providers=$(lc providers list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null || echo "")
    COMPREPLY=($(compgen -W "$providers" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

_lc_complete_models() {{
    local models
    models=$(lc models 2>/dev/null | awk '/^  •/ {{gsub(/^  • /, ""); print $1}}' 2>/dev/null || echo "")
    COMPREPLY=($(compgen -W "$models" -- "${{COMP_WORDS[COMP_CWORD]}}"))
}}

# Register enhanced completion for provider and model options
complete -o default -F _lc lc

# Instructions for setup
# Add the above to your ~/.bashrc or ~/.bash_completion
# Then run: source ~/.bashrc
"#);
}

/// Generate dynamic completion functions for Zsh
fn generate_zsh_dynamic_completions() {
    println!(r#"
# Dynamic completion functions for lc (Zsh)
_lc_providers() {{
    local providers
    providers=($(lc providers list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null || echo ""))
    _describe 'providers' providers
}}

_lc_models() {{
    local models
    models=($(lc models 2>/dev/null | awk '/^  •/ {{gsub(/^  • /, ""); print $1}}' 2>/dev/null || echo ""))
    _describe 'models' models
}}

# Instructions for setup
# Add the above to your ~/.zshrc or a file in your fpath
# Then run: source ~/.zshrc
"#);
}

/// Generate dynamic completion functions for Fish
fn generate_fish_dynamic_completions() {
    println!(r#"
# Dynamic completion functions for lc (Fish)
function __lc_complete_providers
    lc providers list 2>/dev/null | grep "  •" | awk '{{print $2}}' 2>/dev/null
end

function __lc_complete_models
    lc models 2>/dev/null | awk '/^  •/ {{gsub(/^  • /, ""); print $1}}' 2>/dev/null
end

# Add dynamic completions
complete -c lc -s p -l provider -f -a "(__lc_complete_providers)" -d "Provider to use"
complete -c lc -s m -l model -f -a "(__lc_complete_models)" -d "Model to use"

# Instructions for setup
# Add the above to ~/.config/fish/completions/lc.fish
# The file will be loaded automatically by Fish
"#);
}

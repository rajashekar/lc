//! Template management commands

use anyhow::Result;
use colored::Colorize;
use crate::cli::TemplateCommands;
use crate::config;

/// Handle template-related commands
pub async fn handle(command: TemplateCommands) -> Result<()> {
    match command {
        TemplateCommands::Add { name, prompt } => {
            let mut config = config::Config::load()?;
            config.add_template(name.clone(), prompt.clone())?;
            config.save()?;
            println!("{} Template '{}' added", "✓".green(), name);
        }
        TemplateCommands::Delete { name } => {
            let mut config = config::Config::load()?;
            config.remove_template(name.clone())?;
            config.save()?;
            println!("{} Template '{}' removed", "✓".green(), name);
        }
        TemplateCommands::List => {
            let config = config::Config::load()?;
            let templates = config.list_templates();

            if templates.is_empty() {
                println!("No templates configured.");
            } else {
                println!("\n{}", "Templates:".bold().blue());
                for (name, prompt) in templates {
                    let display_prompt = if prompt.len() > 60 {
                        format!("{}...", &prompt[..60])
                    } else {
                        prompt.clone()
                    };
                    println!("  {} {} -> {}", "•".blue(), name.bold(), display_prompt);
                }
            }
        }
    }

    Ok(())
}
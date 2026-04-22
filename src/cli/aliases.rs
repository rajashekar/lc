//! Alias management commands

use crate::cli::AliasCommands;
use crate::data::config::Config;
use anyhow::Result;

/// Handle alias-related commands
pub async fn handle(command: AliasCommands) -> Result<()> {
    match command {
        AliasCommands::Add { name, target } => {
            let mut config = Config::load()?;
            config.add_alias(name.clone(), target.clone())?;
            config.save()?;
            println!("Added alias '{}' -> '{}'", name, target);
            Ok(())
        }
        AliasCommands::Delete { name } => {
            let mut config = Config::load()?;
            config.remove_alias(name.clone())?;
            config.save()?;
            println!("Removed alias '{}'", name);
            Ok(())
        }
        AliasCommands::List => {
            let config = Config::load()?;
            let aliases = config.list_aliases();

            if aliases.is_empty() {
                println!("No aliases configured");
            } else {
                println!("Configured aliases:");
                for (alias_name, target) in aliases {
                    println!("  {} -> {}", alias_name, target);
                }
            }
            Ok(())
        }
    }
}

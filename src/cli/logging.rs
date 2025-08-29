//! Logging and log management commands

use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use crate::cli::{LogCommands, RecentCommands, AnswerCommands};
use crate::database;

/// Handle log-related commands
pub async fn handle(command: LogCommands) -> Result<()> {
    let db = database::Database::new()?;

    match command {
        LogCommands::Show { minimal } => show_logs(&db, minimal).await,
        LogCommands::Recent { command, count } => handle_recent(&db, command, count).await,
        LogCommands::Current => show_current(&db).await,
        LogCommands::Stats => show_stats(&db).await,
        LogCommands::Purge {
            yes,
            older_than_days,
            keep_recent,
            max_size_mb,
        } => handle_purge(&db, yes, older_than_days, keep_recent, max_size_mb).await,
    }
}

async fn show_logs(db: &database::Database, minimal: bool) -> Result<()> {
    let entries = db.get_all_logs()?;

    if entries.is_empty() {
        println!("No chat logs found.");
        return Ok(());
    }

    if minimal {
        use tabled::{Table, Tabled};

        #[derive(Tabled)]
        struct LogEntry {
            #[tabled(rename = "Chat ID")]
            chat_id: String,
            #[tabled(rename = "Model")]
            model: String,
            #[tabled(rename = "Question")]
            question: String,
            #[tabled(rename = "Time")]
            time: String,
        }

        let table_data: Vec<LogEntry> = entries
            .into_iter()
            .map(|entry| LogEntry {
                chat_id: entry.chat_id[..8].to_string(),
                model: entry.model,
                question: if entry.question.len() > 50 {
                    format!("{}...", &entry.question[..50])
                } else {
                    entry.question
                },
                time: entry.timestamp.format("%m-%d %H:%M").to_string(),
            })
            .collect();

        let table = Table::new(table_data);
        println!("{}", table);
    } else {
        println!("\n{}", "Chat Logs:".bold().blue());

        for entry in entries {
            println!(
                "\n{} {} ({})",
                "Session:".bold(),
                &entry.chat_id[..8],
                entry.timestamp.format("%Y-%m-%d %H:%M:%S")
            );
            println!("{} {}", "Model:".bold(), entry.model);

            // Show token usage if available
            if let (Some(input_tokens), Some(output_tokens)) =
                (entry.input_tokens, entry.output_tokens)
            {
                println!(
                    "{} {} input + {} output = {} total tokens",
                    "Tokens:".bold(),
                    input_tokens,
                    output_tokens,
                    input_tokens + output_tokens
                );
            }

            println!("{} {}", "Q:".yellow(), entry.question);
            println!(
                "{} {}",
                "A:".green(),
                if entry.response.len() > 200 {
                    format!("{}...", &entry.response[..200])
                } else {
                    entry.response
                }
            );
            println!("{}", "─".repeat(80).dimmed());
        }
    }

    Ok(())
}

async fn handle_recent(db: &database::Database, command: Option<RecentCommands>, count: usize) -> Result<()> {
    match command {
        Some(RecentCommands::Answer { command }) => {
            let entries = db.get_all_logs()?;
            if let Some(entry) = entries.first() {
                match command {
                    Some(AnswerCommands::Code) => {
                        let code_blocks = extract_code_blocks(&entry.response);
                        if code_blocks.is_empty() {
                            anyhow::bail!("No code blocks found in the last answer");
                        } else {
                            for block in code_blocks {
                                println!("{}", block);
                            }
                        }
                    }
                    None => {
                        println!("{}", entry.response);
                    }
                }
            } else {
                anyhow::bail!("No recent logs found");
            }
        }
        Some(RecentCommands::Question) => {
            let entries = db.get_all_logs()?;
            if let Some(entry) = entries.first() {
                println!("{}", entry.question);
            } else {
                anyhow::bail!("No recent logs found");
            }
        }
        Some(RecentCommands::Model) => {
            let entries = db.get_all_logs()?;
            if let Some(entry) = entries.first() {
                println!("{}", entry.model);
            } else {
                anyhow::bail!("No recent logs found");
            }
        }
        Some(RecentCommands::Session) => {
            let entries = db.get_all_logs()?;
            if let Some(entry) = entries.first() {
                println!("{}", entry.chat_id);
            } else {
                anyhow::bail!("No recent logs found");
            }
        }
        None => {
            // Default behavior - show recent logs
            let mut entries = db.get_all_logs()?;
            entries.truncate(count);

            if entries.is_empty() {
                println!("No recent logs found.");
                return Ok(());
            }

            println!(
                "\n{} (showing {} entries)",
                "Recent Logs:".bold().blue(),
                entries.len()
            );

            for entry in entries {
                println!(
                    "\n{} {} ({})",
                    "Session:".bold(),
                    &entry.chat_id[..8],
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S")
                );
                println!("{} {}", "Model:".bold(), entry.model);

                // Show token usage if available
                if let (Some(input_tokens), Some(output_tokens)) =
                    (entry.input_tokens, entry.output_tokens)
                {
                    println!(
                        "{} {} input + {} output = {} total tokens",
                        "Tokens:".bold(),
                        input_tokens,
                        output_tokens,
                        input_tokens + output_tokens
                    );
                }

                println!("{} {}", "Q:".yellow(), entry.question);
                println!(
                    "{} {}",
                    "A:".green(),
                    if entry.response.len() > 150 {
                        format!("{}...", &entry.response[..150])
                    } else {
                        entry.response
                    }
                );
                println!("{}", "─".repeat(60).dimmed());
            }
        }
    }

    Ok(())
}

async fn show_current(db: &database::Database) -> Result<()> {
    if let Some(session_id) = db.get_current_session_id()? {
        let history = db.get_chat_history(&session_id)?;

        println!("\n{} {}", "Current Session:".bold().blue(), session_id);
        println!("{} {} messages", "Messages:".bold(), history.len());

        for (i, entry) in history.iter().enumerate() {
            println!(
                "\n{} {} ({})",
                format!("Message {}:", i + 1).bold(),
                entry.model,
                entry.timestamp.format("%H:%M:%S")
            );
            println!("{} {}", "Q:".yellow(), entry.question);
            println!(
                "{} {}",
                "A:".green(),
                if entry.response.len() > 100 {
                    format!("{}...", &entry.response[..100])
                } else {
                    entry.response.clone()
                }
            );
        }
    } else {
        println!("No current session found.");
    }

    Ok(())
}

async fn show_stats(db: &database::Database) -> Result<()> {
    let stats = db.get_stats()?;

    println!("\n{}", "Database Statistics:".bold().blue());
    println!();

    // Basic stats
    println!("{} {}", "Total Entries:".bold(), stats.total_entries);
    println!("{} {}", "Unique Sessions:".bold(), stats.unique_sessions);

    // File size formatting
    let file_size_str = if stats.file_size_bytes < 1024 {
        format!("{} bytes", stats.file_size_bytes)
    } else if stats.file_size_bytes < 1024 * 1024 {
        format!("{:.1} KB", stats.file_size_bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", stats.file_size_bytes as f64 / (1024.0 * 1024.0))
    };
    println!("{} {}", "Database Size:".bold(), file_size_str);

    // Date range
    if let Some((earliest, latest)) = stats.date_range {
        println!(
            "{} {} to {}",
            "Date Range:".bold(),
            earliest.format("%Y-%m-%d %H:%M:%S"),
            latest.format("%Y-%m-%d %H:%M:%S")
        );
    } else {
        println!("{} {}", "Date Range:".bold(), "No entries".dimmed());
    }

    // Model usage
    if !stats.model_usage.is_empty() {
        println!("\n{}", "Model Usage:".bold().blue());
        for (model, count) in stats.model_usage {
            let percentage = if stats.total_entries > 0 {
                (count as f64 / stats.total_entries as f64) * 100.0
            } else {
                0.0
            };
            println!(
                "  {} {} ({} - {:.1}%)",
                "•".blue(),
                model.bold(),
                count,
                percentage
            );
        }
    }

    Ok(())
}

async fn handle_purge(
    db: &database::Database,
    yes: bool,
    older_than_days: Option<u32>,
    keep_recent: Option<usize>,
    max_size_mb: Option<u64>,
) -> Result<()> {
    // Check if any specific purge options are provided
    let has_specific_options =
        older_than_days.is_some() || keep_recent.is_some() || max_size_mb.is_some();

    if has_specific_options {
        // Smart purge with specific options
        let deleted_count = db.smart_purge(older_than_days, keep_recent, max_size_mb)?;

        if deleted_count > 0 {
            println!("{} Purged {} log entries", "✓".green(), deleted_count);

            if let Some(days) = older_than_days {
                println!("  - Removed entries older than {} days", days);
            }
            if let Some(count) = keep_recent {
                println!("  - Kept only the {} most recent entries", count);
            }
            if let Some(size) = max_size_mb {
                println!("  - Enforced maximum database size of {} MB", size);
            }
        } else {
            println!("{} No logs needed to be purged", "ℹ️".blue());
        }
    } else {
        // Full purge (existing behavior)
        if !yes {
            print!(
                "Are you sure you want to purge all logs? This cannot be undone. (y/N): "
            );
            // Deliberately flush stdout to ensure prompt appears before user input
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().to_lowercase().starts_with('y') {
                println!("Purge cancelled.");
                return Ok(());
            }
        }

        db.purge_all_logs()?;
        println!("{} All logs purged successfully", "✓".green());
    }

    Ok(())
}

// Helper function to extract code blocks from markdown text
fn extract_code_blocks(text: &str) -> Vec<String> {
    let mut code_blocks = Vec::new();
    let mut in_code_block = false;
    let mut current_block = String::new();

    for line in text.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                if !current_block.trim().is_empty() {
                    code_blocks.push(current_block.trim().to_string());
                }
                current_block.clear();
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
            }
        } else if in_code_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    // Handle case where code block doesn't end properly
    if in_code_block && !current_block.trim().is_empty() {
        code_blocks.push(current_block.trim().to_string());
    }

    code_blocks
}
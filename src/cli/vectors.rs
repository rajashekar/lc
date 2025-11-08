//! Vector database commands

use crate::cli::VectorCommands;
use crate::data::vector_db::VectorDatabase;
use anyhow::Result;
use colored::*;

/// Handle vector database commands
pub async fn handle(command: VectorCommands) -> Result<()> {
    match command {
        VectorCommands::List => {
            let databases = VectorDatabase::list_databases()?;

            if databases.is_empty() {
                println!("No vector databases found.");
                println!(
                    "Create one by running: {}",
                    "lc embed -d <name> -m <model> \"your text\"".dimmed()
                );
            } else {
                println!("\n{} Vector databases:", "📊".bold().blue());

                for db_name in databases {
                    match VectorDatabase::new(&db_name) {
                        Ok(db) => {
                            let count = db.count().unwrap_or(0);
                            let model_info = db.get_model_info().unwrap_or(None);

                            print!("  {} {} ({} vectors)", "•".blue(), db_name.bold(), count);

                            if let Some((model, provider)) = model_info {
                                print!(" - {}:{}", provider.dimmed(), model.dimmed());
                            }

                            println!();
                        }
                        Err(_) => {
                            println!("  {} {} (error reading)", "•".red(), db_name.bold());
                        }
                    }
                }
            }
        }
        VectorCommands::Create { name } => {
            // Check if database already exists
            let databases = VectorDatabase::list_databases()?;
            if databases.contains(&name) {
                anyhow::bail!("Vector database '{}' already exists", name);
            }

            // Create empty database
            let _ = VectorDatabase::new(&name)?;
            println!(
                "{} Vector database '{}' created successfully",
                "✓".green(),
                name
            );
            println!(
                "Use {} to add embeddings",
                format!("lc embed -d {} <text>", name).dimmed()
            );
        }
        VectorCommands::Delete { name, yes } => {
            // Check if database exists
            let databases = VectorDatabase::list_databases()?;
            if !databases.contains(&name) {
                anyhow::bail!("Vector database '{}' not found", name);
            }

            // Ask for confirmation unless --yes is provided
            if !yes {
                println!(
                    "{} Are you sure you want to delete database '{}'? This cannot be undone.",
                    "⚠️".yellow(),
                    name.bold()
                );
                print!("Type 'yes' to confirm: ");
                use std::io::{self, Write};
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "yes" {
                    println!("Deletion cancelled.");
                    return Ok(());
                }
            }

            VectorDatabase::delete_database(&name)?;
            println!(
                "{} Vector database '{}' deleted successfully",
                "✓".green(),
                name
            );
        }
        VectorCommands::Info { name } => {
            // Check if database exists
            let databases = VectorDatabase::list_databases()?;
            if !databases.contains(&name) {
                anyhow::bail!("Vector database '{}' not found", name);
            }

            let db = VectorDatabase::new(&name)?;
            let count = db.count()?;
            let model_info = db.get_model_info()?;

            println!("\n{} Database: {}", "ℹ️".bold().blue(), name.bold());
            println!("  Vector count: {}", count);

            if let Some((model, provider)) = model_info {
                println!("  Model: {}", model);
                println!("  Provider: {}", provider);
            } else {
                println!("  Model: {}", "Not set".dimmed());
                println!("  Provider: {}", "Not set".dimmed());
            }

            // Show recent entries if any
            if count > 0 {
                println!("\n{} Recent entries:", "📝".bold().blue());
                let vectors = db.get_all_vectors()?;
                for (i, entry) in vectors.iter().take(10).enumerate() {
                    let preview = if entry.text.len() > 80 {
                        format!("{}...", &entry.text[..80])
                    } else {
                        entry.text.clone()
                    };

                    let source_info = if let Some(ref file_path) = entry.file_path {
                        if let (Some(chunk_idx), Some(total_chunks)) =
                            (entry.chunk_index, entry.total_chunks)
                        {
                            format!(" [{}:{}/{}]", file_path, chunk_idx + 1, total_chunks)
                        } else {
                            format!(" [{}]", file_path)
                        }
                    } else {
                        String::new()
                    };

                    println!(
                        "  {}. {}{} ({})",
                        i + 1,
                        preview,
                        source_info.dimmed(),
                        entry
                            .created_at
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                            .dimmed()
                    );
                }

                if vectors.len() > 10 {
                    println!("  ... and {} more", vectors.len() - 10);
                }
            }
        }
        VectorCommands::Stats { name } => {
            let databases = VectorDatabase::list_databases()?;
            if !databases.contains(&name) {
                anyhow::bail!("Vector database '{}' not found", name);
            }

            let db = VectorDatabase::new(&name)?;
            let count = db.count()?;
            let model_info = db.get_model_info()?;

            println!("\n{} Vector database: {}", "📊".bold().blue(), name.bold());
            println!("Vectors: {}", count);

            if let Some((model, provider)) = model_info {
                println!("Model: {}:{}", provider, model);

                // Try to get dimensions from first vector
                if count > 0 {
                    let vectors = db.get_all_vectors()?;
                    if let Some(first) = vectors.first() {
                        println!("Dimensions: {}", first.vector.len());
                    }
                }
            } else {
                println!("Model: {}", "Not set".dimmed());
            }

            // Get database file size
            let embeddings_dir = VectorDatabase::embeddings_dir()?;
            let db_path = embeddings_dir.join(format!("{}.db", name));
            if let Ok(metadata) = std::fs::metadata(&db_path) {
                let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                println!("Size: {:.2} MB", size_mb);
            }

            if count > 0 {
                println!("\n{} Recent entries:", "📝".bold().blue());
                let vectors = db.get_all_vectors()?;
                for (i, entry) in vectors.iter().take(5).enumerate() {
                    let preview = if entry.text.len() > 60 {
                        format!("{}...", &entry.text[..60])
                    } else {
                        entry.text.clone()
                    };

                    let source_info = if let Some(ref file_path) = entry.file_path {
                        if let (Some(chunk_idx), Some(total_chunks)) =
                            (entry.chunk_index, entry.total_chunks)
                        {
                            format!(" [{}:{}/{}]", file_path, chunk_idx + 1, total_chunks)
                        } else {
                            format!(" [{}]", file_path)
                        }
                    } else {
                        String::new()
                    };

                    println!(
                        "  {}. {}{} ({})",
                        i + 1,
                        preview,
                        source_info.dimmed(),
                        entry
                            .created_at
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                            .dimmed()
                    );
                }

                if vectors.len() > 5 {
                    println!("  ... and {} more", vectors.len() - 5);
                }
            }
        }
        VectorCommands::Clear { name, yes } => {
            // Check if database exists
            let databases = VectorDatabase::list_databases()?;
            if !databases.contains(&name) {
                anyhow::bail!("Vector database '{}' not found", name);
            }

            let db = VectorDatabase::new(&name)?;
            let count = db.count()?;

            if count == 0 {
                println!("Vector database '{}' is already empty.", name);
                return Ok(());
            }

            // Ask for confirmation unless --yes is provided
            if !yes {
                println!(
                    "{} Are you sure you want to clear {} vectors from database '{}'? This cannot be undone.",
                    "⚠️".yellow(),
                    count,
                    name.bold()
                );
                print!("Type 'yes' to confirm: ");
                use std::io::{self, Write};
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "yes" {
                    println!("Clear operation cancelled.");
                    return Ok(());
                }
            }

            // Delete and recreate the database to clear it
            VectorDatabase::delete_database(&name)?;
            let _ = VectorDatabase::new(&name)?;

            println!(
                "{} Vector database '{}' cleared successfully ({} vectors removed)",
                "✓".green(),
                name,
                count
            );
        }
    }

    Ok(())
}

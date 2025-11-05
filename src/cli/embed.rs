//! Embedding commands implementation

use anyhow::Result;
use colored::*;

use crate::chat;
use crate::cli::set_debug_mode;
use crate::config;
use crate::data::vector_db::{FileProcessor, VectorDatabase};
use crate::provider::EmbeddingRequest;
use crate::utils::resolve_model_and_provider;

/// Handle embed command
pub async fn handle_embed_command(
    model: String,
    provider: Option<String>,
    database: Option<String>,
    files: Vec<String>,
    text: Option<String>,
    debug: bool,
) -> Result<()> {
    // Set debug mode if requested
    if debug {
        set_debug_mode(true);
    }

    // Validate input: either text or files must be provided
    if text.is_none() && files.is_empty() {
        anyhow::bail!("Either text or files must be provided for embedding");
    }

    let config = config::Config::load()?;

    // Resolve provider and model using the same logic as direct prompts
    let (provider_name, resolved_model) =
        resolve_model_and_provider(&config, provider, Some(model))?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    // Allow either API key or resolved custom auth headers (e.g., x-goog-api-key)
    let header_has_resolved_key = provider_config.headers.iter().any(|(k, v)| {
        let k_l = k.to_lowercase();
        (k_l.contains("key") || k_l.contains("token") || k_l.contains("auth"))
            && !v.trim().is_empty()
            && !v.contains("${api_key}")
    });
    if provider_config.api_key.is_none() && !header_has_resolved_key {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    println!("{} Starting embedding process...", "🔄".blue());
    println!("{} Model: {}", "📊".blue(), resolved_model);
    println!("{} Provider: {}", "🏢".blue(), provider_name);

    let mut total_embeddings = 0;
    let mut total_tokens = 0;

    // Process files if provided
    if !files.is_empty() {
        println!("{} Processing files with glob patterns...", "📁".blue());

        // Expand file patterns and filter for text files
        let file_paths = FileProcessor::expand_file_patterns(&files)?;

        if file_paths.is_empty() {
            println!(
                "{} No text files found matching the patterns",
                "⚠️".yellow()
            );
        } else {
            println!(
                "{} Found {} text files to process",
                "✅".green(),
                file_paths.len()
            );

            for file_path in file_paths {
                println!("\n{} Processing file: {}", "📄".blue(), file_path.display());

                // Read and chunk the file
                match FileProcessor::process_file(&file_path) {
                    Ok(chunks) => {
                        println!("{} Split into {} chunks", "✂️".blue(), chunks.len());

                        // Process each chunk
                        for (chunk_index, chunk) in chunks.iter().enumerate() {
                            let embedding_request = EmbeddingRequest {
                                model: resolved_model.clone(),
                                input: chunk.clone(),
                                encoding_format: Some("float".to_string()),
                            };

                            match client.embeddings(&embedding_request).await {
                                Ok(response) => {
                                    if let Some(embedding_data) = response.data.first() {
                                        total_embeddings += 1;
                                        total_tokens += response.usage.total_tokens;

                                        // Store in vector database if specified
                                        if let Some(db_name) = &database {
                                            match VectorDatabase::new(db_name) {
                                                Ok(vector_db) => {
                                                    let file_path_str = file_path.to_string_lossy();
                                                    match vector_db.add_vector_with_metadata(
                                                        chunk,
                                                        &embedding_data.embedding,
                                                        &resolved_model,
                                                        &provider_name,
                                                        Some(&file_path_str),
                                                        Some(chunk_index as i32),
                                                        Some(chunks.len() as i32),
                                                    ) {
                                                        Ok(id) => {
                                                            println!("  {} Chunk {}/{} stored with ID: {}",
                                                                "💾".green(), chunk_index + 1, chunks.len(), id);
                                                        }
                                                        Err(e) => {
                                                            eprintln!("  Warning: Failed to store chunk {}: {}", chunk_index + 1, e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("  Warning: Failed to create/open vector database '{}': {}", db_name, e);
                                                }
                                            }
                                        } else {
                                            // Just show progress without storing
                                            println!(
                                                "  {} Chunk {}/{} embedded ({} dimensions)",
                                                "✅".green(),
                                                chunk_index + 1,
                                                chunks.len(),
                                                embedding_data.embedding.len()
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "  Warning: Failed to embed chunk {}: {}",
                                        chunk_index + 1,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to process file '{}': {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    // Process text if provided
    if let Some(text_content) = text {
        println!("\n{} Processing text input...", "📝".blue());
        println!(
            "{} Text: \"{}\"",
            "📝".blue(),
            if text_content.len() > 50 {
                format!("{}...", &text_content[..50])
            } else {
                text_content.clone()
            }
        );

        let embedding_request = EmbeddingRequest {
            model: resolved_model.clone(),
            input: text_content.clone(),
            encoding_format: Some("float".to_string()),
        };

        match client.embeddings(&embedding_request).await {
            Ok(response) => {
                if let Some(embedding_data) = response.data.first() {
                    total_embeddings += 1;
                    total_tokens += response.usage.total_tokens;

                    println!(
                        "{} Vector dimensions: {}",
                        "📏".blue(),
                        embedding_data.embedding.len()
                    );

                    // Display vector preview
                    let embedding = &embedding_data.embedding;
                    if embedding.len() > 10 {
                        println!("\n{} Vector preview:", "🔍".blue());
                        print!("  [");
                        for (i, val) in embedding.iter().take(5).enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{:.6}", val);
                        }
                        print!(" ... ");
                        for (i, val) in embedding.iter().skip(embedding.len() - 5).enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{:.6}", val);
                        }
                        println!("]");
                    }

                    // Store in vector database if specified
                    if let Some(db_name) = &database {
                        match VectorDatabase::new(db_name) {
                            Ok(vector_db) => {
                                match vector_db.add_vector(
                                    &text_content,
                                    &embedding,
                                    &resolved_model,
                                    &provider_name,
                                ) {
                                    Ok(id) => {
                                        println!(
                                            "\n{} Stored in vector database '{}' with ID: {}",
                                            "💾".green(),
                                            db_name,
                                            id
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Warning: Failed to store in vector database: {}",
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to create/open vector database '{}': {}",
                                    db_name, e
                                );
                            }
                        }
                    }

                    // Output full vector as JSON for programmatic use
                    if files.is_empty() {
                        // Only show full vector for single text input
                        println!("\n{} Full vector (JSON):", "📋".dimmed());
                        println!("{}", serde_json::to_string(&embedding)?);
                    }
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to generate embeddings for text: {}", e);
            }
        }
    }

    // Summary
    println!("\n{} Embedding process completed!", "🎉".green());
    println!(
        "{} Total embeddings generated: {}",
        "📊".blue(),
        total_embeddings
    );
    println!("{} Total tokens used: {}", "💰".yellow(), total_tokens);

    if let Some(db_name) = &database {
        println!(
            "{} All embeddings stored in database: {}",
            "💾".green(),
            db_name
        );
    }

    Ok(())
}

/// Handle similar command
pub async fn handle_similar_command(
    model: Option<String>,
    provider: Option<String>,
    database: String,
    limit: usize,
    query: String,
) -> Result<()> {
    // Open the vector database
    let vector_db = VectorDatabase::new(&database)?;

    // Check if database has any vectors
    let count = vector_db.count()?;
    if count == 0 {
        anyhow::bail!(
            "Vector database '{}' is empty. Add some vectors first using 'lc embed -d {}'",
            database,
            database
        );
    }

    // Get model info from database if not provided
    let (resolved_model, resolved_provider) = match (&model, &provider) {
        (Some(m), Some(p)) => (m.clone(), p.clone()),
        _ => {
            if let Some((db_model, db_provider)) = vector_db.get_model_info()? {
                if model.is_some() || provider.is_some() {
                    println!(
                        "{} Using model from database: {}:{}",
                        "ℹ️".blue(),
                        db_provider,
                        db_model
                    );
                }
                (db_model, db_provider)
            } else {
                anyhow::bail!(
                    "No model specified and database '{}' has no stored model info",
                    database
                );
            }
        }
    };

    let config = config::Config::load()?;

    // Resolve provider and model
    let (provider_name, model_name) =
        resolve_model_and_provider(&config, Some(resolved_provider), Some(resolved_model))?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    // Allow either API key or resolved custom auth headers (e.g., x-goog-api-key)
    let header_has_resolved_key = provider_config.headers.iter().any(|(k, v)| {
        let k_l = k.to_lowercase();
        (k_l.contains("key") || k_l.contains("token") || k_l.contains("auth"))
            && !v.trim().is_empty()
            && !v.contains("${api_key}")
    });
    if provider_config.api_key.is_none() && !header_has_resolved_key {
        anyhow::bail!(
            "No API key configured for provider '{}'. Add one with 'lc keys add {}'",
            provider_name,
            provider_name
        );
    }

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    // Generate embedding for query
    let embedding_request = EmbeddingRequest {
        model: model_name.clone(),
        input: query.clone(),
        encoding_format: Some("float".to_string()),
    };

    println!("{} Searching for similar content...", "🔍".blue());
    println!("{} Database: {}", "📊".blue(), database);
    println!(
        "{} Query: \"{}\"",
        "📝".blue(),
        if query.len() > 50 {
            format!("{}...", &query[..50])
        } else {
            query.clone()
        }
    );

    match client.embeddings(&embedding_request).await {
        Ok(response) => {
            if let Some(embedding_data) = response.data.first() {
                let query_vector = &embedding_data.embedding;

                // Find similar vectors
                let similar_results = vector_db.find_similar(query_vector, limit)?;

                if similar_results.is_empty() {
                    println!(
                        "\n{} No similar content found in database '{}'",
                        "❌".red(),
                        database
                    );
                } else {
                    println!(
                        "\n{} Found {} similar results:",
                        "✅".green(),
                        similar_results.len()
                    );

                    for (i, (entry, similarity)) in similar_results.iter().enumerate() {
                        let similarity_percent = (similarity * 100.0).round() as u32;
                        let similarity_color = if similarity_percent >= 80 {
                            format!("{}%", similarity_percent).green()
                        } else if similarity_percent >= 60 {
                            format!("{}%", similarity_percent).yellow()
                        } else {
                            format!("{}%", similarity_percent).red()
                        };

                        println!(
                            "\n{} {} (Similarity: {})",
                            format!("{}.", i + 1).bold(),
                            similarity_color,
                            format!("ID: {}", entry.id).dimmed()
                        );
                        println!("   {}", entry.text);
                        println!(
                            "   {}",
                            format!(
                                "Added: {}",
                                entry.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                            )
                            .dimmed()
                        );
                    }
                }
            } else {
                anyhow::bail!("No embedding data in response");
            }
        }
        Err(e) => {
            anyhow::bail!("Failed to generate query embedding: {}", e);
        }
    }

    Ok(())
}

/// RAG helper function to retrieve relevant context
pub async fn retrieve_rag_context(
    db_name: &str,
    query: &str,
    _client: &crate::chat::LLMClient,
    _model: &str,
    _provider: &str,
) -> Result<String> {
    crate::debug_log!(
        "RAG: Starting context retrieval for database '{}' with query '{}'",
        db_name,
        query
    );

    // Open the vector database
    let vector_db = VectorDatabase::new(db_name)?;
    crate::debug_log!("RAG: Successfully opened vector database '{}'", db_name);

    // Check if database has any vectors
    let count = vector_db.count()?;
    crate::debug_log!("RAG: Database '{}' contains {} vectors", db_name, count);
    if count == 0 {
        crate::debug_log!("RAG: Database is empty, returning empty context");
        return Ok(String::new());
    }

    // Get model info from database
    let (db_model, db_provider) = if let Some((m, p)) = vector_db.get_model_info()? {
        crate::debug_log!("RAG: Using database model '{}' from provider '{}'", m, p);
        (m, p)
    } else {
        crate::debug_log!("RAG: No model info in database, returning empty context");
        return Ok(String::new());
    };

    // Create a client for the embedding provider (not the chat provider)
    let config = config::Config::load()?;
    let mut config_mut = config.clone();
    let embedding_client = chat::create_authenticated_client(&mut config_mut, &db_provider).await?;
    crate::debug_log!(
        "RAG: Created embedding client for provider '{}'",
        db_provider
    );

    // Use the database's embedding model for consistency
    let embedding_request = EmbeddingRequest {
        model: db_model.clone(),
        input: query.to_string(),
        encoding_format: Some("float".to_string()),
    };

    crate::debug_log!(
        "RAG: Generating embedding for query using model '{}'",
        db_model
    );

    // Generate embedding for query using the correct provider
    let response = embedding_client.embeddings(&embedding_request).await?;
    crate::debug_log!("RAG: Successfully generated embedding for query");

    if let Some(embedding_data) = response.data.first() {
        let query_vector = &embedding_data.embedding;
        crate::debug_log!("RAG: Query vector has {} dimensions", query_vector.len());

        // Find top 3 most similar vectors for context
        let similar_results = vector_db.find_similar(query_vector, 3)?;
        crate::debug_log!("RAG: Found {} similar results", similar_results.len());

        if similar_results.is_empty() {
            crate::debug_log!("RAG: No similar results found, returning empty context");
            return Ok(String::new());
        }

        // Format context
        let mut context = String::new();
        let mut included_count = 0;
        for (entry, similarity) in similar_results {
            crate::debug_log!(
                "RAG: Result similarity: {:.3} for text: '{}'",
                similarity,
                &entry.text[..50.min(entry.text.len())]
            );
            // Only include results with reasonable similarity (>0.3)
            if similarity > 0.3 {
                context.push_str(&format!("- {}\n", entry.text));
                included_count += 1;
            }
        }

        crate::debug_log!(
            "RAG: Included {} results in context (similarity > 0.3)",
            included_count
        );
        crate::debug_log!("RAG: Final context length: {} characters", context.len());

        Ok(context)
    } else {
        crate::debug_log!("RAG: No embedding data in response, returning empty context");
        Ok(String::new())
    }
}

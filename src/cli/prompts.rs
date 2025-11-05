//! Prompt handling and utilities

use crate::{
    config::Config,
    core::chat::{
        create_authenticated_client, send_chat_request_with_streaming,
        send_chat_request_with_validation,
    },
    database::Database,
    debug_log,
};
use anyhow::Result;

/// Handle direct prompt command
pub async fn handle_direct(
    prompt: String,
    provider: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    max_tokens: Option<String>,
    temperature: Option<String>,
    _attachments: Vec<String>,
    _images: Vec<String>,
    _audio_files: Vec<String>,
    tools: Option<String>,
    _vectordb: Option<String>,
    use_search: Option<String>,
    stream: bool,
) -> Result<()> {
    debug_log!(
        "Handling direct prompt - provider: {:?}, model: {:?}, prompt length: {}",
        provider,
        model,
        prompt.len()
    );
    debug_log!(
        "Request options - max_tokens: {:?}, temperature: {:?}, stream: {}",
        max_tokens,
        temperature,
        stream
    );

    // Load configuration
    let mut config = Config::load()?;

    // Determine provider and model
    let (provider_name, model_name) = determine_provider_and_model(&config, provider, model)?;

    debug_log!(
        "Using provider: '{}', model: '{}'",
        provider_name,
        model_name
    );

    // Create authenticated client - this will automatically use templates from provider config
    debug_log!(
        "Creating authenticated client for provider '{}'",
        provider_name
    );
    let client = create_authenticated_client(&mut config, &provider_name).await?;

    // Parse parameters
    let max_tokens_parsed = max_tokens.as_ref().and_then(|s| s.parse().ok());
    let temperature_parsed = temperature.as_ref().and_then(|s| s.parse().ok());

    // Strip provider prefix from model name for API call if present
    // Handle cases where model name itself contains colons (e.g., gpt-oss:20b)
    let api_model_name = if model_name.contains(':') {
        // Split only on the first colon to separate provider from model
        let parts: Vec<&str> = model_name.splitn(2, ':').collect();
        if parts.len() > 1 {
            parts[1].to_string()
        } else {
            model_name.clone()
        }
    } else {
        model_name.clone()
    };

    debug_log!("Using API model name: '{}'", api_model_name);

    // Process search if --use-search is specified
    let final_prompt = if let Some(search_spec) = use_search {
        debug_log!("Processing search with spec: {}", search_spec);
        
        // Parse search spec (format: "provider" or "provider:query")
        let (search_provider, search_query) = if search_spec.contains(':') {
            let parts: Vec<&str> = search_spec.splitn(2, ':').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (search_spec, prompt.clone())
            }
        } else {
            (search_spec, prompt.clone())
        };
        
        debug_log!("Search provider: '{}', query: '{}'", search_provider, search_query);
        
        // Perform the search
        let search_engine = crate::search::SearchEngine::new()?;
        let search_results = search_engine.search(&search_provider, &search_query, Some(5)).await?;
        
        // Extract context from search results
        let search_context = search_engine.extract_context_for_llm(&search_results, 5);
        
        // Combine search context with original prompt
        let combined_prompt = format!(
            "{}\n\nUser's question: {}",
            search_context,
            prompt
        );
        
        debug_log!("Added search context, combined prompt length: {}", combined_prompt.len());
        println!("🔍 Search completed: {} results from {}\n", search_results.results.len(), search_provider);
        
        combined_prompt
    } else {
        prompt.clone()
    };

    // Fetch MCP tools if specified
    let (mcp_tools, mcp_server_names) = if let Some(tools_str) = &tools {
        crate::core::tools::fetch_mcp_tools(tools_str).await?
    } else {
        (None, Vec::new())
    };

    // Send the request - templates will be automatically applied by the client
    if stream {
        debug_log!("Sending streaming chat request");
        // For streaming, we don't get the response back, so we can't save it to database
        // But we should still create/update the session for consistency
        let db = Database::new()?;
        let _session_id = match db.get_current_session_id()? {
            Some(id) => {
                debug_log!("Using existing session for streaming: {}", id);
                id
            }
            None => {
                // Generate a new session ID using UUID for consistency
                let new_session_id = uuid::Uuid::new_v4().to_string();
                db.set_current_session_id(&new_session_id)?;
                debug_log!("Created new session for streaming: {}", new_session_id);
                new_session_id
            }
        };
        
        send_chat_request_with_streaming(
            &client,
            &api_model_name,
            &final_prompt,
            &[], // No history for direct prompt
            system_prompt.as_deref(),
            max_tokens_parsed,
            temperature_parsed,
            &provider_name,
            mcp_tools.clone(),
        )
        .await?;
        
        // Note: We can't save the response to database in streaming mode
        // as the response is streamed directly to stdout
        eprintln!("\nNote: Streaming responses are not saved to conversation history.");
    } else {
        debug_log!("Sending non-streaming chat request");
        
        // Use tool execution if tools are available
        let (response, input_tokens, output_tokens) = if mcp_tools.is_some() && !mcp_server_names.is_empty() {
            // Convert server names to &str references
            let server_refs: Vec<&str> = mcp_server_names.iter().map(|s| s.as_str()).collect();
            
            crate::core::chat::send_chat_request_with_tool_execution(
                &client,
                &api_model_name,
                &final_prompt,
                &[], // No history for direct prompt
                system_prompt.as_deref(),
                max_tokens_parsed,
                temperature_parsed,
                &provider_name,
                mcp_tools.clone(),
                &server_refs,
                None, // Use default max_iterations
            )
            .await?
        } else {
            send_chat_request_with_validation(
                &client,
                &api_model_name,
                &final_prompt,
                &[], // No history for direct prompt
                system_prompt.as_deref(),
                max_tokens_parsed,
                temperature_parsed,
                &provider_name,
                mcp_tools.clone(),
            )
            .await?
        };

        // Print the response
        println!("{}", response);

        // Save to database
        if let Err(e) = save_to_database(
            &prompt,
            &response,
            &provider_name,
            &api_model_name,
            input_tokens,
            output_tokens,
        )
        .await
        {
            debug_log!("Failed to save to database: {}", e);
        }
    }

    Ok(())
}

/// Handle direct prompt with piped input
pub async fn handle_with_piped_input(
    prompt: String,
    provider: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    max_tokens: Option<String>,
    temperature: Option<String>,
    _attachments: Vec<String>,
    _images: Vec<String>,
    _audio_files: Vec<String>,
    _tools: Option<String>,
    _vectordb: Option<String>,
    _use_search: Option<String>,
    stream: bool,
) -> Result<()> {
    // For piped input, the prompt IS the piped content, so we just call handle_direct
    debug_log!("Handling piped input as direct prompt");
    handle_direct(
        prompt,
        provider,
        model,
        system_prompt,
        max_tokens,
        temperature,
        vec![],
        vec![],
        vec![],
        None,
        None,
        None,
        stream,
    )
    .await
}

// Helper function to determine provider and model
fn determine_provider_and_model(
    config: &Config,
    provider: Option<String>,
    model: Option<String>,
) -> Result<(String, String)> {
    debug_log!(
        "Determining provider and model - provider: {:?}, model: {:?}",
        provider,
        model
    );

    // Check if model is an alias first
    if let Some(ref m) = model {
        if let Some(alias_target) = config.get_alias(m) {
            debug_log!("Resolved alias '{}' to '{}'", m, alias_target);
            // Alias target should be in format "provider:model"
            if alias_target.contains(':') {
                let parts: Vec<&str> = alias_target.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let provider_from_alias = parts[0].to_string();
                    let model_from_alias = alias_target.clone();

                    // If provider is also specified, verify they match
                    if let Some(ref p) = provider {
                        if p != &provider_from_alias {
                            anyhow::bail!("Provider mismatch: -p {} conflicts with alias '{}' which maps to {}",
                                        p, m, alias_target);
                        }
                    }

                    debug_log!(
                        "Using provider '{}' and model '{}' from alias",
                        provider_from_alias,
                        model_from_alias
                    );
                    return Ok((provider_from_alias, model_from_alias));
                }
            }
        }
    }

    // If model contains provider prefix (e.g., "openai:gpt-4"), extract both
    if let Some(ref m) = model {
        if m.contains(':') {
            let parts: Vec<&str> = m.split(':').collect();
            let provider_from_model = parts[0].to_string();
            let model_name = m.clone();

            // If provider is also specified, verify they match
            if let Some(ref p) = provider {
                if p != &provider_from_model {
                    anyhow::bail!(
                        "Provider mismatch: -p {} conflicts with model prefix {}",
                        p,
                        provider_from_model
                    );
                }
            }

            debug_log!(
                "Extracted provider '{}' from model '{}'",
                provider_from_model,
                model_name
            );
            return Ok((provider_from_model, model_name));
        }
    }

    // Use provided provider or default to "openai"
    let provider_name = provider.unwrap_or_else(|| "openai".to_string());

    // Use provided model or default for the provider
    let model_name = model.unwrap_or_else(|| {
        // Fallback defaults per provider
        match provider_name.as_str() {
            "openai" => "gpt-4o-mini".to_string(),
            "anthropic" | "claude" => "claude-3-5-sonnet-20241022".to_string(),
            "gemini" => "gemini-1.5-flash".to_string(),
            _ => "gpt-3.5-turbo".to_string(),
        }
    });

    // Add provider prefix if not present
    let final_model = if model_name.contains(':') {
        model_name
    } else {
        format!("{}:{}", provider_name, model_name)
    };

    debug_log!(
        "Final provider: '{}', model: '{}'",
        provider_name,
        final_model
    );
    Ok((provider_name, final_model))
}

// Helper function to save to database
async fn save_to_database(
    prompt: &str,
    response: &str,
    _provider: &str,
    model: &str,
    input_tokens: Option<i32>,
    output_tokens: Option<i32>,
) -> Result<()> {
    let db = Database::new()?;

    // Get or create session ID
    let session_id = match db.get_current_session_id()? {
        Some(id) => {
            debug_log!("Using existing session: {}", id);
            id
        }
        None => {
            // Generate a new session ID using UUID for consistency
            let new_session_id = uuid::Uuid::new_v4().to_string();
            db.set_current_session_id(&new_session_id)?;
            debug_log!("Created new session: {}", new_session_id);
            new_session_id
        }
    };

    // Save the entry with tokens
    db.save_chat_entry_with_tokens(
        &session_id,
        model,
        prompt,
        response,
        input_tokens,
        output_tokens,
    )?;

    debug_log!("Saved chat entry to database with session: {}", session_id);

    Ok(())
}

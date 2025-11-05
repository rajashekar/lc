//! Image generation commands

use anyhow::Result;
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Handle image generation command
pub async fn handle(
    prompt: Vec<String>,
    model: Option<String>,
    provider: Option<String>,
    size: Option<String>,
    count: Option<u32>,
    output: Option<String>,
    debug: bool,
) -> Result<()> {
    // Set debug mode if requested
    if debug {
        crate::utils::cli_utils::set_debug_mode(true);
    }

    // Join prompt parts into a single string
    let prompt_str = prompt.join(" ");
    if prompt_str.is_empty() {
        anyhow::bail!("No prompt provided for image generation");
    }

    let config = crate::config::Config::load()?;

    // Default values
    let size_str = size.unwrap_or_else(|| "1024x1024".to_string());
    let count_val = count.unwrap_or(1);

    // Resolve provider and model using the same logic as other commands
    let (provider_name, model_name) =
        crate::utils::cli_utils::resolve_model_and_provider(&config, provider, model)?;

    // Get provider config with authentication from centralized keys
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    // Allow either API key or resolved custom auth headers
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
    let client =
        crate::core::chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    println!(
        "{} Generating {} image(s) with prompt: \"{}\"",
        "🎨".blue(),
        count_val,
        prompt_str
    );
    println!("{} Model: {}", "🤖".blue(), model_name);
    println!("{} Provider: {}", "🏭".blue(), provider_name);
    println!("{} Size: {}", "📐".blue(), size_str);

    // Create image generation request
    let image_request = crate::core::provider::ImageGenerationRequest {
        prompt: prompt_str.clone(),
        model: Some(model_name.clone()),
        n: Some(count_val),
        size: Some(size_str.clone()),
        quality: Some("standard".to_string()),
        style: None,
        response_format: Some("url".to_string()),
    };

    // Generate images
    print!("{} ", "Generating...".dimmed());
    io::stdout().flush()?;

    match client.generate_images(&image_request).await {
        Ok(response) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Generating..."
            println!(
                "{} Successfully generated {} image(s)!",
                "✅".green(),
                response.data.len()
            );

            // Create output directory if specified
            let output_dir = if let Some(dir) = output {
                let path = Path::new(&dir);
                if !path.exists() {
                    fs::create_dir_all(path)?;
                    println!("{} Created output directory: {}", "📁".blue(), dir);
                }
                Some(dir)
            } else {
                None
            };

            // Process each generated image
            for (i, image_data) in response.data.iter().enumerate() {
                let image_num = i + 1;

                if let Some(url) = &image_data.url {
                    println!(
                        "\n{} Image {}/{}",
                        "🖼️".blue(),
                        image_num,
                        response.data.len()
                    );
                    println!("   URL: {}", url);

                    if let Some(revised_prompt) = &image_data.revised_prompt {
                        if revised_prompt != &prompt_str {
                            println!("   Revised prompt: {}", revised_prompt.dimmed());
                        }
                    }

                    // Download image if output directory is specified
                    if let Some(ref dir) = output_dir {
                        let filename = format!(
                            "image_{}_{}.png",
                            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                            image_num
                        );
                        let filepath = Path::new(dir).join(&filename);

                        match download_image(url, &filepath).await {
                            Ok(_) => {
                                println!("   {} Saved to: {}", "💾".green(), filepath.display());
                            }
                            Err(e) => {
                                eprintln!("   {} Failed to download image: {}", "❌".red(), e);
                            }
                        }
                    }
                } else if let Some(b64_data) = &image_data.b64_json {
                    println!(
                        "\n{} Image {}/{} (Base64)",
                        "🖼️".blue(),
                        image_num,
                        response.data.len()
                    );

                    // For base64 data, always save to a file (either specified output dir or current dir)
                    let save_dir = output_dir.as_deref().unwrap_or(".");
                    let filename = format!(
                        "image_{}_{}.png",
                        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                        image_num
                    );
                    let filepath = Path::new(save_dir).join(&filename);

                    match save_base64_image(b64_data, &filepath) {
                        Ok(_) => {
                            println!("   {} Saved to: {}", "💾".green(), filepath.display());
                        }
                        Err(e) => {
                            eprintln!("   {} Failed to save image: {}", "❌".red(), e);
                        }
                    }

                    if let Some(revised_prompt) = &image_data.revised_prompt {
                        if revised_prompt != &prompt_str {
                            println!("   Revised prompt: {}", revised_prompt.dimmed());
                        }
                    }
                }
            }

            if output_dir.is_none() {
                // Check if we had any URL-based images that weren't downloaded
                let has_url_images = response.data.iter().any(|img| img.url.is_some());
                if has_url_images {
                    println!(
                        "\n{} Use --output <directory> to automatically download URL-based images",
                        "💡".yellow()
                    );
                }
            }
        }
        Err(e) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Generating..."
            anyhow::bail!("Failed to generate images: {}", e);
        }
    }

    Ok(())
}

// Helper function to download image from URL
async fn download_image(url: &str, filepath: &std::path::Path) -> Result<()> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download image: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;
    std::fs::write(filepath, bytes)?;

    Ok(())
}

// Helper function to save base64 image data
fn save_base64_image(b64_data: &str, filepath: &std::path::Path) -> Result<()> {
    use base64::{engine::general_purpose, Engine as _};

    let image_bytes = general_purpose::STANDARD.decode(b64_data)?;
    std::fs::write(filepath, image_bytes)?;

    Ok(())
}

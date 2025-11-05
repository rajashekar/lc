//! Audio processing commands (transcribe, TTS)

use anyhow::Result;
use colored::*;
use std::io::{self, Write};

/// Handle transcribe command
pub async fn handle_transcribe(
    audio_files: Vec<String>,
    model: Option<String>,
    provider: Option<String>,
    language: Option<String>,
    prompt: Option<String>,
    format: Option<String>,
    temperature: Option<f32>,
    output: Option<String>,
    debug: bool,
) -> Result<()> {
    // Set debug mode if requested
    if debug {
        crate::utils::cli_utils::set_debug_mode(true);
    }

    if audio_files.is_empty() {
        anyhow::bail!("No audio files provided for transcription");
    }

    let config = crate::config::Config::load()?;

    // Default to whisper-1 model if not specified
    let model_str = model.unwrap_or_else(|| "whisper-1".to_string());
    let format_str = format.unwrap_or_else(|| "text".to_string());

    // Resolve provider and model
    let (provider_name, model_name) = if let Some(p) = provider {
        (p, model_str)
    } else {
        // Try to find a provider that has the whisper model
        let provider_name = config
            .providers
            .iter()
            .find(|(_, pc)| pc.models.iter().any(|m| m.contains("whisper")))
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "openai".to_string());
        (provider_name, model_str)
    };

    // Get provider config with authentication
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    // Check for API key or custom auth headers
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
        "{} Transcribing {} audio file(s)",
        "🎤".blue(),
        audio_files.len()
    );
    println!("{} Model: {}", "🤖".blue(), model_name);
    println!("{} Provider: {}", "🏭".blue(), provider_name);
    if let Some(ref lang) = language {
        println!("{} Language: {}", "🌐".blue(), lang);
    }
    println!("{} Format: {}", "📄".blue(), format_str);

    let mut all_transcriptions = Vec::new();

    for (i, audio_file) in audio_files.iter().enumerate() {
        println!(
            "\n{} Processing file {}/{}: {}",
            "📁".blue(),
            i + 1,
            audio_files.len(),
            audio_file
        );

        print!("{} ", "Transcribing...".dimmed());
        io::stdout().flush()?;

        // Process audio file (handles both local files and URLs)
        let audio_data = if audio_file.starts_with("http://") || audio_file.starts_with("https://")
        {
            crate::utils::audio::process_audio_url(audio_file)?
        } else {
            crate::utils::audio::process_audio_file(std::path::Path::new(audio_file))?
        };

        // Create transcription request
        let transcription_request = crate::core::provider::AudioTranscriptionRequest {
            file: audio_data,
            model: model_name.clone(),
            language: language.clone(),
            prompt: prompt.clone(),
            response_format: Some(format_str.clone()),
            temperature,
        };

        // Transcribe audio
        match client.transcribe_audio(&transcription_request).await {
            Ok(response) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Transcribing..."
                println!("{} Transcription complete!", "✅".green());

                // Display or save transcription
                let transcription_text = response.text;

                if let Some(ref output_file) = output {
                    // Append to output file if multiple files
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output_file)?;

                    if audio_files.len() > 1 {
                        writeln!(file, "\n=== {} ===", audio_file)?;
                    }
                    writeln!(file, "{}", transcription_text)?;

                    all_transcriptions.push(transcription_text);
                } else {
                    // Print to stdout
                    if audio_files.len() > 1 {
                        println!("\n{} Transcription for {}:", "📝".blue(), audio_file);
                    } else {
                        println!("\n{} Transcription:", "📝".blue());
                    }
                    println!("{}", transcription_text);

                    all_transcriptions.push(transcription_text);
                }
            }
            Err(e) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Transcribing..."
                eprintln!("{} Failed to transcribe {}: {}", "❌".red(), audio_file, e);
            }
        }
    }

    if let Some(output_file) = output {
        println!(
            "\n{} All transcriptions saved to: {}",
            "💾".green(),
            output_file
        );
    }

    Ok(())
}

/// Handle TTS (text-to-speech) command
pub async fn handle_tts(
    text: String,
    model: Option<String>,
    provider: Option<String>,
    voice: Option<String>,
    format: Option<String>,
    speed: Option<f32>,
    output: Option<String>,
    debug: bool,
) -> Result<()> {
    // Set debug mode if requested
    if debug {
        crate::utils::cli_utils::set_debug_mode(true);
    }

    let config = crate::config::Config::load()?;

    // Default to tts-1 model if not specified
    let model_str = model.unwrap_or_else(|| "tts-1".to_string());
    let voice_str = voice.unwrap_or_else(|| "alloy".to_string());
    let format_str = format.unwrap_or_else(|| "mp3".to_string());

    // Generate default output filename
    let output_path = output.unwrap_or_else(|| {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        format!("speech_{}.{}", timestamp, format_str)
    });

    // Resolve provider and model
    let (provider_name, model_name) = if let Some(p) = provider {
        (p, model_str)
    } else {
        // Try to find a provider that has TTS models
        let provider_name = config
            .providers
            .iter()
            .find(|(_, pc)| pc.models.iter().any(|m| m.contains("tts")))
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "openai".to_string());
        (provider_name, model_str)
    };

    // Get provider config with authentication
    let provider_config = config.get_provider_with_auth(&provider_name)?;

    // Check for API key or custom auth headers
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

    // Truncate text for display if it's too long
    let display_text = if text.len() > 100 {
        format!("{}...", &text[..100])
    } else {
        text.clone()
    };

    println!("{} Generating speech", "🔊".blue());
    println!("{} Text: \"{}\"", "📝".blue(), display_text);
    println!("{} Model: {}", "🤖".blue(), model_name);
    println!("{} Provider: {}", "🏭".blue(), provider_name);
    println!("{} Voice: {}", "🎭".blue(), voice_str);
    println!("{} Format: {}", "🎵".blue(), format_str);
    if let Some(s) = speed {
        println!("{} Speed: {}x", "⚡".blue(), s);
    }

    print!("{} ", "Generating speech...".dimmed());
    io::stdout().flush()?;

    // Create TTS request
    let tts_request = crate::core::provider::AudioSpeechRequest {
        model: model_name,
        input: text,
        voice: voice_str,
        response_format: Some(format_str.clone()),
        speed,
    };

    // Generate speech
    match client.generate_speech(&tts_request).await {
        Ok(audio_bytes) => {
            print!("\r{}\r", " ".repeat(25)); // Clear "Generating speech..."

            // Determine the appropriate file extension and format
            let detected_extension =
                crate::utils::audio::get_audio_file_extension(&audio_bytes, Some(&format_str));
            let is_pcm_conversion_needed = crate::utils::audio::is_likely_pcm(&audio_bytes)
                || format_str.to_lowercase() == "pcm";

            // Process audio data for better compatibility
            let (final_audio_data, final_extension, conversion_info) = if is_pcm_conversion_needed {
                // Convert PCM to WAV for better playability
                let wav_data = crate::utils::audio::pcm_to_wav(&audio_bytes, None, None, None);
                (
                    wav_data,
                    "wav",
                    Some("Converted PCM to WAV for better compatibility"),
                )
            } else {
                (audio_bytes, detected_extension, None)
            };

            // Determine final output filename
            let final_output = if output_path.ends_with(&format!(".{}", final_extension)) {
                output_path
            } else {
                // Replace or add the correct extension
                let path = std::path::Path::new(&output_path);
                if let Some(stem) = path.file_stem() {
                    if let Some(parent) = path.parent() {
                        parent
                            .join(format!("{}.{}", stem.to_string_lossy(), final_extension))
                            .to_string_lossy()
                            .to_string()
                    } else {
                        format!("{}.{}", stem.to_string_lossy(), final_extension)
                    }
                } else {
                    format!("{}.{}", output_path, final_extension)
                }
            };

            // Save audio to file
            std::fs::write(&final_output, &final_audio_data)?;

            println!("{} Speech generated successfully!", "✅".green());
            println!("{} Saved to: {}", "💾".green(), final_output);

            // Show conversion info if applicable
            if let Some(info) = conversion_info {
                println!("{} {}", "🔄".blue(), info);
            }

            // Show file size
            let metadata = std::fs::metadata(&final_output)?;
            let size_kb = metadata.len() as f64 / 1024.0;
            println!("{} File size: {:.2} KB", "📊".blue(), size_kb);

            // Show format info
            println!(
                "{} Format: {} ({})",
                "🎵".blue(),
                final_extension.to_uppercase(),
                if is_pcm_conversion_needed {
                    "24kHz, 16-bit, Mono"
                } else {
                    "Original format"
                }
            );
        }
        Err(e) => {
            print!("\r{}\r", " ".repeat(25)); // Clear "Generating speech..."
            anyhow::bail!("Failed to generate speech: {}", e);
        }
    }

    Ok(())
}

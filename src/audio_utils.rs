use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::path::Path;

/// Supported audio formats
#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    Mp3,
    Mp4,
    Mpeg,
    Mpga,
    M4a,
    Wav,
    Webm,
    Ogg,
    Flac,
}

impl AudioFormat {
    /// Detect audio format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "mp4" => Some(AudioFormat::Mp4),
            "mpeg" => Some(AudioFormat::Mpeg),
            "mpga" => Some(AudioFormat::Mpga),
            "m4a" => Some(AudioFormat::M4a),
            "wav" => Some(AudioFormat::Wav),
            "webm" => Some(AudioFormat::Webm),
            "ogg" => Some(AudioFormat::Ogg),
            "flac" => Some(AudioFormat::Flac),
            _ => None,
        }
    }

    /// Get MIME type for the audio format
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Mp4 => "audio/mp4",
            AudioFormat::Mpeg => "audio/mpeg",
            AudioFormat::Mpga => "audio/mpeg",
            AudioFormat::M4a => "audio/m4a",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Webm => "audio/webm",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Flac => "audio/flac",
        }
    }
}

/// Process an audio file and return a data URL for API consumption
pub fn process_audio_file(path: &Path) -> Result<String> {
    // Check if file exists
    if !path.exists() {
        anyhow::bail!("Audio file not found: {}", path.display());
    }

    // Detect audio format from extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| anyhow::anyhow!("No file extension found"))?;

    let format = AudioFormat::from_extension(extension)
        .ok_or_else(|| anyhow::anyhow!("Unsupported audio format: {}", extension))?;

    // Read the audio file
    let audio_data = fs::read(path)?;

    // Check file size (limit to 25MB for most providers)
    const MAX_SIZE: usize = 25 * 1024 * 1024; // 25MB
    if audio_data.len() > MAX_SIZE {
        anyhow::bail!(
            "Audio file too large: {} bytes (max: {} bytes)",
            audio_data.len(),
            MAX_SIZE
        );
    }

    // Encode to base64
    let base64_data = general_purpose::STANDARD.encode(&audio_data);

    // Create data URL
    let data_url = format!("data:{};base64,{}", format.mime_type(), base64_data);

    Ok(data_url)
}

/// Process an audio from a URL
pub fn process_audio_url(url: &str) -> Result<String> {
    // For now, just validate and return the URL
    // In the future, we could download and process the audio
    if !url.starts_with("http://") && !url.starts_with("https://") {
        anyhow::bail!("Invalid audio URL: must start with http:// or https://");
    }

    Ok(url.to_string())
}

/// Process multiple audio inputs (files or URLs)
pub fn process_audio_files(paths: &[String]) -> Result<Vec<String>> {
    let mut processed_audio = Vec::new();

    for path_str in paths {
        let processed = if path_str.starts_with("http://") || path_str.starts_with("https://") {
            process_audio_url(path_str)?
        } else {
            let path = Path::new(path_str);
            process_audio_file(path)?
        };

        processed_audio.push(processed);
    }

    Ok(processed_audio)
}

/// Download audio from URL and save to file
pub async fn download_audio(url: &str, output_path: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to download audio: HTTP {}", response.status());
    }
    
    let bytes = response.bytes().await?;
    fs::write(output_path, bytes)?;
    
    Ok(())
}

/// Save base64 audio data to file
pub fn save_base64_audio(b64_data: &str, filepath: &Path) -> Result<()> {
    let audio_bytes = general_purpose::STANDARD.decode(b64_data)?;
    fs::write(filepath, audio_bytes)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_detection() {
        assert!(matches!(
            AudioFormat::from_extension("mp3"),
            Some(AudioFormat::Mp3)
        ));
        assert!(matches!(
            AudioFormat::from_extension("WAV"),
            Some(AudioFormat::Wav)
        ));
        assert!(matches!(
            AudioFormat::from_extension("ogg"),
            Some(AudioFormat::Ogg)
        ));
        assert!(matches!(
            AudioFormat::from_extension("flac"),
            Some(AudioFormat::Flac)
        ));
        assert!(AudioFormat::from_extension("txt").is_none());
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
        assert_eq!(AudioFormat::Ogg.mime_type(), "audio/ogg");
        assert_eq!(AudioFormat::Flac.mime_type(), "audio/flac");
    }
}
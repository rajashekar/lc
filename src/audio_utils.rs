use anyhow::Result;
use std::path::Path;

/// Audio file data with metadata
#[allow(dead_code)]
pub struct AudioData {
    pub data: String, // Base64 encoded audio data
    pub filename: String,
    pub mime_type: String,
}

/// Process an audio file and return base64 encoded data
pub fn process_audio_file(file_path: &Path) -> Result<String> {
    let audio_bytes = std::fs::read(file_path)?;
    
    // Encode to base64
    use base64::{engine::general_purpose, Engine as _};
    let base64_data = general_purpose::STANDARD.encode(&audio_bytes);
    
    // Create data URL with appropriate MIME type based on file extension
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let mime_type = match extension.as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "m4a" | "mp4" => "audio/mp4",
        "webm" => "audio/webm",
        _ => "audio/wav", // Default to WAV
    };
    
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

/// Process an audio URL and return base64 encoded data
pub fn process_audio_url(url: &str) -> Result<String> {
    // For now, just return the URL as-is
    // In a full implementation, you might want to download and encode the audio
    Ok(url.to_string())
}

/// Generate a WAV header for PCM audio data
/// 
/// This creates a standard WAV file header for 16-bit PCM audio.
/// Based on Gemini's audio format: 16-bit little-endian PCM at 24kHz sample rate, mono channel
pub fn generate_wav_header(data_size: u32, sample_rate: u32, channels: u16, bits_per_sample: u16) -> Vec<u8> {
    let mut header = Vec::with_capacity(44);
    
    // RIFF header
    header.extend_from_slice(b"RIFF");
    
    // File size - 8 bytes (will be filled in later)
    let file_size = 36 + data_size;
    header.extend_from_slice(&file_size.to_le_bytes());
    
    // WAVE format
    header.extend_from_slice(b"WAVE");
    
    // fmt subchunk
    header.extend_from_slice(b"fmt ");
    
    // Subchunk1 size (16 for PCM)
    header.extend_from_slice(&16u32.to_le_bytes());
    
    // Audio format (1 for PCM)
    header.extend_from_slice(&1u16.to_le_bytes());
    
    // Number of channels
    header.extend_from_slice(&channels.to_le_bytes());
    
    // Sample rate
    header.extend_from_slice(&sample_rate.to_le_bytes());
    
    // Byte rate (sample_rate * channels * bits_per_sample / 8)
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    header.extend_from_slice(&byte_rate.to_le_bytes());
    
    // Block align (channels * bits_per_sample / 8)
    let block_align = channels * bits_per_sample / 8;
    header.extend_from_slice(&block_align.to_le_bytes());
    
    // Bits per sample
    header.extend_from_slice(&bits_per_sample.to_le_bytes());
    
    // data subchunk
    header.extend_from_slice(b"data");
    
    // Data size
    header.extend_from_slice(&data_size.to_le_bytes());
    
    header
}

/// Convert PCM audio data to WAV format
/// 
/// This function takes raw PCM audio data and wraps it with a proper WAV header
/// to make it playable by standard media players.
/// 
/// Default parameters are based on Gemini's audio format:
/// - 24kHz sample rate
/// - 16-bit depth
/// - Mono channel
pub fn pcm_to_wav(pcm_data: &[u8], sample_rate: Option<u32>, channels: Option<u16>, bits_per_sample: Option<u16>) -> Vec<u8> {
    let sample_rate = sample_rate.unwrap_or(24000); // Default to 24kHz (Gemini's format)
    let channels = channels.unwrap_or(1); // Default to mono
    let bits_per_sample = bits_per_sample.unwrap_or(16); // Default to 16-bit
    
    let data_size = pcm_data.len() as u32;
    let header = generate_wav_header(data_size, sample_rate, channels, bits_per_sample);
    
    let mut wav_data = Vec::with_capacity(header.len() + pcm_data.len());
    wav_data.extend_from_slice(&header);
    wav_data.extend_from_slice(pcm_data);
    
    wav_data
}

/// Detect if audio data is likely PCM format
/// 
/// This is a heuristic check - PCM data typically doesn't have recognizable headers
/// and the data should be relatively uniform in distribution.
pub fn is_likely_pcm(data: &[u8]) -> bool {
    // Check if it's not a known audio format by looking for headers
    if data.len() < 4 {
        return false;
    }
    
    // Check for common audio format headers
    let header = &data[0..4];
    
    // WAV files start with "RIFF"
    if header == b"RIFF" {
        return false;
    }
    
    // MP3 files often start with ID3 tag or sync frame
    if header[0..3] == [0x49, 0x44, 0x33] || // ID3
       (header[0] == 0xFF && (header[1] & 0xE0) == 0xE0) { // MP3 sync frame
        return false;
    }
    
    // FLAC files start with "fLaC"
    if header == b"fLaC" {
        return false;
    }
    
    // OGG files start with "OggS"
    if header == b"OggS" {
        return false;
    }
    
    // If we don't recognize the format and the data size is reasonable for audio, assume PCM
    true
}

/// Get the appropriate file extension based on the detected or specified format
pub fn get_audio_file_extension(data: &[u8], requested_format: Option<&str>) -> &'static str {
    // If a specific format was requested, use that
    if let Some(format) = requested_format {
        return match format.to_lowercase().as_str() {
            "mp3" => "mp3",
            "wav" => "wav",
            "flac" => "flac",
            "ogg" => "ogg",
            "aac" => "aac",
            "opus" => "opus",
            "pcm" => "wav", // Convert PCM to WAV for better compatibility
            _ => "wav", // Default to WAV for unknown formats
        };
    }
    
    // Auto-detect based on data
    if is_likely_pcm(data) {
        "wav" // Convert PCM to WAV
    } else {
        // Try to detect format from header
        if data.len() >= 4 {
            let header = &data[0..4];
            if header == b"RIFF" {
                "wav"
            } else if header[0..3] == [0x49, 0x44, 0x33] || 
                     (header[0] == 0xFF && (header[1] & 0xE0) == 0xE0) {
                "mp3"
            } else if header == b"fLaC" {
                "flac"
            } else if header == b"OggS" {
                "ogg"
            } else {
                "wav" // Default to WAV
            }
        } else {
            "wav" // Default to WAV
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_header_generation() {
        let header = generate_wav_header(1000, 44100, 2, 16);
        assert_eq!(header.len(), 44);
        assert_eq!(&header[0..4], b"RIFF");
        assert_eq!(&header[8..12], b"WAVE");
        assert_eq!(&header[12..16], b"fmt ");
    }

    #[test]
    fn test_pcm_to_wav_conversion() {
        let pcm_data = vec![0u8; 1000]; // 1000 bytes of silence
        let wav_data = pcm_to_wav(&pcm_data, Some(44100), Some(2), Some(16));
        
        // Should have WAV header (44 bytes) + PCM data
        assert_eq!(wav_data.len(), 44 + 1000);
        assert_eq!(&wav_data[0..4], b"RIFF");
        assert_eq!(&wav_data[8..12], b"WAVE");
    }

    #[test]
    fn test_pcm_detection() {
        // Test with WAV header - should not be detected as PCM
        let wav_header = b"RIFF\x24\x08\x00\x00WAVE";
        assert!(!is_likely_pcm(wav_header));
        
        // Test with random data - should be detected as PCM
        let pcm_data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        assert!(is_likely_pcm(&pcm_data));
    }

    #[test]
    fn test_file_extension_detection() {
        // Test PCM detection
        let pcm_data = vec![0x12, 0x34, 0x56, 0x78];
        assert_eq!(get_audio_file_extension(&pcm_data, None), "wav");
        
        // Test WAV detection
        let wav_data = b"RIFF\x24\x08\x00\x00WAVE";
        assert_eq!(get_audio_file_extension(wav_data, None), "wav");
        
        // Test requested format override
        assert_eq!(get_audio_file_extension(&pcm_data, Some("mp3")), "mp3");
        assert_eq!(get_audio_file_extension(&pcm_data, Some("pcm")), "wav");
    }
}
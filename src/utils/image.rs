use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::path::Path;

/// Supported image formats
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
}

impl ImageFormat {
    /// Detect image format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "png" => Some(ImageFormat::Png),
            "gif" => Some(ImageFormat::Gif),
            "webp" => Some(ImageFormat::WebP),
            _ => None,
        }
    }

    /// Get MIME type for the image format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::Gif => "image/gif",
            ImageFormat::WebP => "image/webp",
        }
    }
}

/// Process an image file and return a data URL
pub fn process_image_file(path: &Path) -> Result<String> {
    // Check if file exists
    if !path.exists() {
        anyhow::bail!("Image file not found: {}", path.display());
    }

    // Detect image format from extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| anyhow::anyhow!("No file extension found"))?;

    let format = ImageFormat::from_extension(extension)
        .ok_or_else(|| anyhow::anyhow!("Unsupported image format: {}", extension))?;

    // Read the image file
    let image_data = fs::read(path)?;

    // Check file size (limit to 20MB for most providers)
    const MAX_SIZE: usize = 20 * 1024 * 1024; // 20MB
    if image_data.len() > MAX_SIZE {
        anyhow::bail!(
            "Image file too large: {} bytes (max: {} bytes)",
            image_data.len(),
            MAX_SIZE
        );
    }

    // Encode to base64
    let base64_data = general_purpose::STANDARD.encode(&image_data);

    // Create data URL
    let data_url = format!("data:{};base64,{}", format.mime_type(), base64_data);

    Ok(data_url)
}

/// Process an image from a URL
pub fn process_image_url(url: &str) -> Result<String> {
    // For now, just validate and return the URL
    // In the future, we could download and process the image
    if !url.starts_with("http://") && !url.starts_with("https://") {
        anyhow::bail!("Invalid image URL: must start with http:// or https://");
    }

    Ok(url.to_string())
}

/// Process multiple image inputs (files or URLs)
pub fn process_images(paths: &[String]) -> Result<Vec<String>> {
    let mut processed_images = Vec::new();

    for path_str in paths {
        let processed = if path_str.starts_with("http://") || path_str.starts_with("https://") {
            process_image_url(path_str)?
        } else {
            let path = Path::new(path_str);
            process_image_file(path)?
        };

        processed_images.push(processed);
    }

    Ok(processed_images)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_detection() {
        assert!(matches!(
            ImageFormat::from_extension("jpg"),
            Some(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            ImageFormat::from_extension("JPEG"),
            Some(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            ImageFormat::from_extension("png"),
            Some(ImageFormat::Png)
        ));
        assert!(matches!(
            ImageFormat::from_extension("gif"),
            Some(ImageFormat::Gif)
        ));
        assert!(matches!(
            ImageFormat::from_extension("webp"),
            Some(ImageFormat::WebP)
        ));
        assert!(ImageFormat::from_extension("txt").is_none());
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Gif.mime_type(), "image/gif");
        assert_eq!(ImageFormat::WebP.mime_type(), "image/webp");
    }
}

use super::FileReader;
use anyhow::{Context, Result};
use std::io::Read;

#[cfg(feature = "pdf")]
extern crate pdf_extract;

pub struct PdfReader;

impl PdfReader {
    pub fn new() -> Self {
        Self
    }

    /// Internal method to extract text from PDF bytes with comprehensive error handling
    fn extract_text_from_bytes_internal(&self, bytes: &[u8]) -> Result<String> {
        #[cfg(feature = "pdf")]
        {
            // Attempt to extract text using pdf-extract
            match pdf_extract::extract_text_from_mem(bytes) {
                Ok(text) => {
                    // Check if the extracted text is mostly empty or contains only whitespace
                    let cleaned_text = text.trim();
                    if cleaned_text.is_empty() {
                        // This might be a bitmap-only PDF
                        return Ok("[image page]".to_string());
                    }

                    // Preserve page breaks by converting form feed characters
                    let formatted_text = text.replace('\x0C', "\u{000C}");

                    // Ensure UTF-8 encoding
                    Ok(formatted_text)
                }
                Err(e) => {
                    // Check if this might be an encrypted PDF
                    let error_msg = e.to_string().to_lowercase();
                    if error_msg.contains("encrypt")
                        || error_msg.contains("password")
                        || error_msg.contains("security")
                    {
                        // Try passwordless decryption attempt (pdf-extract handles this internally)
                        // If it fails, return appropriate error
                        Err(anyhow::anyhow!(
                            "PDF appears to be encrypted and requires a password for text extraction. \
                            Error: {}", e
                        ))
                    } else {
                        // Check if this might be a bitmap-only PDF
                        if error_msg.contains("no text")
                            || error_msg.contains("image")
                            || error_msg.contains("scan")
                        {
                            Ok("[image page]".to_string())
                        } else {
                            Err(anyhow::anyhow!("Failed to extract text from PDF: {}", e))
                        }
                    }
                }
            }
        }
        #[cfg(not(feature = "pdf"))]
        {
            let _ = bytes; // Suppress unused parameter warning
            Err(anyhow::anyhow!(
                "PDF support is not enabled. Please compile with the 'pdf' feature flag to enable PDF processing."
            ))
        }
    }
}

impl FileReader for PdfReader {
    fn read_as_text(&self, file_path: &str) -> Result<String> {
        let bytes = std::fs::read(file_path)
            .with_context(|| format!("Failed to read PDF file: {}", file_path))?;

        self.read_as_text_from_bytes(&bytes)
            .with_context(|| format!("Failed to extract text from PDF file: {}", file_path))
    }

    fn read_as_text_from_bytes(&self, bytes: &[u8]) -> Result<String> {
        self.extract_text_from_bytes_internal(bytes)
    }

    fn read_as_text_from_reader(&self, mut reader: Box<dyn Read>) -> Result<String> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .with_context(|| "Failed to read bytes from reader")?;

        self.read_as_text_from_bytes(&bytes)
    }

    fn can_handle(&self, extension: &str) -> bool {
        extension.to_lowercase() == "pdf"
    }
}

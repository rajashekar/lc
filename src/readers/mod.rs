#[cfg(feature = "pdf")]
pub mod pdf;

use anyhow::{Context, Result};
use std::io::Read;

/// Trait for reading different file types
pub trait FileReader {
    /// Read file content as text from a file path
    fn read_as_text(&self, file_path: &str) -> Result<String>;
    
    /// Read file content as text from bytes
    fn read_as_text_from_bytes(&self, _bytes: &[u8]) -> Result<String> {
        // Default implementation: not supported
        Err(anyhow::anyhow!("Reading from bytes not supported by this file reader"))
    }
    
    /// Read file content as text from a readable stream
    fn read_as_text_from_reader(&self, mut reader: Box<dyn Read>) -> Result<String> {
        // Default implementation: read all bytes and use bytes method
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)
            .with_context(|| "Failed to read bytes from reader")?;
        self.read_as_text_from_bytes(&bytes)
    }
    
    /// Check if this reader can handle the given file extension
    fn can_handle(&self, extension: &str) -> bool;
}

/// Get appropriate reader for file extension
pub fn get_reader_for_extension(extension: &str) -> Option<Box<dyn FileReader>> {
    match extension.to_lowercase().as_str() {
        #[cfg(feature = "pdf")]
        "pdf" => Some(Box::new(pdf::PdfReader::new())),
        _ => None,
    }
}

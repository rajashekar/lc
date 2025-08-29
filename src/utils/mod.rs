// Utility modules
pub mod audio;
pub mod cli_utils;
pub mod image;
pub mod token;
pub mod input;
pub mod test;
pub mod template_processor;

// Re-export with old names for compatibility
pub use audio as audio_utils;
pub use image as image_utils;
pub use token as token_utils;
pub use test as test_utils;

// Re-export CLI utilities for tests
pub use cli_utils::{is_code_file, read_and_format_attachments, resolve_model_and_provider, set_debug_mode};
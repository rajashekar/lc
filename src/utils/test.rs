//! Test utilities for managing test providers and cleanup
//!
//! This module provides utilities to ensure test providers use the "test-" prefix
//! and are automatically cleaned up after tests complete.

use std::fs;
use std::sync::Once;

/// Prefix for test providers to avoid conflicts with real configurations
pub const TEST_PROVIDER_PREFIX: &str = "test-";

static INIT: Once = Once::new();

/// Initialize test environment - call this once at the start of test runs
pub fn init_test_env() {
    INIT.call_once(|| {
        cleanup_test_providers().unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to clean up test providers during init: {}",
                e
            );
        });
    });
}

/// Clean up test providers from the configuration directory
pub fn cleanup_test_providers() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let config_dir = home_dir.join("Library/Application Support/lc/providers");

    if !config_dir.exists() {
        return Ok(());
    }

    let mut cleaned_count = 0;

    // Read directory and remove any files that start with test- prefix
    for entry in fs::read_dir(&config_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if file_name_str.starts_with(TEST_PROVIDER_PREFIX) {
            let file_path = entry.path();
            if file_path.is_file() {
                fs::remove_file(&file_path)?;
                cleaned_count += 1;
            }
        }
    }

    if cleaned_count > 0 {
        println!("Cleaned up {} test provider files", cleaned_count);
    }

    Ok(())
}

/// Get test provider name with prefix
pub fn get_test_provider_name(base_name: &str) -> String {
    format!("{}{}", TEST_PROVIDER_PREFIX, base_name)
}

/// RAII guard for test cleanup
pub struct TestGuard;

impl TestGuard {
    pub fn new() -> Self {
        init_test_env();
        TestGuard
    }
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        if let Err(e) = cleanup_test_providers() {
            eprintln!("Warning: Failed to clean up test providers: {}", e);
        }
    }
}

/// Macro to wrap test functions with automatic setup and cleanup
#[macro_export]
macro_rules! test_with_cleanup {
    ($test_body:block) => {{
        let _guard = $crate::test_utils::TestGuard::new();
        $test_body
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name_generation() {
        assert_eq!(get_test_provider_name("openai"), "test-openai");
        assert_eq!(get_test_provider_name("anthropic"), "test-anthropic");
        assert_eq!(
            get_test_provider_name("custom-provider"),
            "test-custom-provider"
        );
    }

    #[test]
    fn test_cleanup_function() {
        // Test that cleanup function can be called without errors
        let result = cleanup_test_providers();
        assert!(result.is_ok());
    }

    #[test]
    fn test_guard_creation() {
        let _guard = TestGuard::new();
        // If we get here, the guard was created successfully
    }
}

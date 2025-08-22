//! Test to verify configuration directory isolation during tests
//!
//! This test ensures that when running tests, the configuration
//! is stored in a temporary directory instead of the production
//! configuration directory.

mod common;

use common::TestEnvironment;
use lc::config::Config;
use std::fs;

#[test]
fn test_config_directory_isolation() {
    // Get the config directory being used
    let config_dir = Config::config_dir().expect("Failed to get config directory");
    let config_path_str = config_dir.to_string_lossy();
    
    // Verify we're using a test directory in temp location
    assert!(
        config_path_str.contains("lc_test") ||
        config_path_str.contains("tmp") ||
        config_path_str.contains("temp") ||
        config_path_str.contains("Temp"),
        "Config directory should be in temp location for tests: {}",
        config_path_str
    );
    
    // Verify it's not the production directory
    #[cfg(target_os = "macos")]
    assert!(
        !config_path_str.contains("Library/Application Support/lc") ||
        config_path_str.contains("test"),
        "Should not use production config directory in tests: {}",
        config_path_str
    );
    
    #[cfg(target_os = "linux")]
    assert!(
        !config_path_str.contains(".local/share/lc") ||
        config_path_str.contains("test"),
        "Should not use production config directory in tests: {}",
        config_path_str
    );
    
    #[cfg(target_os = "windows")]
    assert!(
        !config_path_str.contains("AppData\\Local\\lc") ||
        config_path_str.contains("Temp"),
        "Should not use production config directory in tests: {}",
        config_path_str
    );
    
    // Test that we can create config files in the test directory
    let mut config = Config::load().expect("Failed to load config");
    
    // Add a test provider (this will save the provider file immediately)
    config.add_provider(
        "test-provider".to_string(),
        "https://api.test.com".to_string()
    ).expect("Failed to add provider");
    
    // Verify config files were created in test directory
    let config_file = config_dir.join("config.toml");
    assert!(config_file.exists(), "Config file should exist in test directory");
    
    let providers_dir = config_dir.join("providers");
    assert!(providers_dir.exists(), "Providers directory should exist in test directory");
    
    // Verify the provider file was created by add_provider
    let provider_file = providers_dir.join("test-provider.toml");
    assert!(provider_file.exists(), "Provider file should exist in test directory");
}

#[test]
fn test_automatic_test_isolation() {
    // No need to set up environment - it should be automatic
    let config_dir = Config::config_dir().expect("Failed to get config directory");
    
    // Use the test environment helper to verify isolation
    let test_env = TestEnvironment::new();
    test_env.verify_test_isolation();
    
    // Verify the directory matches what the helper found
    assert_eq!(config_dir, test_env.config_dir);
}

#[test]
fn test_config_operations_in_test_environment() {
    // No explicit test environment setup needed - automatic detection handles it
    
    // Load config (should create in test directory)
    let mut config = Config::load().expect("Failed to load config");
    
    // Add a provider
    config.add_provider(
        "test-provider".to_string(),
        "https://api.test.com".to_string()
    ).expect("Failed to add provider");
    
    // Save config
    config.save().expect("Failed to save config");
    
    // Reload config to verify persistence
    let config2 = Config::load().expect("Failed to reload config");
    assert!(config2.has_provider("test-provider"));
    
    // Verify the provider configuration
    let provider = config2.get_provider("test-provider").expect("Provider should exist");
    assert_eq!(provider.endpoint, "https://api.test.com");
}

#[test]
fn test_no_production_config_modification() {
    // This test verifies that production config is never touched
    let config_dir = Config::config_dir().expect("Failed to get config directory");
    let path_str = config_dir.to_string_lossy();
    
    // The path should contain test indicators
    assert!(
        path_str.contains("test") || path_str.contains("tmp") || path_str.contains("temp"),
        "Test should use temporary directory, got: {}",
        path_str
    );
    
    // Create a test file to verify we're in the right place
    let test_marker = config_dir.join("test_marker.txt");
    fs::write(&test_marker, "This is a test file").expect("Failed to write test marker");
    assert!(test_marker.exists());
    
    // Clean up
    fs::remove_file(test_marker).ok();
}
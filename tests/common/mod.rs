//! Common test utilities for LC CLI integration tests
//!
//! This module provides shared functionality for testing CLI commands,
//! including test data creation, temporary environments, and assertion helpers.

use lc::config::{Config, ProviderConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Once;
// Removed unused import: tempfile::TempDir

/// Prefix for test providers to avoid conflicts with real configurations
const TEST_PROVIDER_PREFIX: &str = "test-";

static BUILD_ONCE: Once = Once::new();

/// Test environment setup that verifies test isolation
/// No longer needs to set environment variables as the config module
/// automatically detects test environment
pub struct TestEnvironment {
    pub config_dir: PathBuf,
}

impl TestEnvironment {
    /// Create a new test environment verification helper
    pub fn new() -> Self {
        // Get the actual config directory being used
        let config_dir = lc::config::Config::config_dir()
            .expect("Failed to get config directory");
        
        TestEnvironment {
            config_dir,
        }
    }
    
    /// Get the path to the test configuration directory
    #[allow(dead_code)]
    pub fn config_path(&self) -> &PathBuf {
        &self.config_dir
    }
    
    /// Verify that we're using a test directory, not production
    pub fn verify_test_isolation(&self) {
        let path_str = self.config_dir.to_string_lossy();
        
        // Verify we're in a temp directory
        assert!(
            path_str.contains("lc_test") || path_str.contains("tmp") || path_str.contains("temp"),
            "Config directory should be in temp location for tests: {}",
            path_str
        );
        
        // Verify we're NOT in production directories
        #[cfg(target_os = "macos")]
        assert!(
            !path_str.contains("Library/Application Support/lc"),
            "Should not use production config directory in tests"
        );
        
        #[cfg(target_os = "linux")]
        assert!(
            !path_str.contains(".local/share/lc"),
            "Should not use production config directory in tests"
        );
        
        #[cfg(target_os = "windows")]
        assert!(
            !path_str.contains("AppData") || path_str.contains("Temp"),
            "Should not use production config directory in tests"
        );
    }
}

/// Get the path to the compiled test binary
/// This ensures the binary is built once and returns the correct path for the platform
#[allow(dead_code)]
pub fn get_test_binary_path() -> PathBuf {
    BUILD_ONCE.call_once(|| {
        let output = std::process::Command::new("cargo")
            .args(&["build", "--bin", "lc"])
            .output()
            .expect("Failed to build test binary");
        
        if !output.status.success() {
            panic!(
                "Failed to build test binary: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    });

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("lc");
    
    #[cfg(windows)]
    path.set_extension("exe");
    
    if !path.exists() {
        panic!("Test binary not found at: {:?}", path);
    }
    
    path
}

/// Get test provider name with prefix
#[allow(dead_code)]
pub fn get_test_provider_name(base_name: &str) -> String {
    format!("{}{}", TEST_PROVIDER_PREFIX, base_name)
}

/// Helper function to create a temporary config for testing
#[allow(dead_code)]
pub fn create_test_config() -> Config {
    Config {
        providers: HashMap::new(),
        default_provider: None,
        default_model: None,
        aliases: HashMap::new(),
        system_prompt: None,
        templates: HashMap::new(),
        max_tokens: None,
        temperature: None,
        stream: None,
    }
}

/// Helper function to verify test environment isolation
#[allow(dead_code)]
pub fn verify_test_isolation() {
    let test_env = TestEnvironment::new();
    test_env.verify_test_isolation();
}

/// Helper function to create a test provider config
#[allow(dead_code)]
pub fn create_test_provider_config(endpoint: &str) -> ProviderConfig {
    ProviderConfig {
        endpoint: endpoint.to_string(),
        api_key: None, // Keys are now stored centrally in keys.toml
        models: vec!["test-model-1".to_string(), "test-model-2".to_string()],
        models_path: "/models".to_string(),
        chat_path: "/chat/completions".to_string(),
        images_path: Some("/images/generations".to_string()),
        embeddings_path: Some("/embeddings".to_string()),
        audio_path: Some("/audio/transcriptions".to_string()),
        speech_path: Some("/audio/speech".to_string()),
        headers: HashMap::new(),
        token_url: None,
        cached_token: None,
        auth_type: None,
        vars: std::collections::HashMap::new(),
        chat_templates: None,
        images_templates: None,
        embeddings_templates: None,
        models_templates: None,
        audio_templates: None,
        speech_templates: None,
    }
}

/// Helper function to create a config with test providers using test- prefix
#[allow(dead_code)]
pub fn create_config_with_providers() -> Config {
    let mut config = Config {
        providers: HashMap::new(),
        default_provider: None,
        default_model: None,
        aliases: HashMap::new(),
        system_prompt: None,
        templates: HashMap::new(),
        max_tokens: None,
        temperature: None,
        stream: None,
    };

    // Add test providers with test- prefix
    let openai_name = get_test_provider_name("openai");
    let anthropic_name = get_test_provider_name("anthropic");
    
    config.providers.insert(
        openai_name.clone(),
        create_test_provider_config("https://api.openai.com"),
    );

    config.providers.insert(
        anthropic_name.clone(),
        create_test_provider_config("https://api.anthropic.com"),
    );

    config.default_provider = Some(openai_name.clone());

    // Set up API keys in centralized keys.toml for test setup
    let mut keys = lc::keys::KeysConfig::load().unwrap_or_default();
    keys.set_api_key(openai_name.clone(), "test-api-key".to_string()).unwrap();
    keys.set_api_key(anthropic_name.clone(), "test-api-key".to_string()).unwrap();

    config
}

/// Test data constants
pub mod test_data {
    #[allow(dead_code)]
    pub const TEST_PROVIDERS: &[(&str, &str)] = &[
        ("test-openai", "https://api.openai.com"),
        ("test-anthropic", "https://api.anthropic.com"),
        ("test-cohere", "https://api.cohere.ai"),
        ("test-huggingface", "https://api-inference.huggingface.co"),
    ];

    #[allow(dead_code)]
    pub const TEST_MODELS: &[&str] =
        &["gpt-4", "gpt-3.5-turbo", "claude-3-opus", "claude-3-sonnet"];

    #[allow(dead_code)]
    pub const TEST_HEADERS: &[(&str, &str)] = &[
        ("X-API-Version", "2023-12-01"),
        ("X-Client", "lc-cli"),
        ("User-Agent", "LC/1.0"),
    ];
}

/// Assertion helpers for common test patterns
pub mod assertions {
    use lc::config::Config;

    #[allow(dead_code)]
    pub fn assert_provider_exists(config: &Config, name: &str) {
        assert!(
            config.has_provider(name),
            "Provider '{}' should exist",
            name
        );
    }

    #[allow(dead_code)]
    pub fn assert_provider_not_exists(config: &Config, name: &str) {
        assert!(
            !config.has_provider(name),
            "Provider '{}' should not exist",
            name
        );
    }

    #[allow(dead_code)]
    pub fn assert_provider_endpoint(config: &Config, name: &str, expected_endpoint: &str) {
        let provider = config.get_provider(name).expect("Provider should exist");
        assert_eq!(
            provider.endpoint, expected_endpoint,
            "Provider '{}' endpoint should be '{}'",
            name, expected_endpoint
        );
    }

    #[allow(dead_code)]
    pub fn assert_provider_has_api_key(_config: &Config, name: &str) {
        // Load keys from centralized keys.toml
        let keys = lc::keys::KeysConfig::load().expect("Should load keys config");
        assert!(
            keys.has_auth(name),
            "Provider '{}' should have an API key",
            name
        );
    }

    #[allow(dead_code)]
    pub fn assert_provider_api_key(_config: &Config, name: &str, expected_key: &str) {
        // Load keys from centralized keys.toml
        let keys = lc::keys::KeysConfig::load().expect("Should load keys config");
        let actual_key = keys.get_api_key(name).expect("Provider should have an API key");
        assert_eq!(
            actual_key,
            expected_key,
            "Provider '{}' API key should be '{}'",
            name,
            expected_key
        );
    }

    #[allow(dead_code)]
    pub fn assert_default_provider(config: &Config, expected_name: &str) {
        assert_eq!(
            config.default_provider.as_ref().unwrap(),
            expected_name,
            "Default provider should be '{}'",
            expected_name
        );
    }

    #[allow(dead_code)]
    pub fn assert_header_exists(
        config: &Config,
        provider: &str,
        header_name: &str,
        expected_value: &str,
    ) {
        let headers = config
            .list_headers(provider)
            .expect("Provider should exist");
        assert_eq!(
            headers.get(header_name).unwrap(),
            expected_value,
            "Header '{}' should have value '{}'",
            header_name,
            expected_value
        );
    }

    #[allow(dead_code)]
    pub fn assert_header_count(config: &Config, provider: &str, expected_count: usize) {
        let headers = config
            .list_headers(provider)
            .expect("Provider should exist");
        assert_eq!(
            headers.len(),
            expected_count,
            "Provider '{}' should have {} headers",
            provider,
            expected_count
        );
    }
}

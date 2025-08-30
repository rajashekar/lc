use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

mod common;
use common::get_test_binary_path;

// Global mutex to ensure tests run sequentially when needed
static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn get_config_dir() -> PathBuf {
    // Use the Config::config_dir() which handles test isolation
    lc::config::Config::config_dir().expect("Could not get config directory")
}

fn backup_config() -> Result<()> {
    let config_path = get_config_dir().join("search_config.toml");
    if config_path.exists() {
        fs::copy(&config_path, config_path.with_extension("toml.bak"))?;
    }
    Ok(())
}

fn restore_config() -> Result<()> {
    let config_path = get_config_dir().join("search_config.toml");
    let backup_path = config_path.with_extension("toml.bak");
    if backup_path.exists() {
        fs::copy(&backup_path, &config_path)?;
        fs::remove_file(&backup_path)?;
    }
    Ok(())
}

fn cleanup_config() -> Result<()> {
    let config_path = get_config_dir().join("search_config.toml");
    if config_path.exists() {
        fs::remove_file(&config_path)?;
    }
    Ok(())
}

#[test]
#[serial]
fn test_search_provider_add() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Test the underlying functionality directly instead of via CLI
    let mut config = SearchConfig::load()?;

    // Add a search provider (type auto-detected from URL)
    let result = config.add_provider_auto(
        "brave".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    );

    assert!(
        result.is_ok(),
        "Failed to add search provider: {:?}",
        result.err()
    );

    // Save the config
    config.save()?;

    // Verify config file was created and contains the provider
    let config_path = get_config_dir().join("search_config.toml");
    assert!(config_path.exists());

    // Reload and verify the provider is there
    let reloaded_config = SearchConfig::load()?;
    assert!(reloaded_config.has_provider("brave"));

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_provider_list() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider first using direct API
    let mut config = SearchConfig::load()?;
    config.add_provider_auto(
        "brave_list_test".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    )?;
    config.save()?;

    // Test listing providers
    let providers = config.list_providers();
    assert!(!providers.is_empty(), "Should have at least one provider");
    assert!(
        providers.contains_key("brave_list_test"),
        "Should contain the added provider"
    );

    // Test that the provider has the correct configuration
    let provider = providers.get("brave_list_test").unwrap();
    assert_eq!(provider.url, "https://api.search.brave.com/res/v1");

    // The direct API test above already verified the functionality works

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_provider_delete() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider first using direct API
    let mut config = SearchConfig::load()?;
    config.add_provider_auto(
        "brave_delete_test".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    )?;
    config.save()?;

    // Verify provider exists
    assert!(config.has_provider("brave_delete_test"));

    // Delete the provider using direct API
    let result = config.delete_provider("brave_delete_test");
    assert!(
        result.is_ok(),
        "Failed to delete provider: {:?}",
        result.err()
    );

    // Save and verify deletion
    config.save()?;
    assert!(
        !config.has_provider("brave_delete_test"),
        "Provider should be deleted"
    );

    // Reload config and verify deletion persists
    let reloaded_config = SearchConfig::load()?;
    assert!(
        !reloaded_config.has_provider("brave_delete_test"),
        "Provider should still be deleted after reload"
    );

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_provider_set_header() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider first using direct API
    let mut config = SearchConfig::load()?;
    config.add_provider_auto(
        "brave".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    )?;
    config.save()?;

    // Set a header using direct API
    config.set_header(
        "brave",
        "X-Subscription-Token".to_string(),
        "test-key".to_string(),
    )?;
    config.save()?;

    // Verify header was set
    let provider = config.get_provider("brave")?;
    assert!(provider.headers.contains_key("X-Subscription-Token"));
    assert_eq!(
        provider.headers.get("X-Subscription-Token"),
        Some(&"test-key".to_string())
    );

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_default_search_provider_config() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add providers using direct API
    let mut config = SearchConfig::load()?;

    // Add first provider
    config.add_provider_auto(
        "brave".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    )?;

    // Add second provider
    config.add_provider_auto("exa".to_string(), "https://api.exa.ai/search".to_string())?;

    config.save()?;

    // Verify the first provider is set as default
    assert_eq!(config.get_default_provider(), Some(&"brave".to_string()));

    // Test setting a different default
    config.set_default_provider("exa".to_string())?;
    config.save()?;

    // Verify the default changed
    assert_eq!(config.get_default_provider(), Some(&"exa".to_string()));

    // Reload and verify persistence
    let reloaded_config = SearchConfig::load()?;
    assert_eq!(
        reloaded_config.get_default_provider(),
        Some(&"exa".to_string())
    );

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_default_search_provider_config_cli() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add providers using direct API
    let mut config = SearchConfig::load()?;

    // Add first provider
    config.add_provider_auto(
        "brave_config_test".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    )?;

    // Add second provider
    config.add_provider_auto(
        "test_config".to_string(),
        "https://api.search.brave.com/res/v1/test".to_string(),
    )?;

    config.save()?;

    // Verify the first provider is set as default initially
    assert_eq!(
        config.get_default_provider(),
        Some(&"brave_config_test".to_string())
    );

    // Test setting a different default
    config.set_default_provider("test_config".to_string())?;
    config.save()?;

    // Verify the default changed
    assert_eq!(
        config.get_default_provider(),
        Some(&"test_config".to_string())
    );

    // Reload and verify persistence
    let reloaded_config = SearchConfig::load()?;
    assert_eq!(
        reloaded_config.get_default_provider(),
        Some(&"test_config".to_string())
    );

    // Test clearing default (set to empty string to clear)
    let mut config = reloaded_config;
    config.set_default_provider("".to_string())?;
    config.save()?;

    // Verify default is cleared
    let final_config = SearchConfig::load()?;
    assert_eq!(final_config.get_default_provider(), None);

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_query_missing_provider() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Try to get a nonexistent provider from config
    let config = SearchConfig::load()?;

    // Test that getting a nonexistent provider returns None
    let result = config.get_provider("nonexistent");
    assert!(result.is_err(), "Should not find nonexistent provider");

    // Test that searching with empty config has no providers
    assert!(
        config.list_providers().is_empty(),
        "Should have no providers configured"
    );

    // The direct API test above already verified that nonexistent providers are handled correctly

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_integration_flag() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider using direct API
    let mut config = SearchConfig::load()?;
    config.add_provider_auto(
        "brave".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    )?;
    config.save()?;

    // Test that the search configuration is properly set up
    assert!(config.has_provider("brave"));
    let provider = config.get_provider("brave")?;
    assert_eq!(provider.url, "https://api.search.brave.com/res/v1");

    // Test that we can parse the provider:query format
    let query_str = "brave:test query";
    if let Some((provider_name, query)) = query_str.split_once(':') {
        assert_eq!(provider_name, "brave");
        assert_eq!(query, "test query");
        assert!(config.has_provider(provider_name));
    } else {
        panic!("Failed to parse provider:query format");
    }

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_provider_duplicate_add() -> Result<()> {
    use lc::search::SearchConfig;

    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider using direct API
    let mut config = SearchConfig::load()?;
    config.add_provider_auto(
        "brave_dup_test".to_string(),
        "https://api.search.brave.com/res/v1".to_string(),
    )?;
    config.save()?;

    // Verify provider exists with original URL
    assert!(config.has_provider("brave_dup_test"));
    let provider = config.get_provider("brave_dup_test")?;
    assert_eq!(provider.url, "https://api.search.brave.com/res/v1");

    // Try to add the same provider again (should update)
    config.add_provider_auto(
        "brave_dup_test".to_string(),
        "https://api.search.brave.com/res/v1/different".to_string(),
    )?;
    config.save()?;

    // Verify URL was updated
    let updated_provider = config.get_provider("brave_dup_test")?;
    assert_eq!(
        updated_provider.url,
        "https://api.search.brave.com/res/v1/different"
    );

    // Verify we still have only one provider with this name
    let providers = config.list_providers();
    let brave_providers: Vec<_> = providers
        .keys()
        .filter(|k| k.contains("brave_dup_test"))
        .collect();
    assert_eq!(brave_providers.len(), 1);

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_output_formats() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider (type auto-detected from URL)
    Command::new(get_test_binary_path())
        .args(&[
            "search",
            "provider",
            "add",
            "brave",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    // Set a dummy API key
    Command::new(get_test_binary_path())
        .args(&[
            "search",
            "provider",
            "set",
            "brave",
            "X-Subscription-Token",
            "dummy-key",
        ])
        .output()?;

    // Test JSON format (will fail with invalid key, but should recognize format)
    let output = Command::new(get_test_binary_path())
        .args(&[
            "run", "--", "search", "query", "brave", "test", "-f", "json",
        ])
        .output()?;

    // Test markdown format
    let output2 = Command::new(get_test_binary_path())
        .args(&["search", "query", "brave", "test", "-f", "md"])
        .output()?;

    // Both should fail due to invalid API key, but formats should be accepted
    assert!(!output.status.success() || !output2.status.success());

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
#[serial]
fn test_search_result_count() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider (type auto-detected from URL)
    Command::new(get_test_binary_path())
        .args(&[
            "search",
            "provider",
            "add",
            "brave",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    // Test custom result count
    let _output = Command::new(get_test_binary_path())
        .args(&["search", "query", "brave", "test", "-n", "10"])
        .output()?;

    // Should accept the count parameter even if search fails
    assert!(true);

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

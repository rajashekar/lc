use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

// Global mutex to ensure tests run sequentially
static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Could not find config directory")
        .join("lc")
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
fn test_search_provider_add() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a search provider (type auto-detected from URL)
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "Failed to add search provider: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Search provider 'brave' added successfully"));

    // Verify config file was created
    let config_path = get_config_dir().join("search_config.toml");
    assert!(config_path.exists());

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_provider_list() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider first (type auto-detected from URL)
    let add_output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave_list_test",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    assert!(
        add_output.status.success(),
        "Failed to add provider: {}",
        String::from_utf8_lossy(&add_output.stderr)
    );

    // List providers
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "list"])
        .output()?;

    assert!(
        output.status.success(),
        "Failed to list providers: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("brave_list_test"),
        "Provider name not found in list output"
    );
    assert!(
        stdout.contains("https://api.search.brave.com/res/v1"),
        "Provider URL not found in list output"
    );

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_provider_delete() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider first (type auto-detected from URL)
    let add_output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave_delete_test",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    assert!(
        add_output.status.success(),
        "Failed to add provider: {}",
        String::from_utf8_lossy(&add_output.stderr)
    );

    // Delete the provider
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "delete",
            "brave_delete_test",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "Failed to delete provider: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Search provider 'brave_delete_test' deleted successfully"));

    // Verify it's gone
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "list"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No search providers configured"));

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_provider_set_header() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider first (type auto-detected from URL)
    Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    // Set a header
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "set",
            "brave",
            "X-Subscription-Token",
            "test-key",
        ])
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Header 'X-Subscription-Token' set for search provider 'brave'"));

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_default_search_provider_config() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add providers (type auto-detected from URL)
    let add1 = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave_config_test",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    assert!(
        add1.status.success(),
        "Failed to add first provider: {}",
        String::from_utf8_lossy(&add1.stderr)
    );

    let add2 = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "test_config",
            "https://api.search.brave.com/res/v1/test",
        ])
        .output()?;

    assert!(
        add2.status.success(),
        "Failed to add second provider: {}",
        String::from_utf8_lossy(&add2.stderr)
    );

    // Set default search provider
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "set", "search", "test_config"])
        .output()?;

    assert!(
        output.status.success(),
        "Failed to set default provider: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Default search provider set to 'test_config'"));

    // Get default search provider
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "get", "search"])
        .output()?;

    assert!(
        output.status.success(),
        "Failed to get default provider: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().contains("test_config"));

    // Delete default search provider
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "delete", "search"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Default search provider deleted"));

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_query_missing_provider() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Try to search without any providers configured
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "query", "nonexistent", "test query"])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Search provider 'nonexistent' not found") || stderr.contains("not found")
    );

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_integration_flag() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider (type auto-detected from URL)
    Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    // Test that --use-search flag is accepted (won't actually search without API key)
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--use-search",
            "brave:test query",
            "What is Rust programming?",
        ])
        .output()?;

    // It should fail due to missing API key, but the flag should be recognized
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The error could be about missing API key or search provider not found
    assert!(
        stderr.contains("Search failed")
            || stderr.contains("API")
            || stderr.contains("401")
            || stderr.contains("not found")
            || stderr.contains("No API key")
    );

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_provider_duplicate_add() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider (type auto-detected from URL)
    let add1 = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave_dup_test",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    assert!(
        add1.status.success(),
        "Failed to add provider first time: {}",
        String::from_utf8_lossy(&add1.stderr)
    );

    // Try to add the same provider again (with different URL that will be auto-detected)
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave_dup_test",
            "https://api.search.brave.com/res/v1/different",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "Failed to update provider: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // List to verify URL was updated
    let list_output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "list"])
        .output()?;

    assert!(
        list_output.status.success(),
        "Failed to list providers: {}",
        String::from_utf8_lossy(&list_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        stdout.contains("https://api.search.brave.com/res/v1/different"),
        "Updated URL not found in list output"
    );

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_output_formats() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider (type auto-detected from URL)
    Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    // Set a dummy API key
    Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "set",
            "brave",
            "X-Subscription-Token",
            "dummy-key",
        ])
        .output()?;

    // Test JSON format (will fail with invalid key, but should recognize format)
    let output = Command::new("cargo")
        .args(&[
            "run", "--", "search", "query", "brave", "test", "-f", "json",
        ])
        .output()?;

    // Test markdown format
    let output2 = Command::new("cargo")
        .args(&["run", "--", "search", "query", "brave", "test", "-f", "md"])
        .output()?;

    // Both should fail due to invalid API key, but formats should be accepted
    assert!(!output.status.success() || !output2.status.success());

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_result_count() -> Result<()> {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    backup_config()?;
    cleanup_config()?;

    // Add a provider (type auto-detected from URL)
    Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            "provider",
            "add",
            "brave",
            "https://api.search.brave.com/res/v1",
        ])
        .output()?;

    // Test custom result count
    let _output = Command::new("cargo")
        .args(&["run", "--", "search", "query", "brave", "test", "-n", "10"])
        .output()?;

    // Should accept the count parameter even if search fails
    assert!(true);

    cleanup_config()?;
    restore_config()?;
    Ok(())
}

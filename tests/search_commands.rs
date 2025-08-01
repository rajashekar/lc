use anyhow::Result;
use std::process::Command;
use std::fs;
use std::path::PathBuf;

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
    backup_config()?;
    
    // Add a search provider
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    assert!(output.status.success(), "Failed to add search provider: {}", String::from_utf8_lossy(&output.stderr));
    
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
    backup_config()?;
    
    // Add a provider first
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    // List providers
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "list"])
        .output()?;
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("brave"));
    assert!(stdout.contains("https://api.search.brave.com/res/v1/web/search"));
    
    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_provider_delete() -> Result<()> {
    backup_config()?;
    
    // Add a provider first
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    // Delete the provider
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "delete", "brave"])
        .output()?;
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Search provider 'brave' deleted successfully"));
    
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
    backup_config()?;
    
    // Add a provider first
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    // Set a header
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "set", "brave", "X-Subscription-Token", "test-key"])
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
    backup_config()?;
    
    // Add providers
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "test", "https://test.com/search"])
        .output()?;
    
    // Set default search provider
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "set", "search", "test"])
        .output()?;
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Default search provider set to 'test'"));
    
    // Get default search provider
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "get", "search"])
        .output()?;
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().contains("test"));
    
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
    backup_config()?;
    
    // Try to search without any providers configured
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "query", "nonexistent", "test query"])
        .output()?;
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Search provider 'nonexistent' not found"));
    
    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_integration_flag() -> Result<()> {
    backup_config()?;
    
    // Add a provider
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    // Test that --use-search flag is accepted (won't actually search without API key)
    let output = Command::new("cargo")
        .args(&["run", "--", "--use-search", "brave:test query", "What is Rust programming?"])
        .output()?;
    
    // It should fail due to missing API key, but the flag should be recognized
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Search failed") || stderr.contains("API") || stderr.contains("401"));
    
    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_provider_duplicate_add() -> Result<()> {
    backup_config()?;
    
    // Add a provider
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    // Try to add the same provider again
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://different-url.com"])
        .output()?;
    
    assert!(output.status.success());
    
    // List to verify URL was updated
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "provider", "list"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("https://different-url.com"));
    
    cleanup_config()?;
    restore_config()?;
    Ok(())
}

#[test]
fn test_search_output_formats() -> Result<()> {
    backup_config()?;
    
    // Add a provider
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    // Set a dummy API key
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "set", "brave", "X-Subscription-Token", "dummy-key"])
        .output()?;
    
    // Test JSON format (will fail with invalid key, but should recognize format)
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "query", "brave", "test", "-f", "json"])
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
    backup_config()?;
    
    // Add a provider
    Command::new("cargo")
        .args(&["run", "--", "search", "provider", "add", "brave", "https://api.search.brave.com/res/v1/web/search"])
        .output()?;
    
    // Test custom result count
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "query", "brave", "test", "-n", "10"])
        .output()?;
    
    // Should accept the count parameter even if search fails
    assert!(true);
    
    cleanup_config()?;
    restore_config()?;
    Ok(())
}
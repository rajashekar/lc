#![cfg(unix)]

use lc::data::keys::KeysConfig;
use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_keys_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    env::set_var("LC_TEST_CONFIG_DIR", temp_dir.path());

    // Create a new KeysConfig and save it
    let mut keys = KeysConfig::default();
    keys.set_api_key("test-provider".to_string(), "test-key".to_string())
        .unwrap();

    // Get the path to the keys file
    let keys_path = temp_dir.path().join("keys.toml");

    // Verify file exists
    assert!(keys_path.exists());

    // Check permissions
    let metadata = fs::metadata(&keys_path).unwrap();
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Verify mode is 0o600 (usually represented as 33152 in decimal for regular file + 600)
    // We mask with 0o777 to check just the permissions part
    assert_eq!(mode & 0o777, 0o600, "File permissions should be 0o600");
}

#[test]
#[serial]
fn test_existing_file_permissions_corrected() {
    use std::os::unix::fs::PermissionsExt;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    env::set_var("LC_TEST_CONFIG_DIR", temp_dir.path());

    let keys_path = temp_dir.path().join("keys.toml");

    // Create an existing file with insecure permissions (e.g. 0o644)
    {
        use std::io::Write;
        let mut file = fs::File::create(&keys_path).unwrap();
        file.write_all(b"dummy content").unwrap();
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o644);
        file.set_permissions(permissions).unwrap();
    }

    // Verify it's initially insecure
    let initial_mode = fs::metadata(&keys_path).unwrap().permissions().mode();
    assert_eq!(
        initial_mode & 0o777,
        0o644,
        "Initial permissions should be 0o644"
    );

    // Save KeysConfig, which should fix the permissions
    let mut keys = KeysConfig::default();
    keys.set_api_key(
        "test-provider-correction".to_string(),
        "test-key".to_string(),
    )
    .unwrap();

    // Verify permissions are corrected to 0o600
    let metadata = fs::metadata(&keys_path).unwrap();
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    assert_eq!(
        mode & 0o777,
        0o600,
        "File permissions should be corrected to 0o600"
    );
}

#[cfg(feature = "unix-sockets")]
#[tokio::test]
async fn test_mcp_daemon_socket_permissions() {
    use lc::mcp_daemon::McpDaemon;
    use std::os::unix::fs::PermissionsExt;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    env::set_var("LC_TEST_CONFIG_DIR", temp_dir.path());

    // Spawn daemon in background
    let handle = tokio::spawn(async move {
        // We need to re-set the env var in the spawned task thread?
        // Actually env vars are process-wide, so it should be fine.
        // But we need to handle potential errors gracefully
        if let Ok(mut daemon) = McpDaemon::new() {
            let _ = daemon.start().await;
        }
    });

    // Wait for socket
    let socket_path = temp_dir.path().join("mcp_daemon.sock");
    let mut retries = 50;
    while !socket_path.exists() && retries > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        retries -= 1;
    }

    if !socket_path.exists() {
        handle.abort();
        panic!("Socket file not created within timeout");
    }

    // Check permissions
    let metadata = fs::metadata(&socket_path).unwrap();
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Verify mode is 0o600
    // We mask with 0o777 to check just the permissions part
    // Note: If this fails, it means the fix hasn't been applied yet!

    // Clean up before asserting so we don't leave the daemon running on failure
    handle.abort();

    assert_eq!(
        mode & 0o777,
        0o600,
        "Socket permissions should be 0o600, but were {:o}",
        mode & 0o777
    );
}

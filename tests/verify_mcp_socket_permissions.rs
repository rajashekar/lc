#![cfg(all(unix, feature = "unix-sockets"))]

use lc::services::mcp_daemon::McpDaemon;
use serial_test::serial;
use std::env;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

#[tokio::test]
#[serial]
async fn test_mcp_socket_permissions() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    env::set_var("LC_TEST_CONFIG_DIR", temp_dir.path());

    // Create a new McpDaemon instance
    // Note: This relies on LC_TEST_CONFIG_DIR being set
    let mut daemon = McpDaemon::new().unwrap();
    let socket_path = McpDaemon::get_socket_path().unwrap();

    // Spawn the daemon in a background task
    // The daemon runs indefinitely, so we spawn it and will abort it later
    let daemon_handle = tokio::spawn(async move {
        if let Err(e) = daemon.start().await {
            eprintln!("Daemon error: {}", e);
        }
    });

    // Wait for the socket file to be created
    let mut tries = 0;
    while !socket_path.exists() && tries < 50 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        tries += 1;
    }

    assert!(socket_path.exists(), "Socket file was not created within timeout");

    // Check permissions
    let metadata = std::fs::metadata(&socket_path).expect("Failed to get socket metadata");
    let mode = metadata.permissions().mode();

    // Clean up: Abort the daemon task since it runs forever
    daemon_handle.abort();

    // Verify mode is 0o600 (read/write for owner only)
    // We mask with 0o777 to ignore file type bits
    assert_eq!(
        mode & 0o777,
        0o600,
        "Socket permissions should be 0o600 (rw-------), but got 0o{:o}",
        mode & 0o777
    );
}

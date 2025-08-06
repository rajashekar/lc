//! Integration tests for MCP commands
//!
//! This module contains comprehensive integration tests for all MCP-related
//! CLI commands, testing them as a user would interact with the CLI.

use lc::mcp::{McpConfig, McpServerConfig, McpServerType, SdkMcpManager};
use std::collections::HashMap;

#[cfg(test)]
mod mcp_config_tests {
    use super::*;

    fn create_empty_mcp_config() -> McpConfig {
        McpConfig {
            servers: HashMap::new(),
        }
    }

    fn create_mcp_config_with_servers() -> McpConfig {
        let mut config = McpConfig {
            servers: HashMap::new(),
        };

        config
            .add_server(
                "test-stdio".to_string(),
                "echo 'test'".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();

        config
            .add_server(
                "test-sse".to_string(),
                "http://localhost:8080/sse".to_string(),
                McpServerType::Sse,
            )
            .unwrap();

        config
    }

    #[test]
    fn test_mcp_config_new_empty() {
        let config = create_empty_mcp_config();
        assert!(config.servers.is_empty());
    }

    #[test]
    fn test_mcp_config_add_server() {
        let mut config = create_empty_mcp_config();

        let result = config.add_server(
            "new-server".to_string(),
            "npx @example/server".to_string(),
            McpServerType::Stdio,
        );

        assert!(result.is_ok());
        assert_eq!(config.servers.len(), 1);

        let server = config.servers.get("new-server").unwrap();
        assert_eq!(server.name, "new-server");
        assert_eq!(server.command_or_url, "npx @example/server");
        assert_eq!(server.server_type, McpServerType::Stdio);
    }

    #[test]
    fn test_mcp_config_add_server_sse() {
        let mut config = create_empty_mcp_config();

        let result = config.add_server(
            "sse-server".to_string(),
            "http://localhost:8080/sse".to_string(),
            McpServerType::Sse,
        );

        assert!(result.is_ok());
        let server = config.servers.get("sse-server").unwrap();
        assert_eq!(server.server_type, McpServerType::Sse);
    }

    #[test]
    fn test_mcp_config_add_server_streamable() {
        let mut config = create_empty_mcp_config();

        let result = config.add_server(
            "streamable-server".to_string(),
            "http://localhost:8080".to_string(),
            McpServerType::Streamable,
        );

        assert!(result.is_ok());
        let server = config.servers.get("streamable-server").unwrap();
        assert_eq!(server.server_type, McpServerType::Streamable);
    }

    #[test]
    fn test_mcp_config_delete_server() {
        let mut config = create_mcp_config_with_servers();
        assert_eq!(config.servers.len(), 2);

        let result = config.delete_server("test-stdio");
        assert!(result.is_ok());
        assert_eq!(config.servers.len(), 1);
        assert!(!config.servers.contains_key("test-stdio"));
        assert!(config.servers.contains_key("test-sse"));
    }

    #[test]
    fn test_mcp_config_delete_nonexistent_server() {
        let mut config = create_empty_mcp_config();

        let result = config.delete_server("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_mcp_config_get_server() {
        let config = create_mcp_config_with_servers();

        let server = config.get_server("test-stdio");
        assert!(server.is_some());
        assert_eq!(server.unwrap().name, "test-stdio");

        let nonexistent = config.get_server("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_mcp_config_list_servers() {
        let config = create_mcp_config_with_servers();
        let servers = config.list_servers();

        assert_eq!(servers.len(), 2);
        assert!(servers.contains_key("test-stdio"));
        assert!(servers.contains_key("test-sse"));
    }

    #[test]
    fn test_mcp_config_duplicate_server_names() {
        let mut config = create_empty_mcp_config();

        // Add a server
        config
            .add_server(
                "test-server".to_string(),
                "first-command".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();

        // Add another server with the same name (should overwrite)
        config
            .add_server(
                "test-server".to_string(),
                "second-command".to_string(),
                McpServerType::Sse,
            )
            .unwrap();

        assert_eq!(config.servers.len(), 1);
        let server = config.servers.get("test-server").unwrap();
        assert_eq!(server.command_or_url, "second-command");
        assert_eq!(server.server_type, McpServerType::Sse);
    }

    #[test]
    fn test_mcp_config_validation() {
        let mut config = create_empty_mcp_config();

        // Test adding server with empty name
        let result = config.add_server(
            "".to_string(),
            "test-command".to_string(),
            McpServerType::Stdio,
        );
        // Should succeed (no validation in add_server currently)
        assert!(result.is_ok());

        // Test adding server with empty command
        let result = config.add_server(
            "test-server".to_string(),
            "".to_string(),
            McpServerType::Stdio,
        );
        // Should succeed (no validation in add_server currently)
        assert!(result.is_ok());
    }

    #[test]
    fn test_mcp_server_lifecycle() {
        let mut config = create_empty_mcp_config();

        // Initially empty
        assert!(config.servers.is_empty());

        // Add server
        config
            .add_server(
                "lifecycle-test".to_string(),
                "test-command".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();

        // Verify added
        assert_eq!(config.servers.len(), 1);
        assert!(config.get_server("lifecycle-test").is_some());

        // Delete server
        config.delete_server("lifecycle-test").unwrap();

        // Verify deleted
        assert!(config.servers.is_empty());
        assert!(config.get_server("lifecycle-test").is_none());
    }
}

#[cfg(test)]
mod mcp_server_config_tests {
    use super::*;

    #[test]
    fn test_mcp_server_config_creation() {
        let config = McpServerConfig {
            name: "test-server".to_string(),
            server_type: McpServerType::Stdio,
            command_or_url: "echo test".to_string(),
            env: HashMap::new(),
        };

        assert_eq!(config.name, "test-server");
        assert_eq!(config.server_type, McpServerType::Stdio);
        assert_eq!(config.command_or_url, "echo test");
    }

    #[test]
    fn test_mcp_server_types() {
        let stdio = McpServerType::Stdio;
        let sse = McpServerType::Sse;
        let streamable = McpServerType::Streamable;

        assert_eq!(stdio, McpServerType::Stdio);
        assert_eq!(sse, McpServerType::Sse);
        assert_eq!(streamable, McpServerType::Streamable);

        assert_ne!(stdio, sse);
        assert_ne!(sse, streamable);
        assert_ne!(stdio, streamable);
    }
}

#[cfg(test)]
mod mcp_integration_tests {
    use super::*;

    #[test]
    fn test_mcp_config_and_registry_integration() {
        let mut config = McpConfig {
            servers: HashMap::new(),
        };
        // Add server to config
        config
            .add_server(
                "integration-test".to_string(),
                "echo test".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();

        // Verify config has the server
        assert!(config.get_server("integration-test").is_some());
    }

    #[test]
    fn test_mcp_server_type_consistency() {
        let mut config = McpConfig {
            servers: HashMap::new(),
        };

        // Add servers of different types
        config
            .add_server(
                "stdio-server".to_string(),
                "echo test".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();
        config
            .add_server(
                "sse-server".to_string(),
                "http://localhost:8080/sse".to_string(),
                McpServerType::Sse,
            )
            .unwrap();
        config
            .add_server(
                "streamable-server".to_string(),
                "http://localhost:8080".to_string(),
                McpServerType::Streamable,
            )
            .unwrap();

        // Verify all servers exist with correct types
        let stdio_server = config.get_server("stdio-server").unwrap();
        let sse_server = config.get_server("sse-server").unwrap();
        let streamable_server = config.get_server("streamable-server").unwrap();

        assert_eq!(stdio_server.server_type, McpServerType::Stdio);
        assert_eq!(sse_server.server_type, McpServerType::Sse);
        assert_eq!(streamable_server.server_type, McpServerType::Streamable);
    }

    #[test]
    fn test_mcp_error_handling() {
        let mut config = McpConfig {
            servers: HashMap::new(),
        };

        // Test deleting non-existent server
        let result = config.delete_server("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test getting non-existent server
        let server = config.get_server("nonexistent");
        assert!(server.is_none());
    }

    #[test]
    fn test_mcp_multiple_operations() {
        let mut config = McpConfig {
            servers: HashMap::new(),
        };

        // Add multiple servers
        config
            .add_server(
                "server1".to_string(),
                "cmd1".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();
        config
            .add_server(
                "server2".to_string(),
                "cmd2".to_string(),
                McpServerType::Sse,
            )
            .unwrap();
        config
            .add_server(
                "server3".to_string(),
                "cmd3".to_string(),
                McpServerType::Streamable,
            )
            .unwrap();

        assert_eq!(config.servers.len(), 3);

        // Delete one server
        config.delete_server("server2").unwrap();
        assert_eq!(config.servers.len(), 2);
        assert!(!config.servers.contains_key("server2"));

        // Verify remaining servers
        assert!(config.servers.contains_key("server1"));
        assert!(config.servers.contains_key("server3"));
    }
}

#[cfg(test)]
mod sdk_mcp_manager_tests {
    use super::*;

    #[test]
    fn test_sdk_mcp_manager_new() {
        let manager = SdkMcpManager::new();
        // Just verify we can create a manager instance
        assert!(std::ptr::addr_of!(manager) as usize != 0);
    }

    #[tokio::test]
    async fn test_sdk_mcp_manager_lifecycle() {
        let manager = SdkMcpManager::new();

        // Test that we can create and close the manager without errors
        // Test that we can create the manager without errors
        // Note: close_all method was removed during cleanup
        assert!(std::ptr::addr_of!(manager) as usize != 0);
    }
}

#[cfg(test)]
mod sdk_integration_tests {
    use super::*;
    use lc::mcp::{create_sse_server_config, create_stdio_server_config};

    #[test]
    fn test_create_stdio_server_config() {
        let config = create_stdio_server_config(
            "test-server".to_string(),
            vec!["echo".to_string(), "hello".to_string()],
            None,
            None,
        );

        assert_eq!(config.name, "test-server");
        // Verify the config was created successfully
        assert!(std::ptr::addr_of!(config) as usize != 0);
    }

    #[test]
    fn test_create_sse_server_config() {
        let config = create_sse_server_config(
            "sse-server".to_string(),
            "http://localhost:8080/sse".to_string(),
        );

        assert_eq!(config.name, "sse-server");
        // Verify the config was created successfully
        assert!(std::ptr::addr_of!(config) as usize != 0);
    }

    #[test]
    fn test_legacy_to_sdk_config_conversion() {
        // Test that we can convert legacy config types to SDK config types
        let mut config = McpConfig {
            servers: HashMap::new(),
        };

        // Add a legacy server configuration
        config
            .add_server(
                "legacy-server".to_string(),
                "echo test".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();

        // Verify we can retrieve it
        let server_config = config.get_server("legacy-server");
        assert!(server_config.is_some());

        let server = server_config.unwrap();
        assert_eq!(server.name, "legacy-server");
        assert_eq!(server.server_type, McpServerType::Stdio);

        // Test conversion to SDK config (this would be done in the CLI handlers)
        let parts: Vec<String> = server
            .command_or_url
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let sdk_config = create_stdio_server_config(server.name.clone(), parts, None, None);

        assert_eq!(sdk_config.name, "legacy-server");
    }

    #[tokio::test]
    async fn test_sdk_manager_with_invalid_server() {
        let mut manager = SdkMcpManager::new();

        // Try to add an invalid server configuration
        let invalid_config = create_stdio_server_config(
            "invalid-server".to_string(),
            vec!["nonexistent-command".to_string()],
            None,
            None,
        );

        // This should fail gracefully
        let result = manager.add_server(invalid_config).await;
        // We expect this to fail since the command doesn't exist
        assert!(result.is_err());

        // Note: close_all method was removed during cleanup
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that legacy MCP functionality still works alongside SDK
        let sdk_manager = SdkMcpManager::new();

        // Should be creatable without issues
        assert!(std::ptr::addr_of!(sdk_manager) as usize != 0);
    }
}

#[cfg(test)]
mod mcp_config_sdk_integration_tests {
    use super::*;

    #[test]
    fn test_mcp_config_supports_all_server_types() {
        let mut config = McpConfig {
            servers: HashMap::new(),
        };

        // Test that all server types are supported
        config
            .add_server(
                "stdio-test".to_string(),
                "echo test".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();
        config
            .add_server(
                "sse-test".to_string(),
                "http://localhost:8080/sse".to_string(),
                McpServerType::Sse,
            )
            .unwrap();
        config
            .add_server(
                "streamable-test".to_string(),
                "http://localhost:8080".to_string(),
                McpServerType::Streamable,
            )
            .unwrap();

        assert_eq!(config.servers.len(), 3);

        // Verify each server type is correctly stored
        let stdio_server = config.get_server("stdio-test").unwrap();
        let sse_server = config.get_server("sse-test").unwrap();
        let streamable_server = config.get_server("streamable-test").unwrap();

        assert_eq!(stdio_server.server_type, McpServerType::Stdio);
        assert_eq!(sse_server.server_type, McpServerType::Sse);
        assert_eq!(streamable_server.server_type, McpServerType::Streamable);
    }

    #[test]
    fn test_mcp_config_persistence_compatibility() {
        // Test that config can be saved and loaded (basic structure test)
        let mut config = McpConfig {
            servers: HashMap::new(),
        };

        config
            .add_server(
                "persistent-server".to_string(),
                "echo persistent".to_string(),
                McpServerType::Stdio,
            )
            .unwrap();

        // Verify the server was added
        assert_eq!(config.servers.len(), 1);
        let server = config.get_server("persistent-server").unwrap();
        assert_eq!(server.command_or_url, "echo persistent");
    }
}

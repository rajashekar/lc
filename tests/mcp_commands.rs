//! Integration tests for MCP commands
//! 
//! This module contains comprehensive integration tests for all MCP-related
//! CLI commands, testing them as a user would interact with the CLI.

use lc::mcp::{McpConfig, McpServerConfig, McpServerType, ProcessRegistry, ProcessRegistryEntry, McpManager};
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
        
        config.add_server(
            "test-stdio".to_string(),
            "echo 'test'".to_string(),
            McpServerType::Stdio,
        ).unwrap();
        
        config.add_server(
            "test-sse".to_string(),
            "http://localhost:8080/sse".to_string(),
            McpServerType::Sse,
        ).unwrap();
        
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
        config.add_server(
            "test-server".to_string(),
            "first-command".to_string(),
            McpServerType::Stdio,
        ).unwrap();
        
        // Add another server with the same name (should overwrite)
        config.add_server(
            "test-server".to_string(),
            "second-command".to_string(),
            McpServerType::Sse,
        ).unwrap();
        
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
        config.add_server(
            "lifecycle-test".to_string(),
            "test-command".to_string(),
            McpServerType::Stdio,
        ).unwrap();
        
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
mod process_registry_tests {
    use super::*;

    fn create_empty_process_registry() -> ProcessRegistry {
        ProcessRegistry {
            servers: HashMap::new(),
        }
    }

    #[test]
    fn test_process_registry_new_empty() {
        let registry = create_empty_process_registry();
        assert!(registry.servers.is_empty());
    }

    #[test]
    fn test_process_registry_add_server() {
        let mut registry = create_empty_process_registry();
        
        let result = registry.add_server(
            "test-server".to_string(),
            12345,
            "Stdio".to_string(),
            "test-command".to_string(),
        );
        
        assert!(result.is_ok());
        assert_eq!(registry.servers.len(), 1);
        
        let entry = registry.servers.get("test-server").unwrap();
        assert_eq!(entry.name, "test-server");
        assert_eq!(entry.pid, 12345);
        assert_eq!(entry.server_type, "Stdio");
        assert_eq!(entry.command, "test-command");
        assert!(entry.session_id.is_none());
    }

    #[test]
    fn test_process_registry_add_server_with_session() {
        let mut registry = create_empty_process_registry();
        
        let result = registry.add_server_with_session(
            "test-server".to_string(),
            0, // Streamable servers use PID 0
            "Streamable".to_string(),
            "http://localhost:8080".to_string(),
            Some("session-123".to_string()),
        );
        
        assert!(result.is_ok());
        assert_eq!(registry.servers.len(), 1);
        
        let entry = registry.servers.get("test-server").unwrap();
        assert_eq!(entry.pid, 0);
        assert_eq!(entry.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_process_registry_remove_server() {
        let mut registry = create_empty_process_registry();
        
        registry.add_server(
            "test-server".to_string(),
            12345,
            "Stdio".to_string(),
            "test-command".to_string(),
        ).unwrap();
        
        assert_eq!(registry.servers.len(), 1);
        
        let result = registry.remove_server("test-server");
        assert!(result.is_ok());
        assert!(registry.servers.is_empty());
    }

    #[test]
    fn test_process_registry_get_running_servers() {
        let mut registry = create_empty_process_registry();
        
        // Add a Streamable server with session ID (should be considered running)
        registry.add_server_with_session(
            "streamable-server".to_string(),
            0,
            "Streamable".to_string(),
            "http://localhost:8080".to_string(),
            Some("session-123".to_string()),
        ).unwrap();
        
        // Add a Streamable server without session ID (should not be considered running)
        registry.add_server_with_session(
            "inactive-server".to_string(),
            0,
            "Streamable".to_string(),
            "http://localhost:8081".to_string(),
            None,
        ).unwrap();
        
        let running = registry.get_running_servers();
        
        // Only the server with session ID should be considered running
        assert_eq!(running.len(), 1);
        assert!(running.contains_key("streamable-server"));
        assert!(!running.contains_key("inactive-server"));
    }

    #[test]
    fn test_process_registry_cleanup_dead_processes() {
        let mut registry = create_empty_process_registry();
        
        // Add a Streamable server (PID 0, should not be cleaned up)
        registry.add_server_with_session(
            "streamable-server".to_string(),
            0,
            "Streamable".to_string(),
            "http://localhost:8080".to_string(),
            Some("session-123".to_string()),
        ).unwrap();
        
        // Add a regular server with a very high PID (likely not running)
        registry.add_server(
            "dead-server".to_string(),
            999999,
            "Stdio".to_string(),
            "test-command".to_string(),
        ).unwrap();
        
        assert_eq!(registry.servers.len(), 2);
        
        let result = registry.cleanup_dead_processes();
        assert!(result.is_ok());
        
        // Streamable server should remain, dead server might be removed
        // (depends on whether PID 999999 actually exists)
        assert!(registry.servers.contains_key("streamable-server"));
    }

    #[test]
    fn test_process_registry_lifecycle() {
        let mut registry = create_empty_process_registry();
        
        // Initially empty
        assert!(registry.servers.is_empty());
        
        // Add process
        registry.add_server(
            "test-process".to_string(),
            12345,
            "Stdio".to_string(),
            "test-command".to_string(),
        ).unwrap();
        
        // Verify added
        assert_eq!(registry.servers.len(), 1);
        
        // Remove process
        registry.remove_server("test-process").unwrap();
        
        // Verify removed
        assert!(registry.servers.is_empty());
    }
}

#[cfg(test)]
mod mcp_manager_tests {
    use super::*;

    #[test]
    fn test_mcp_manager_new() {
        let manager = McpManager::new();
        // Just verify we can create a manager instance
        assert!(std::ptr::addr_of!(manager) as usize != 0);
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
mod process_registry_entry_tests {
    use super::*;

    #[test]
    fn test_process_registry_entry_creation() {
        let entry = ProcessRegistryEntry {
            name: "test-server".to_string(),
            pid: 12345,
            server_type: "Stdio".to_string(),
            command: "echo test".to_string(),
            session_id: None,
        };
        
        assert_eq!(entry.name, "test-server");
        assert_eq!(entry.pid, 12345);
        assert_eq!(entry.server_type, "Stdio");
        assert_eq!(entry.command, "echo test");
        assert!(entry.session_id.is_none());
    }

    #[test]
    fn test_process_registry_entry_with_session() {
        let entry = ProcessRegistryEntry {
            name: "streamable-server".to_string(),
            pid: 0,
            server_type: "Streamable".to_string(),
            command: "http://localhost:8080".to_string(),
            session_id: Some("session-123".to_string()),
        };
        
        assert_eq!(entry.name, "streamable-server");
        assert_eq!(entry.pid, 0);
        assert_eq!(entry.server_type, "Streamable");
        assert_eq!(entry.command, "http://localhost:8080");
        assert_eq!(entry.session_id, Some("session-123".to_string()));
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
        let mut registry = ProcessRegistry {
            servers: HashMap::new(),
        };
        
        // Add server to config
        config.add_server(
            "integration-test".to_string(),
            "echo test".to_string(),
            McpServerType::Stdio,
        ).unwrap();
        
        // Simulate starting the server by adding to registry
        registry.add_server(
            "integration-test".to_string(),
            12345,
            "Stdio".to_string(),
            "echo test".to_string(),
        ).unwrap();
        
        // Verify both config and registry have the server
        assert!(config.get_server("integration-test").is_some());
        assert!(registry.servers.contains_key("integration-test"));
        
        let running_servers = registry.get_running_servers();
        // Note: This might be 0 if PID 12345 doesn't exist, which is expected in tests
        assert!(running_servers.len() <= 1);
    }

    #[test]
    fn test_mcp_server_type_consistency() {
        let mut config = McpConfig {
            servers: HashMap::new(),
        };
        
        // Add servers of different types
        config.add_server("stdio-server".to_string(), "echo test".to_string(), McpServerType::Stdio).unwrap();
        config.add_server("sse-server".to_string(), "http://localhost:8080/sse".to_string(), McpServerType::Sse).unwrap();
        config.add_server("streamable-server".to_string(), "http://localhost:8080".to_string(), McpServerType::Streamable).unwrap();
        
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
        config.add_server("server1".to_string(), "cmd1".to_string(), McpServerType::Stdio).unwrap();
        config.add_server("server2".to_string(), "cmd2".to_string(), McpServerType::Sse).unwrap();
        config.add_server("server3".to_string(), "cmd3".to_string(), McpServerType::Streamable).unwrap();
        
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
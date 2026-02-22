# Sentinel's Journal

## 2025-02-22 - [MCP Daemon Socket Permissions]
**Vulnerability:** The MCP daemon Unix socket was created with default umask permissions (likely 0o755 or 0o775), allowing other users on the system to connect to it. This could allow unauthorized access to MCP tools and potential command execution.
**Learning:** `UnixListener::bind` does not set restrictive permissions by default. When creating IPC sockets, especially those exposing powerful capabilities like command execution, explicit permission hardening (0o600) is required.
**Prevention:** Always explicitly set file permissions on Unix domain sockets immediately after binding using `std::fs::set_permissions`.

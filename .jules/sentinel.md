## 2025-03-08 - Fix insecure MCP daemon Unix socket permissions

**Vulnerability:** The MCP daemon in `src/services/mcp_daemon.rs` created a Unix domain socket (`mcp_daemon.sock`) using `UnixListener::bind` without explicitly setting file permissions. By default, this socket could potentially be accessible to other users on the system (e.g., `0o755` or `0o644` depending on umask), leading to a local authorization bypass where any user could issue commands to the MCP daemon (like calling tools).

**Learning:** When dealing with inter-process communication (IPC) via Unix domain sockets, the default permissions are often governed by the process's `umask`, which is generally not restrictive enough for secure IPC. It is a security necessity to explicitly lock down socket permissions to `0o600` immediately after creation to ensure only the owner can access them.

**Prevention:** Always restrict permissions on sensitive files or IPC endpoints (like Unix sockets or named pipes) to the principle of least privilege. In Rust on Unix, use `tokio::fs::set_permissions` or `std::os::unix::fs::PermissionsExt` to enforce an explicit mode like `0o600`.

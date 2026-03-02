## 2024-03-02 - [Insecure Unix Socket Permissions in MCP Daemon]
**Vulnerability:** The MCP Daemon created a Unix socket (`UnixListener::bind`) without explicitly setting restrictive file permissions, relying on the system umask. This could allow other local users to connect to the daemon and interact with it.
**Learning:** Even internal IPC mechanisms like Unix domain sockets need explicit permission controls, as default system umasks are often `022` (resulting in `0o755` permissions), which is too permissive for sensitive daemon communication.
**Prevention:** Always follow `UnixListener::bind` with explicit permission setting (e.g., `0o600`) using `std::os::unix::fs::PermissionsExt` to restrict access to the owner only.

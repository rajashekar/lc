## 2026-02-24 - [MCP Daemon Socket Permissions]
**Vulnerability:** The MCP daemon created a Unix domain socket without explicitly restricting file permissions. This meant the socket could be accessible to other users on the system (depending on umask), allowing them to connect to the daemon and execute arbitrary tools.
**Learning:** `UnixListener::bind` does not set restrictive permissions by default. When creating IPC mechanisms like sockets, always explicitly set permissions to `0o600` (owner only) immediately after creation to prevent unauthorized access.
**Prevention:** Use `std::os::unix::fs::PermissionsExt` to set the mode of the socket file to `0o600` immediately after binding.

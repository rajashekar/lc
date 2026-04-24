## 2026-04-06 - Fix TOCTOU vulnerability in search config
**Vulnerability:** Time-Of-Check to Time-Of-Use (TOCTOU) race conditions in `src/search/config.rs` when loading and saving configuration files. Using `.exists()` before `fs::read_to_string` or `fs::create_dir_all` allowed the file system state to change between the check and the use.
**Learning:** Checking if a file or directory exists before acting on it is a common pattern that introduces TOCTOU vulnerabilities. For example, if a file is deleted between the `exists()` check and the `read_to_string` call, the read will fail.
**Prevention:** Unconditionally perform the desired file system operation (`fs::read_to_string` or `fs::create_dir_all`) and handle the resulting errors directly (e.g., checking for `std::io::ErrorKind::NotFound`). This atomic approach prevents race conditions.

## 2025-04-24 - Configuration File Permission Gap
**Vulnerability:** Configuration files containing sensitive secrets (e.g. `auth_token` for WebChat proxy, MCP environment variables) were created with default file permissions (`fs::write`), allowing any local user on the machine to potentially read sensitive details.
**Learning:** Even if the directory itself is managed, creating files via standard `fs::write` or `tokio::fs::write` does not enforce restrictive read/write settings. On Unix systems, this defaults to the system's umask which might be overly permissive (like 0644).
**Prevention:** For any configuration file containing sensitive data, explicitly enforce restrictive permissions (`0o600`) using `std::fs::OpenOptions` or `tokio::fs::OpenOptions` alongside `std::os::unix::fs::OpenOptionsExt` before creating or opening the file, and apply `set_permissions` as a fallback.

## 2026-04-06 - Fix TOCTOU vulnerability in search config
**Vulnerability:** Time-Of-Check to Time-Of-Use (TOCTOU) race conditions in `src/search/config.rs` when loading and saving configuration files. Using `.exists()` before `fs::read_to_string` or `fs::create_dir_all` allowed the file system state to change between the check and the use.
**Learning:** Checking if a file or directory exists before acting on it is a common pattern that introduces TOCTOU vulnerabilities. For example, if a file is deleted between the `exists()` check and the `read_to_string` call, the read will fail.
**Prevention:** Unconditionally perform the desired file system operation (`fs::read_to_string` or `fs::create_dir_all`) and handle the resulting errors directly (e.g., checking for `std::io::ErrorKind::NotFound`). This atomic approach prevents race conditions.

## 2024-05-28 - Insecure File Permissions for Configuration Files Containing Secrets
**Vulnerability:** The application was writing sensitive S3 credentials (`secret_access_key`) to `sync.toml` using `fs::write()`, which creates the file with default system permissions (often `0o644` or similar, depending on umask). This allows other users on the system to read the file and access the credentials.
**Learning:** Even when configuration files seem innocuous, if they can contain sensitive data like API keys or credentials, their file permissions must be strictly controlled at creation time to prevent unauthorized access.
**Prevention:** Use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` (on Unix platforms) to explicitly set restrictive permissions (e.g., `0o600` for read/write only by the owner) when creating or opening the file for writing.

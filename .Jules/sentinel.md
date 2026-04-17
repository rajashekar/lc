## 2026-04-06 - Fix TOCTOU vulnerability in search config
**Vulnerability:** Time-Of-Check to Time-Of-Use (TOCTOU) race conditions in `src/search/config.rs` when loading and saving configuration files. Using `.exists()` before `fs::read_to_string` or `fs::create_dir_all` allowed the file system state to change between the check and the use.
**Learning:** Checking if a file or directory exists before acting on it is a common pattern that introduces TOCTOU vulnerabilities. For example, if a file is deleted between the `exists()` check and the `read_to_string` call, the read will fail.
**Prevention:** Unconditionally perform the desired file system operation (`fs::read_to_string` or `fs::create_dir_all`) and handle the resulting errors directly (e.g., checking for `std::io::ErrorKind::NotFound`). This atomic approach prevents race conditions.

## 2024-05-18 - Insecure File Permissions Exposing AWS Secrets
**Vulnerability:** The sync configuration file `sync.toml` (which stores sensitive AWS credentials like `secret_access_key` for S3 sync) was created using the standard `fs::write()` function. This applies default umask permissions, typically making the file world-readable (`0o644`) on Unix-like systems and exposing secrets to unauthorized local users.
**Learning:** Standard library file writing functions in Rust do not restrict permissions by default. Relying on default umask is unsafe when writing configuration files containing secrets or authentication tokens.
**Prevention:** Always use `std::fs::OpenOptions` along with `std::os::unix::fs::OpenOptionsExt` to explicitly set secure file modes (e.g., `0o600`) before file creation to prevent TOCTOU vulnerabilities and ensure secrets remain protected from unauthorized local access.

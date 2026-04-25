## 2026-04-06 - Fix TOCTOU vulnerability in search config
**Vulnerability:** Time-Of-Check to Time-Of-Use (TOCTOU) race conditions in `src/search/config.rs` when loading and saving configuration files. Using `.exists()` before `fs::read_to_string` or `fs::create_dir_all` allowed the file system state to change between the check and the use.
**Learning:** Checking if a file or directory exists before acting on it is a common pattern that introduces TOCTOU vulnerabilities. For example, if a file is deleted between the `exists()` check and the `read_to_string` call, the read will fail.
**Prevention:** Unconditionally perform the desired file system operation (`fs::read_to_string` or `fs::create_dir_all`) and handle the resulting errors directly (e.g., checking for `std::io::ErrorKind::NotFound`). This atomic approach prevents race conditions.

## 2024-05-18 - Fix insecure file permissions for config files
**Vulnerability:** Configuration files (`config.toml`, provider configs) were being written using `fs::write`, which uses default file permissions. These files often contain sensitive API keys and tokens. Writing them with world-readable permissions could allow other local users to extract the keys.
**Learning:** Default file creation permissions in Rust (often `0o666` modified by umask) are too permissive for files storing secrets. Explicit, restrictive permissions (`0o600`) must be used for configuration and credential files to prevent unauthorized local access.
**Prevention:** When writing sensitive configuration files, especially on Unix-like systems, always use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` to explicitly set the mode to `0o600` (read/write by owner only). Do not rely on `fs::write` for files containing secrets.

## 2024-06-01 - Path Traversal vulnerabilities from PathBuf::join

**Vulnerability:** Found missing validation when joining external/untrusted file names with base directories using `PathBuf::join`. This could allow path traversal and arbitrary file write if an attacker controls the filename, such as in `src/sync/sync.rs`, `src/core/provider_installer.rs`, `src/models/dump_metadata.rs`, etc.
**Learning:** In Rust, `PathBuf::join` completely replaces the base path if the appended string is an absolute path (e.g., starts with `/` or `\`). Always validate untrusted/external filenames.
**Prevention:** Explicitly check `!name.contains("..") && !name.starts_with('/') && !name.starts_with('\\')` before joining them to a base directory to prevent path traversal and arbitrary file write vulnerabilities.

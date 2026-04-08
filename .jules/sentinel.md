## 2025-02-28 - [MEDIUM] TOCTOU in File/Directory Creation

**Vulnerability:** A recurring Time-Of-Check to Time-Of-Use (TOCTOU) pattern where `.exists()` is called before file operations (like `tokio::fs::remove_file()` or creating directories).
**Learning:** This introduces a race condition because the filesystem state can change between the check and the actual operation, leading to errors or security issues (e.g., symlink attacks).
**Prevention:** Avoid using `.exists()`. Instead, perform the operation directly and handle standard `std::io::ErrorKind` errors (e.g., `NotFound`, `AlreadyExists`) via `match` blocks.

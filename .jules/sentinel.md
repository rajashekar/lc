## 2024-05-18 - Fix TOCTOU vulnerability patterns
**Vulnerability:** Checking file existence before acting on it using `.exists()` created Time-Of-Check to Time-Of-Use (TOCTOU) race conditions in `mcp_daemon.rs` and `provider_installer.rs`.
**Learning:** These race conditions can lead to failed file operations and unhandled errors when the file state changes between the check and the action. Standard library ErrorKind matching is the safer, standard approach.
**Prevention:** Always perform file system operations directly and handle specific `io::ErrorKind` (like `NotFound` or `AlreadyExists`) rather than checking state beforehand.

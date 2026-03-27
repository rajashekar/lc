## 2024-05-01 - Fix TOCTOU vulnerability in mcp daemon socket creation
**Vulnerability:** In `src/services/mcp_daemon.rs`, the code checked if a Unix socket file existed before attempting to delete it (`if path.exists() { remove_file() }`), creating a Time-Of-Check to Time-Of-Use (TOCTOU) race condition.
**Learning:** Checking for file existence before deletion is an anti-pattern that can lead to race conditions. The file system state can change between the check and the action.
**Prevention:** Execute the file deletion directly and gracefully handle the `NotFound` error if the file doesn't exist, e.g., using a `match` statement on the result of `remove_file()`.

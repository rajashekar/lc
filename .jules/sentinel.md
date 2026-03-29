## 2024-05-24 - Fix TOCTOU vulnerability in file deletion
**Vulnerability:** The codebase had instances where file deletion was guarded by a `.exists()` check (e.g., in `mcp_daemon.rs`). This is a Time-of-Check to Time-of-Use (TOCTOU) vulnerability where a file could be created or removed by another process between the check and the actual removal, leading to unexpected behavior or errors.
**Learning:** Checking for file existence before performing an operation on it is inherently unsafe in multi-processing environments. Using `.exists()` should generally be avoided for such file operations.
**Prevention:** Perform the action directly (e.g., `remove_file`) and explicitly handle expected errors like `std::io::ErrorKind::NotFound` to ensure atomic operations.

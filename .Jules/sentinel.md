## 2024-05-30 - TOCTOU File Operations Pattern

**Vulnerability:** A pervasive Time-Of-Check to Time-Of-Use (TOCTOU) vulnerability pattern was observed where file existence is checked with `.exists()` prior to file creation or deletion. In `mcp_daemon.rs`, checking if a socket exists before deleting it or creating its parent directory allowed a window for race conditions.

**Learning:** This is a common pattern in the codebase, particularly observed across `provider_installer.rs`, `search/config.rs`, and caching layers where `std::fs::remove_file` or `create_dir` logic is wrapped in `.exists()` checks.

**Prevention:** Never use `.exists()` as a precondition for file modification operations (removal, creation). Instead, execute the desired operation directly and handle the resulting standard `std::io::ErrorKind` (like `NotFound` for removal or `AlreadyExists` for creation). Additionally, for sockets or sensitive directories, always verify that the path is not a symlink after creation using `symlink_metadata` to mitigate symlink attacks.

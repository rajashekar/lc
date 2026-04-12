## 2024-05-24 - Fix Path Traversal in Vector Database Name
**Vulnerability:** Path traversal sequence in user-supplied `name` used in database path (`{name}.db`), potentially resulting in creation/deletion of `.db` files outside the expected `embeddings` directory.
**Learning:** `PathBuf::join` replaces the current path completely if the argument contains a leading slash or can traverse directories if the argument contains `../`. Always sanitize parameters used to construct paths, even if they're supposedly from internal trusted configurations or users.
**Prevention:** Implement an explicit validation helper function (`validate_name`) to check for any occurrences of `/`, `\`, and `..` before path construction and abort early.

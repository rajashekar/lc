## 2024-06-03 - Path Traversal in Model Metadata Caching
**Vulnerability:** Path traversal vulnerability due to using unsanitized user-controlled input (`provider_name`) to construct file paths for reading and writing JSON files within the `models/` directory.
**Learning:** Even internal configuration keys or identifiers can be sources of vulnerability if they originate from user input (e.g., config files) and are used directly in file system operations.
**Prevention:** Always sanitize input used in file paths or explicitly check for and reject input containing path traversal characters like `..`, `/`, and `\`.

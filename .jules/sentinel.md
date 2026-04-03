## 2024-05-18 - Path Traversal in model caching
**Vulnerability:** Path traversal in `models/{}.json` format string. If `provider_name` contains path separators, it allows an attacker to read/write arbitrary files outside the `models` directory.
**Learning:** Found string formatting used for paths that combine static path prefixes with unvalidated user input or variable strings.
**Prevention:** Always validate parameters used to construct file paths to ensure they do not contain path separators (`/`, `\`) or parent references (`..`).

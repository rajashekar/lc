## 2024-05-24 - Overly Permissive CORS Configuration

**Vulnerability:** The proxy server unconditionally enabled `CorsLayer::permissive()`, which disabled CORS protections entirely.
**Learning:** Axum proxy servers should not apply permissive CORS by default, as it creates a Cross-Site Request Forgery (CSRF) risk.
**Prevention:** Only conditionally apply `CorsLayer::permissive()` if explicitly enabled by the user via a CLI flag or configuration.

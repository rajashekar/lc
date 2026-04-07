## 2024-05-18 - Restrict Permissive CORS Default
**Vulnerability:** Axum proxy servers like `start_proxy_server` in `src/services/proxy.rs` and `start_webchatproxy_server` in `src/services/webchatproxy.rs` were unconditionally applying `CorsLayer::permissive()`, disabling CORS entirely and exposing local services to CSRF-style attacks from arbitrary external websites.
**Learning:** Security features like CORS should not be disabled by default for developer convenience. It must be explicitly opted-in via a CLI flag to minimize the attack surface.
**Prevention:** Require a `--cors` CLI flag to conditionally enable `CorsLayer::permissive()` in all local Axum servers. Ensure both CLI and service layers pass and respect this parameter.

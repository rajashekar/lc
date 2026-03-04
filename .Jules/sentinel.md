## 2024-03-04 - Insecure Default CORS Configuration in Proxy

**Vulnerability:** The proxy server implementation (`src/services/proxy.rs`) applied `CorsLayer::permissive()` indiscriminately, allowing unrestricted cross-origin requests. This overly permissive CORS configuration was enabled by default, presenting a significant security risk for any deployment where the proxy might be exposed.

**Learning:** When developing network services or proxies, CORS must be disabled by default or configured with strict origins. Reusing permissive defaults from development stages in production-ready proxy commands can easily lead to CSRF vulnerabilities and data leakage across domains.

**Prevention:** Ensure that all network service endpoints adhere to the principle of least privilege regarding CORS. Introduce explicit opt-in flags (like `--cors`) for permissive cross-origin resource sharing, rather than hardcoding `.layer(CorsLayer::permissive())` globally across routes.

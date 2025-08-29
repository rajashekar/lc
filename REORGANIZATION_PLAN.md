# Source Code Reorganization Plan

## Current Structure Issues
- 30+ files at the root of `src/`
- Mixed concerns (utils, core features, data, services)
- Hard to navigate and understand the architecture

## Proposed New Structure

```
src/
├── main.rs          # Binary entry point (stays)
├── lib.rs           # Library entry point (stays)
├── error.rs         # Global error types (stays)
├── errors.rs        # Additional errors (merge with error.rs)
│
├── core/            # Core functionality
│   ├── mod.rs
│   ├── chat.rs
│   ├── completion.rs
│   ├── provider.rs
│   ├── provider_installer.rs
│   └── http_client.rs
│
├── data/            # Data storage and configuration
│   ├── mod.rs
│   ├── database.rs
│   ├── vector_db.rs
│   ├── keys.rs
│   └── config.rs
│
├── models/          # Model-related functionality
│   ├── mod.rs
│   ├── metadata.rs (from model_metadata.rs)
│   ├── cache.rs (from models_cache.rs)
│   ├── unified_cache.rs
│   └── dump_metadata.rs
│
├── services/        # Server/daemon services
│   ├── mod.rs
│   ├── proxy.rs
│   ├── webchatproxy.rs
│   ├── mcp.rs
│   └── mcp_daemon.rs
│
├── utils/           # Utility functions
│   ├── mod.rs
│   ├── audio.rs (from audio_utils.rs)
│   ├── image.rs (from image_utils.rs)
│   ├── token.rs (from token_utils.rs)
│   ├── template_processor.rs
│   ├── input.rs
│   └── test.rs (from test_utils.rs)
│
├── analytics/       # Usage tracking
│   ├── mod.rs
│   └── usage_stats.rs
│
├── cli/             # CLI commands (existing, already organized)
├── readers/         # File readers (existing)
├── search/          # Search providers (existing)
└── sync/            # Sync functionality (existing)
```

## Migration Steps

1. Create new directories
2. Move files to appropriate directories
3. Create mod.rs files for each module
4. Update all import statements throughout the codebase
5. Remove cli_old.rs after completing command migrations
6. Test compilation and fix any issues

## Benefits
- Clear separation of concerns
- Easier navigation
- Better code organization
- More intuitive for new contributors
- Follows Rust best practices
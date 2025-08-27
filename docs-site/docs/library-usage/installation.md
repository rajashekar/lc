# Installation

Learn how to add LC as a dependency to your Rust project.

## Adding the Dependency

Add LC to your `Cargo.toml` file:

```toml
[dependencies]
lc-cli = "0.1.1"
```

:::info Package Name
Note that the package name is `lc-cli` (as defined in the `[package]` section), not just `lc`.
:::

## With Specific Features

LC supports optional features that you can enable based on your needs:

```toml
[dependencies]
lc-cli = { version = "0.1.1", features = ["unix-sockets", "pdf"] }
```

### Available Features

| Feature | Description | Default | Platforms |
|---------|-------------|---------|-----------|
| `unix-sockets` | Unix domain socket support for MCP daemon | ✅ (Unix only) | Linux, macOS, WSL2 |
| `pdf` | PDF processing capabilities | ✅ | All platforms |

### Platform-Specific Builds

#### Unix Systems (Linux, macOS, WSL2)
```toml
[dependencies]
lc-cli = "0.1.1"  # All features enabled by default
```

#### Windows (without Unix sockets)
```toml
[dependencies]
lc-cli = { version = "0.1.1", default-features = false, features = ["pdf"] }
```

#### Minimal Build (no optional features)
```toml
[dependencies]
lc-cli = { version = "0.1.1", default-features = false }
```

## Async Runtime

LC is built on `tokio`, so your project needs an async runtime. Add this to your `Cargo.toml`:

```toml
[dependencies]
lc-cli = "0.1.1"
tokio = { version = "1.35", features = ["full"] }
```

And use `#[tokio::main]` in your main function:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here
    Ok(())
}
```

## Verification

Create a simple test to verify the installation:

```rust
// src/main.rs
use lc_cli::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to load the configuration
    match Config::load() {
        Ok(config) => println!("LC library loaded successfully!"),
        Err(e) => println!("Config load failed (expected if not configured): {}", e),
    }
    Ok(())
}
```

Run with:
```bash
cargo run
```

If you see "LC library loaded successfully!" or a config-related error (which is normal if you haven't set up providers yet), the library is installed correctly.

## Next Steps

- [**Basic Usage**](basic-usage.md) - Write your first LC library code
- [**Configuration**](configuration.md) - Set up providers and API keys

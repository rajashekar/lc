---
id: installation
title: Installation
sidebar_position: 1
---

# Installation

LLM Client can be installed from source or using pre-built binaries. Choose the method that works best for your system.

## Prerequisites

### Install Rust

LLM Client is written in Rust, so you'll need the Rust toolchain installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify the installation:

```bash
rustc --version
cargo --version
```

## Installation Methods

### 🚀 Quick Install (Coming Soon)

**Note**: Package manager installations are planned for future releases. Currently, install from source or use cargo.

```bash
# macOS
brew install lc                    # 🚧 Coming soon

# Windows
scoop install lc                   # 🚧 Coming soon
choco install lc                   # 🚧 Coming soon
winget install lc                  # 🚧 Coming soon

# Linux
apt install lc                     # 🚧 Coming soon
dnf install lc                     # 🚧 Coming soon
yay -S lc                          # 🚧 Coming soon (AUR)

# Universal (Rust users)
cargo install lc-cli               # 🚧 Coming soon
```

### 📦 From Cargo (Rust Users)

**Note**: Package will be published to crates.io soon.

```bash
# Install from crates.io (when published)
cargo install lc-cli

# Or install directly from git (current method)
cargo install --git https://github.com/your-username/lc
```

### 📁 Pre-built Binaries (Recommended)

**Note**: GitHub releases with pre-built binaries are coming soon.

1. Download the latest binary for your platform from [GitHub Releases](https://github.com/your-username/lc/releases)
2. Extract and place in your PATH

| Platform | Architecture | Binary Name |
|----------|-------------|-------------|
| **Linux** | x86_64 | `lc-linux-amd64` |
| **Linux** | ARM64 | `lc-linux-arm64` |
| **macOS** | Intel | `lc-macos-amd64` |
| **macOS** | Apple Silicon | `lc-macos-arm64` |
| **Windows** | x86_64 | `lc-windows-amd64.exe` |

### 🔨 From Source (Current Method)

1. Clone the repository:

```bash
git clone <repository-url>
cd lc
```

2. Build the release binary:

```bash
# Full build with all features (including PDF support)
cargo build --release

# Or build without PDF support to reduce dependencies
cargo build --release --no-default-features
```

3. The binary will be available at `target/release/lc`

### Add to PATH

To use `lc` from anywhere on your system, add it to your PATH:

#### Linux/macOS

```bash
# Option 1: Copy to system bin directory
sudo cp target/release/lc /usr/local/bin/

# Option 2: Copy to user bin directory
cp target/release/lc ~/.local/bin/

# Option 3: Add the target directory to PATH
echo 'export PATH="$PATH:/path/to/lc/target/release"' >> ~/.bashrc
source ~/.bashrc
```

#### Windows

```powershell
# Copy to a directory in your PATH
copy target\release\lc.exe C:\Windows\System32\

# Or add the directory to PATH via System Properties
```

## Platform-Specific Builds

### Cross-Compilation

You can build for different platforms:

```bash
# Windows from Linux/macOS
cargo build --target x86_64-pc-windows-gnu --release

# macOS Intel
cargo build --target x86_64-apple-darwin --release

# macOS Apple Silicon
cargo build --target aarch64-apple-darwin --release

# Linux x86_64
cargo build --target x86_64-unknown-linux-gnu --release

# Linux ARM64
cargo build --target aarch64-unknown-linux-gnu --release
```

## Verify Installation

After installation, verify that `lc` is working:

```bash
# Check version
lc --version

# Show help
lc --help

# List providers (should show empty list on first run)
lc providers list
```

## Configuration Directories

LLM Client stores its configuration and data in platform-specific locations:

| Platform | Directory |
|----------|-----------|
| **Linux** | `~/.config/lc/` |
| **macOS** | `~/Library/Application Support/lc/` |
| **Windows** | `%APPDATA%\lc\` |

These directories are created automatically on first run.

### Building with Specific Features

```bash
# Build with all default features (includes PDF support)
cargo build --release

# Build without any optional features
cargo build --release --no-default-features

# Build with specific features only
cargo build --release --no-default-features --features pdf

# Build for distribution (smaller binary without PDF support)
cargo build --release --no-default-features
```

## Next Steps

Now that you have `lc` installed, proceed to the [Quick Start Guide](/getting-started/quick-start) to configure your first provider and start using the tool.

## Troubleshooting

### Rust Installation Issues

If you encounter issues installing Rust:

- Ensure you have a C compiler installed (gcc, clang, or MSVC)
- On Windows, you may need Visual Studio Build Tools
- Check the [Rust installation guide](https://www.rust-lang.org/tools/install) for platform-specific help

### Build Errors

Common solutions:

- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build --release`
- Check for missing system dependencies

### Permission Denied

If you get permission errors when copying to system directories:

- Use `sudo` on Linux/macOS
- Run as Administrator on Windows
- Or choose a user-writable directory instead

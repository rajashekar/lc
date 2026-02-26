---
id: installation
title: Installation
sidebar_position: 1
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# Installation

LLM Client can be installed from source or using pre-built binaries. Choose the method that works best for your system.

## Prerequisites

### System Dependencies

Before installing LLM Client, you need to install system dependencies required for building Rust applications that use OpenSSL and native libraries.

<Tabs>
  <TabItem value="linux" label="Linux" default>

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y pkg-config libssl-dev build-essential
```

**RHEL/CentOS/Fedora:**
```bash
# RHEL/CentOS
sudo yum install -y pkgconfig openssl-devel gcc

# Fedora
sudo dnf install -y pkgconf-devel openssl-devel gcc
```

**Alpine Linux:**
```bash
sudo apk add pkgconf openssl-dev build-base
```

  </TabItem>
  <TabItem value="macos" label="macOS">

On macOS, Xcode Command Line Tools usually provide the necessary dependencies:

```bash
xcode-select --install
```

If you encounter OpenSSL-related build errors, install additional dependencies via Homebrew:

```bash
brew install pkg-config openssl@3
```

  </TabItem>
  <TabItem value="windows" label="Windows">

On Windows, Rust typically bundles OpenSSL statically, so no additional system packages are required. However, you'll need:

- **Visual Studio Build Tools** or **Visual Studio Community** with C++ support
- **Windows SDK**

You can install these via the [Visual Studio Installer](https://visualstudio.microsoft.com/downloads/).

  </TabItem>
</Tabs>

#### Why These Dependencies?

LLM Client uses several Rust crates that require native system libraries:

- **pkg-config**: Helps the build system locate installed libraries and their configuration
- **OpenSSL development libraries**: Required for HTTPS/TLS connections to AI providers
- **Build tools**: C compiler and linker for compiling native dependencies

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

### 🚀 One-Liner Install Script (Recommended)

**The fastest way to get started on any platform!**

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/rajashekar/lc/main/install.sh | bash
```

**Customization options:**

```bash
# Install to custom directory
LC_BIN_DIR=~/.bin curl -fsSL https://raw.githubusercontent.com/rajashekar/lc/main/install.sh | bash

# Install specific version
LC_VERSION=v0.1.1 curl -fsSL https://raw.githubusercontent.com/rajashekar/lc/main/install.sh | bash

# Force overwrite existing installation
LC_FORCE=true curl -fsSL https://raw.githubusercontent.com/rajashekar/lc/main/install.sh | bash
```

**Supported platforms:**
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon) 
- Windows (WSL, WSL2, Git Bash, MSYS2)

**Features:**
- ✅ Automatic platform and architecture detection
- ✅ Downloads pre-built binaries from GitHub releases
- ✅ Installs to `$HOME/.local/bin` by default
- ✅ Provides PATH setup instructions
- ✅ Verifies installation

### 📦 From Cargo (Rust Users)

```bash
# Install from crates.io (published!)
cargo install lc-cli

# Or install directly from git (development version)
cargo install --git https://github.com/rajashekar/lc
```

### 📁 Manual Binary Download

1. Download the latest binary for your platform from [GitHub Releases](https://github.com/rajashekar/lc/releases)
2. Extract and place in your PATH

| Platform | Architecture | Binary Name |
|----------|-------------|-------------|
| **Linux** | x86_64 | `lc-linux-x86_64.tar.gz` |
| **Linux** | ARM64 | `lc-linux-arm64.tar.gz` |
| **macOS** | Intel | `lc-macos-x86_64.tar.gz` |
| **macOS** | Apple Silicon | `lc-macos-arm64.tar.gz` |
| **Windows** | x86_64 | `lc-windows-amd64.zip` |

### 🚧 Package Managers (Coming Soon)

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
```

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

<Tabs>
  <TabItem value="unix" label="Linux/macOS" default>

```bash
# Option 1: Copy to system bin directory
sudo cp target/release/lc /usr/local/bin/

# Option 2: Copy to user bin directory
cp target/release/lc ~/.local/bin/

# Option 3: Add the target directory to PATH
echo 'export PATH="$PATH:/path/to/lc/target/release"' >> ~/.bashrc
source ~/.bashrc
```

  </TabItem>
  <TabItem value="windows" label="Windows">

```powershell
# Copy to a directory in your PATH
copy target\release\lc.exe C:\Windows\System32\

# Or add the directory to PATH via System Properties
```

  </TabItem>
</Tabs>

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

### System Dependencies Issues

If you encounter build errors related to OpenSSL or pkg-config, ensure you have installed the system dependencies listed above. See the [Troubleshooting Guide](/troubleshooting) for detailed solutions.

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

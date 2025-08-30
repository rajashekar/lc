<div align="center">

[![Documentation](https://img.shields.io/badge/docs-lc.viwq.dev-blue)](https://lc.viwq.dev)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)](https://www.rust-lang.org)
[![CI](https://github.com/rajashekar/lc/workflows/Test%20Linux%20Build/badge.svg)](https://github.com/rajashekar/lc/actions)
[![Crates.io](https://img.shields.io/crates/v/lc-cli.svg)](https://crates.io/crates/lc-cli)
[![Downloads](https://img.shields.io/crates/d/lc-cli.svg)](https://crates.io/crates/lc-cli)
[![GitHub release](https://img.shields.io/github/release/rajashekar/lc.svg)](https://github.com/rajashekar/lc/releases)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)](https://github.com/rajashekar/lc)

# LLM Client (lc)

<img src="docs-site/static/img/social-card.png" alt="LLM Client" width="450" />

A fast, Rust-based command-line tool for interacting with Large Language Models.

</div>

## Quick Start

### Installation

```bash
# Option 1: One-liner install script (recommended)
curl -fsSL https://raw.githubusercontent.com/rajashekar/lc/main/install.sh | bash

# Option 2: Install from crates.io
cargo install lc-cli

# Option 3: Install from source
git clone https://github.com/rajashekar/lc.git
cd lc
cargo build --release

# Add a provider
lc providers add openai https://api.openai.com/v1
or
lc providers install openai

# Set your API key
lc keys add openai

# Start chatting
lc -m openai:gpt-4 "What is the capital of France?"
or
# set default provider and model
lc config set provider openai
lc config set model gpt-4
# Direct prompt with specific model
lc "What is the capital of France?"
```


## System Requirements

**Before building from source**, ensure you have the required system dependencies:

- **Linux (Ubuntu/Debian)**: `sudo apt install -y pkg-config libssl-dev build-essential`
- **Linux (RHEL/CentOS/Fedora)**: `sudo yum install -y pkgconfig openssl-devel gcc` (or `dnf`)
- **macOS**: `xcode-select --install` (+ Homebrew if needed: `brew install pkg-config openssl@3`)
- **Windows**: Visual Studio Build Tools with C++ support

These dependencies are required for Rust crates that link against OpenSSL and native libraries.

📖 **Full installation instructions**: [Installation Guide](https://lc.viwq.dev/getting-started/installation)
🔧 **Having build issues?** See [Troubleshooting Guide](https://lc.viwq.dev/troubleshooting)
## Key Features

- 🚀 **Lightning Fast** - ~3ms cold start (50x faster than Python alternatives)
- 🔧 **Universal** - Works with any OpenAI-compatible API
- 🧠 **Smart** - Built-in vector database and RAG support
- 🛠️ **Tools** - Model Context Protocol (MCP) support for extending LLM capabilities
- 🔍 **Web Search** - Integrated web search with multiple providers (Brave, Exa, Serper) for enhanced context
- 👁️ **Vision Support** - Process and analyze images with vision-capable models
- � **PDF Support** - Read and process PDF files with optional dependency
- 🔐 **Secure** - Encrypted configuration sync
- 💬 **Intuitive** - Simple commands with short aliases
- 🎨 **Flexible Templates** - Configure request/response formats for any LLM API
- ⚡ **Shell Completion** - Tab completion for commands, providers, models, and more

## Documentation

For comprehensive documentation, visit **[lc.viwq.dev](https://lc.viwq.dev)**

### Quick Links

- [Installation Guide](https://lc.viwq.dev/getting-started/installation)
- [Quick Start Tutorial](https://lc.viwq.dev/getting-started/quick-start)
- [Command Reference](https://lc.viwq.dev/commands/overview)
- [Provider Setup](https://lc.viwq.dev/features/providers)
- [Vector Database & RAG](https://lc.viwq.dev/advanced/vector-database)
- [Model Context Protocol (MCP)](https://lc.viwq.dev/advanced/mcp)
- [Template System](docs/TEMPLATE_SYSTEM.md) - Configure custom request/response formats

## Example Usage

```bash
# Direct prompt with specific model
lc -m openai:gpt-4 "Explain quantum computing"

# Interactive chat session
lc chat -m anthropic:claude-3.5-sonnet

# find embedding models
lc models embed
or
lc m e

# create embeddings for your text
lc embed -m text-embedding-3-small -v knowledge "Machine learning is a subset of AI"
lc embed -m text-embedding-3-small -v knowledge "Deep learning uses neural networks"
lc embed -m text-embedding-3-small -v knowledge "Python is popular for data science"

# Embed files with intelligent chunking
lc embed -m text-embedding-3-small -v docs -f README.md
lc embed -m text-embedding-3-small -v docs -f "*.md"
lc embed -m text-embedding-3-small -v docs -f "/path/to/docs/*.txt"

# above will create a vector db with knowledge
# you can get all vector dbs by using below command
lc vectors list

## to get details of the vector db
lc vectors stats knowledge

# Search similar content
lc similar -v knowledge "What is neural network programming?"

# RAG-enhanced chat
lc chat -v knowledge -m openai:gpt-4
lc -m openai:gpt-4 -v knowledge "Explain the relationship between AI and programming languages"

# Use MCP tools for internet access
lc -t fetch "What's the latest news about AI?"

# Multiple MCP tools
lc -t fetch,playwright "Navigate to example.com and analyze its content"

# Web search integration
lc --use-search brave "What are the latest developments in quantum computing?"

# Search with specific query
lc --use-search "brave:quantum computing 2024" "Summarize the findings"

# Generate images from text prompts
lc image "A futuristic city with flying cars" -m dall-e-3 -s 1024x1024
lc img "Abstract art with vibrant colors" -c 2 -o ./generated_images
```

### Web Search Integration

`lc` supports web search integration to enhance prompts with real-time information:

```bash
# Configure Brave Search
lc search provider add brave https://api.search.brave.com/res/v1/web/search -t brave
lc search provider set brave X-Subscription-Token YOUR_API_KEY

# Configure Exa (AI-powered search)
lc search provider add exa https://api.exa.ai -t exa
lc search provider set exa x-api-key YOUR_API_KEY

# Configure Serper (Google Search API)
lc search provider add serper https://google.serper.dev -t serper
lc search provider set serper X-API-KEY YOUR_API_KEY

# Set default search provider
lc config set search brave

# Direct search
lc search query brave "rust programming language" -f json
lc search query exa "machine learning best practices" -n 10
lc search query serper "latest AI developments" -f md

# Use search results as context
lc --use-search brave "What are the latest AI breakthroughs?"
lc --use-search exa "Explain transformer architecture"
lc --use-search serper "What are the current trends in quantum computing?"

# Search with custom query
lc --use-search "brave:specific search terms" "Analyze these results"
lc --use-search "exa:neural networks 2024" "Summarize recent advances"
lc --use-search "serper:GPT-4 alternatives 2024" "Compare the latest language models"
```

### Image Generation

`lc` supports text-to-image generation using compatible providers:

```bash
# Basic image generation
lc image "A beautiful sunset over mountains"

# Generate with specific model and size
lc image "A futuristic robot" -m dall-e-3 -s 1024x1024

# Generate multiple images
lc image "Abstract geometric patterns" -c 4

# Save to specific directory
lc image "A cozy coffee shop" -o ./my_images

# Use short alias
lc img "A magical forest" -m dall-e-2 -s 512x512

# Generate with specific provider
lc image "Modern architecture" -p openai -m dall-e-3

# Debug mode to see API requests
lc image "Space exploration" --debug
```

**Supported Parameters:**
- `-m, --model`: Image generation model (e.g., dall-e-2, dall-e-3)
- `-p, --provider`: Provider to use (openai, etc.)
- `-s, --size`: Image size (256x256, 512x512, 1024x1024, 1792x1024, 1024x1792)
- `-c, --count`: Number of images to generate (1-10, default: 1)
- `-o, --output`: Output directory for saved images (default: current directory)
- `--debug`: Enable debug mode to see API requests

**Note:** Image generation is currently supported by OpenAI-compatible providers. Generated images are automatically saved with timestamps and descriptive filenames.

### TLS Configuration and Debugging

`lc` uses secure HTTPS connections by default with proper certificate verification. For development and debugging scenarios, you may need to disable TLS verification:

```bash
# macOS/Linux/Unix - Disable TLS certificate verification for development/debugging
# ⚠️  WARNING: Only use this for development with tools like Proxyman, Charles, etc.
LC_DISABLE_TLS_VERIFY=1 lc -m openai:gpt-4 "Hello world"
LC_DISABLE_TLS_VERIFY=1 lc embed -m openai:text-embedding-3-small "test text"
LC_DISABLE_TLS_VERIFY=1 lc chat -m anthropic:claude-3.5-sonnet
```

```cmd
REM Windows Command Prompt
set LC_DISABLE_TLS_VERIFY=1
lc -m openai:gpt-4 "Hello world"
lc embed -m openai:text-embedding-3-small "test text"
```

```powershell
# Windows PowerShell
$env:LC_DISABLE_TLS_VERIFY="1"
lc -m openai:gpt-4 "Hello world"
# or inline:
$env:LC_DISABLE_TLS_VERIFY=1; lc embed -m openai:text-embedding-3-small "test text"
```

**Common Use Cases:**
- **HTTP Debugging Tools**: When using Proxyman, Charles, Wireshark, or similar tools that intercept HTTPS traffic
- **Corporate Networks**: Behind corporate firewalls with custom certificates
- **Development Environments**: Testing with self-signed certificates
- **Local Development**: Working with local API servers without proper certificates

**⚠️ Security Warning**: The `LC_DISABLE_TLS_VERIFY` environment variable should **NEVER** be used in production environments as it disables important security checks that protect against man-in-the-middle attacks.

**Alternative Solutions**:
- **Install Root Certificates**: Install your debugging tool's root certificate in the system keychain
- **Bypass Specific Domains**: Configure your debugging tool to exclude specific APIs from interception
- **Use System Certificates**: Ensure your system's certificate store is up to date

### Vision/Image Support

`lc` supports image inputs for vision-capable models across multiple providers:

```bash
# Single image analysis
lc -m gpt-4-vision-preview -i photo.jpg "What's in this image?"

# Multiple images
lc -m claude-3-opus-20240229 -i before.jpg -i after.jpg "Compare these images"

# Image from URL
lc -m gemini-pro-vision -i https://example.com/image.jpg "Describe this image"

# Interactive chat with images
lc chat -m gpt-4-vision-preview -i screenshot.png

# Find vision-capable models
lc models --vision

# Combine with other features
lc -m gpt-4-vision-preview -i diagram.png -a notes.txt "Explain this diagram with the context from my notes"
```

Supported formats: JPG, PNG, GIF, WebP (max 20MB per image)

### Model Context Protocol (MCP)

`lc` supports MCP servers to extend LLM capabilities with external tools:

```bash
# Add an MCP server
lc mcp add fetch "uvx mcp-server-fetch" --type stdio

# List available functions
lc mcp functions fetch

# Use tools in prompts
lc -t fetch "Get the current weather in Tokyo"

# Interactive chat with tools
lc chat -m gpt-4 -t fetch
```

**Platform Support for MCP Daemon:**
- **Unix systems** (Linux, macOS, WSL2): Full MCP daemon support with persistent connections via Unix sockets (enabled by default with the `unix-sockets` feature)
- **Windows**: MCP daemon functionality is not available due to lack of Unix socket support. Direct MCP connections without the daemon work on all platforms.
- **WSL2**: Full Unix compatibility including MCP daemon support (works exactly like Linux)

To build without Unix socket support:
```bash
cargo build --release --no-default-features --features pdf
```

Learn more about MCP in our [documentation](https://lc.viwq.dev/advanced/mcp).

### File Attachments and PDF Support

`lc` can process and analyze various file types, including PDFs:

```bash
# Attach text files to your prompt
lc -a document.txt "Summarize this document"

# Process PDF files (requires PDF feature)
lc -a report.pdf "What are the key findings in this report?"

# Multiple file attachments
lc -a file1.txt -a data.pdf -a config.json "Analyze these files"

# Combine with other features
lc -a research.pdf -v knowledge "Compare this with existing knowledge"

# Combine images with text attachments
lc -m gpt-4-vision-preview -i chart.png -a data.csv "Analyze this chart against the CSV data"
```

**Note:** PDF support requires the `pdf` feature (enabled by default). To build without PDF support:

```bash
cargo build --release --no-default-features
```

To explicitly enable PDF support:

```bash
cargo build --release --features pdf
```

### Template System

`lc` supports configurable request/response templates, allowing you to work with any LLM API format without code changes:

```toml
# Fix GPT-5's max_completion_tokens and temperature requirement
[chat_templates."gpt-5.*"]
request = """
{
  "model": "{{ model }}",
  "messages": {{ messages | json }}{% if max_tokens %},
  "max_completion_tokens": {{ max_tokens }}{% endif %},
  "temperature": 1{% if tools %},
  "tools": {{ tools | json }}{% endif %}{% if stream %},
  "stream": {{ stream }}{% endif %}
}
"""
```

See [Template System Documentation](docs/TEMPLATE_SYSTEM.md) and [config_samples/templates_sample.toml](config_samples/templates_sample.toml) for more examples.

## Features

`lc` supports several optional features that can be enabled or disabled during compilation:

### Default Features

- `pdf`: Enables PDF file processing and analysis
- `unix-sockets`: Enables Unix domain socket support for MCP daemon (Unix systems only)

### Build Options

```bash
# Build with all default features
cargo build --release

# Build with minimal features (no PDF, no Unix sockets)
cargo build --release --no-default-features

# Build with only PDF support (no Unix sockets)
cargo build --release --no-default-features --features pdf

# Build with only Unix socket support (no PDF)
cargo build --release --no-default-features --features unix-sockets

# Explicitly enable all features
cargo build --release --features "pdf,unix-sockets"
```

**Note:** The `unix-sockets` feature is only functional on Unix-like systems (Linux, macOS, BSD, WSL2). On Windows native command prompt/PowerShell, this feature has no effect and MCP daemon functionality is not available regardless of the feature flag. WSL2 provides full Unix compatibility.

### Windows-Specific Build Information

#### Compilation on Windows

Due to AWS SDK dependencies requiring specific C++ toolchain setup, the recommended build for Windows excludes S3 sync support:

```bash
# Recommended build for Windows (excludes S3 sync to avoid AWS LC compilation issues)
cargo build --release --no-default-features --features "pdf unix-sockets"

# Run tests on Windows
cargo test --no-default-features --features "pdf unix-sockets"
```

If you need S3 sync functionality on Windows, ensure you have:
- Visual Studio 2019 or later with C++ build tools
- Windows SDK installed
- Then build with: `cargo build --release --features s3-sync`

#### Feature Availability

| Feature | Windows | macOS | Linux | WSL2 |
|---------|---------|-------|-------|------|
| MCP Daemon | ❌ | ✅ | ✅ | ✅ |
| Direct MCP | ✅ | ✅ | ✅ | ✅ |
| S3 Sync | ⚠️* | ✅ | ✅ | ✅ |
| PDF Processing | ✅ | ✅ | ✅ | ✅ |
| Vision/Images | ✅ | ✅ | ✅ | ✅ |
| Web Search | ✅ | ✅ | ✅ | ✅ |
| Vector DB/RAG | ✅ | ✅ | ✅ | ✅ |

*S3 Sync on Windows requires Visual Studio C++ build tools and is disabled by default to avoid compilation issues.

## Contributing

Contributions are welcome! Please see our [Contributing Guide](https://lc.viwq.dev/development/contributing).

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

For detailed documentation, examples, and guides, visit **[lc.viwq.dev](https://lc.viwq.dev)**
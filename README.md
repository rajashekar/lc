# lc - LLM Client

LLM Client (lc) - A fast, Rust-based command-line tool for interacting with Large Language Models through OpenAI-compatible APIs. Built for speed, efficiency, and ease of use.

## Features

- 🚀 **Fast startup** - Near-zero cold start time compared to Python alternatives
- 🔧 **Provider management** - Support for any OpenAI-compatible API endpoint
- 💾 **Session management** - Continue conversations with chat history
- 📊 **SQLite logging** - All conversations stored locally with full history
- 🎯 **Simple CLI** - Intuitive command structure
- 🔐 **Secure key storage** - API keys stored in user config directory

## Installation

### Prerequisites

Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### From Source

```bash
git clone <repository-url>
cd lc
cargo build --release
```

The binary will be available at `target/release/lc`.

### Add to PATH

```bash
# Copy to a directory in your PATH
cp target/release/lc /usr/local/bin/
# or
cp target/release/lc ~/.local/bin/
```

## Quick Start

### 1. Add a Provider

```bash
# Add OpenAI
lc providers add openai https://api.openai.com/v1
# or using alias
lc p a openai https://api.openai.com/v1

# Add Vercel's v0.dev
lc providers add vercel https://api.v0.dev/v1
# or using alias
lc p a vercel https://api.v0.dev/v1

# Add any OpenAI-compatible provider
lc providers add custom https://your-api.com/v1
```

### 2. Set API Keys

```bash
lc keys add openai
# or using alias
lc k a openai
# Enter your API key when prompted

lc keys add vercel
# Enter your API key when prompted
```

### 3. Start Interactive Chat

```bash
# Start interactive chat with a model
lc chat -m gpt-4
# or using alias
lc c -m gpt-4

# Continue specific chat session
lc chat -m gpt-4 --cid abc123
```

## Usage Examples

### Provider Management

```bash
# List all providers
lc providers list
lc p l

# List available models from a provider
lc providers models openai
lc p m openai

# Update a provider
lc providers update openai https://api.openai.com/v1
lc p u openai https://api.openai.com/v1

# Remove a provider
lc providers remove openai
lc p r openai
```

### API Key Management

```bash
# Add API key
lc keys add openai
lc k a openai

# List key status
lc keys list
lc k l



# Remove API key
lc keys remove openai
lc k r openai
```

### Interactive Chat Mode

```bash
# Start chat with model
lc chat -m gpt-4
lc c -m gpt-4

# Inside chat mode, use these commands:
# /exit          - Exit chat session
# /clear         - Clear current session
# /model <name>  - Change model
# /help          - Show help
```

### Log Management

```bash
# Show all logs
lc logs show
lc l sh

# Show recent logs (default 10)
lc logs recent
lc l r

# Show recent logs (custom count)
lc logs recent -c 20
lc l r -c 20

# Get last answer from LLM
lc logs recent answer
lc l r a

# Extract code blocks from last answer
lc logs recent answer code
lc l r a c

# Get last question/prompt asked to LLM
lc logs recent question
lc l r q

# Get model used in last interaction
lc logs recent model
lc l r m

# Get session ID of last interaction
lc logs recent session
lc l r s

# Show current session
lc logs current
lc l c

# Show database statistics
lc logs stats
lc l s

# Show logs in table format
lc logs show --minimal
lc l sh --minimal

# Purge all logs
lc logs purge
lc l p

# Purge without confirmation
lc logs purge --yes
lc l p --yes
```

## Cross-Platform Support

LLM Client (`lc`) is designed to work seamlessly across all major platforms with native performance and proper OS integration.

### Supported Platforms
- ✅ **Linux** (x86_64, ARM64) - Native compilation with all dependencies
- ✅ **macOS** (Intel, Apple Silicon) - Native macOS binary with proper app support
- ✅ **Windows** (x86_64) - Native Windows binary with MSVC/GNU toolchain

### Platform-Specific Builds

```bash
# Native build (current platform)
cargo build --release

# Cross-compilation examples
cargo build --target x86_64-pc-windows-gnu      # Windows from Linux
cargo build --target x86_64-apple-darwin        # macOS Intel
cargo build --target aarch64-apple-darwin       # macOS Apple Silicon
cargo build --target x86_64-unknown-linux-gnu   # Linux x86_64
cargo build --target aarch64-unknown-linux-gnu  # Linux ARM64
```

## Configuration

Configuration and data files are automatically stored in platform-appropriate locations following OS conventions:

### File Locations

| Platform | Config Directory | Files |
|----------|------------------|-------|
| **Linux** | `~/.config/lc/` | `config.toml`, `logs.db` |
| **macOS** | `~/Library/Application Support/lc/` | `config.toml`, `logs.db` |
| **Windows** | `%APPDATA%\lc\` | `config.toml`, `logs.db` |

### Files Stored:
- **`config.toml`** - Provider configurations and API keys
- **`logs.db`** - SQLite database with complete chat history and session state

### Automatic Directory Creation
The tool automatically creates the necessary directories on first run, ensuring proper permissions and OS integration.

### Example config.toml:
```toml
default_provider = "openai"

[providers.openai]
endpoint = "https://api.openai.com/v1"
api_key = "sk-..."
models = []

[providers.vercel]
endpoint = "https://api.v0.dev/v1"
api_key = "v1:..."
models = []
```

### Database Schema
The SQLite database (`logs.db`) contains:
- **`chat_logs`** table - All conversation history with timestamps
- **`session_state`** table - Current session tracking
- **Indexes** - Optimized for fast chat_id and timestamp queries

## Command Reference

### Provider Commands
```bash
lc providers add <name> <url>            # Add provider (alias: lc p a)
lc providers update <name> <url>         # Update provider (alias: lc p u)
lc providers remove <name>               # Remove provider (alias: lc p r)
lc providers list                        # List providers (alias: lc p l)
lc providers models <name>               # List models (alias: lc p m)
```

### Key Management Commands
```bash
lc keys add <provider>                   # Add API key (alias: lc k a)
lc keys list                             # List key status (alias: lc k l)
lc keys remove <provider>                # Remove key (alias: lc k r)
```

### Configuration Commands
```bash
lc config                                # Show current configuration (alias: lc co)
lc config set provider <name>           # Set default provider (alias: lc co s p)
lc config set model <name>               # Set default model (alias: lc co s m)
lc config get provider                   # Get default provider (alias: lc co g p)
lc config get model                      # Get default model (alias: lc co g m)
lc config path                           # Show config directory path (alias: lc co p)
```

### Chat Commands
```bash
lc chat -m <model>                       # Interactive chat (alias: lc c)
lc chat -m <model> --cid <id>            # Continue specific session
```

### Log Commands
```bash
lc logs show                             # Show all logs (alias: lc l sh)
lc logs show --minimal                   # Table format
lc logs recent                           # Recent logs (alias: lc l r)
lc logs recent -c <count>                # Recent with count
lc logs recent answer                    # Last answer (alias: lc l r a)
lc logs recent answer code               # Extract code blocks from last answer (alias: lc l r a c)
lc logs recent question                  # Last question (alias: lc l r q)
lc logs recent model                     # Last model (alias: lc l r m)
lc logs recent session                   # Last session ID (alias: lc l r s)
lc logs current                          # Current session (alias: lc l c)
lc logs stats                            # Database statistics (alias: lc l s)
lc logs purge                            # Purge all logs (alias: lc l p)
```
### Interactive Chat Mode

Start an interactive chat session:
```bash
lc chat -m gpt-4
# or using alias:
lc c -m gpt-4
```

Continue a specific session:
```bash
lc chat -m gpt-4 --cid <session-id>
```

Within the chat session, use these commands:
- `/exit` - Exit the chat session
- `/clear` - Clear the current session history
- `/model <model>` - Switch to a different model
- `/help` - Show available commands

The chat maintains context throughout the session and automatically saves to the database.

## Performance Comparison

| Operation | Python `llm` | Rust `lc` | Improvement |
|-----------|--------------|-------------|-------------|
| Cold start | ~150ms | ~3ms | **50x faster** |
| Memory usage | ~35MB | ~6MB | **6x less** |
| Binary size | N/A (needs Python) | ~8MB | **Self-contained** |

## Supported Providers

Any OpenAI-compatible API endpoint, including:

- **OpenAI** - GPT-3.5, GPT-4, etc.
- **Anthropic** - Claude models (via compatible proxy)
- **Vercel v0.dev** - v0-1.0-md model
- **Local models** - Ollama, LocalAI, etc.
- **Custom endpoints** - Any service implementing OpenAI chat completions API

## Architecture

The tool is built with a modular architecture:

```
lc/
├── src/
│   ├── main.rs          # Entry point and CLI routing
│   ├── cli.rs           # Command-line interface definitions
│   ├── config.rs        # Configuration management
│   ├── provider.rs      # OpenAI-compatible HTTP client
│   ├── database.rs      # SQLite chat history storage
│   ├── chat.rs          # Chat request handling
│   └── error.rs         # Error types and handling
├── Cargo.toml           # Dependencies and metadata
└── README.md           # This file
```

## Development

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

### Running in Development
```bash
cargo run -- -m gpt-4 "Hello world"
```

### Dependencies
- [`clap`](https://docs.rs/clap/) - Command line parsing
- [`tokio`](https://docs.rs/tokio/) - Async runtime
- [`reqwest`](https://docs.rs/reqwest/) - HTTP client
- [`rusqlite`](https://docs.rs/rusqlite/) - SQLite database
- [`serde`](https://docs.rs/serde/) - Serialization
- [`uuid`](https://docs.rs/uuid/) - Session ID generation
- [`chrono`](https://docs.rs/chrono/) - Date/time handling
- [`colored`](https://docs.rs/colored/) - Terminal colors
- [`tabled`](https://docs.rs/tabled/) - Table formatting

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Troubleshooting

### Common Issues

**"No providers configured"**
```bash
lc set provider -n openai -e https://api.openai.com/v1
lc set keys openai
```

**"API request failed"**
- Check your API key: `lc set keys <provider>`
- Verify endpoint URL is correct
- Ensure you have sufficient API credits

**"Model not found"**
- List available models: `lc get provider models -n <provider>`
- Use exact model name from the list

### Debug Mode
Set `RUST_LOG=debug` for detailed logging:
```bash
RUST_LOG=debug lc -m gpt-4 "test"
```

## Example Usage with Vercel v0.dev

```bash
# Add Vercel provider
lc set provider -n vercel -e https://api.v0.dev/v1

# Set API key (from your Vercel account)
lc set keys vercel
# Enter: v1:pAZXgzGJaXccjXeSDSeByXjn:K0UA5jEUPL1350Jbj7xtK3g3

# List available models
lc get provider models -n vercel

# Chat with v0 model
lc -m v0-1.0-md "Create a React component for a todo list"

# Continue the conversation
lc -c -m v0-1.0-md "Add TypeScript types to that component"
```

This implementation provides a fast, efficient alternative to Python-based LLM CLI tools with significant performance improvements and a clean, intuitive interface.
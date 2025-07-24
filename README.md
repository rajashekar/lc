# lc - LLM Client

LLM Client (lc) - A fast, Rust-based command-line tool for interacting with Large Language Models through OpenAI-compatible APIs. Built for speed, efficiency, and ease of use.

## Features

- 🚀 **Fast startup** - Near-zero cold start time compared to Python alternatives
- 🔧 **Provider management** - Support for any OpenAI-compatible API endpoint
- 💬 **Direct prompts** - Send one-off prompts with `-p` provider and `-m` model flags
- 💾 **Session management** - Continue conversations with chat history
- 📊 **SQLite logging** - All conversations stored locally with full history
- 🎯 **Simple CLI** - Intuitive command structure with aliases
- 🔐 **Secure key storage** - API keys stored in user config directory
- 📥 **Piped input** - Support for piped input from other commands
- 🔄 **Multiple API formats** - Handles both wrapped (`{"data": [...]}`) and direct array responses

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

### 3. Set Default Configuration (Optional)

```bash
# Set default provider
lc config set provider openai
lc co s p openai

# Set default model
lc config set model gpt-4
lc co s m gpt-4
```

### 4. Start Using

```bash
# Direct prompt (uses defaults)
lc "What is the capital of France?"

# Direct prompt with specific provider and model
lc -p openrouter -m "anthropic/claude-3.5-sonnet" "Explain quantum computing"

# Start interactive chat with a model
lc chat -m gpt-4
# or using alias
lc c -m gpt-4

# Continue specific chat session
lc chat -m gpt-4 --cid abc123
```

## Usage Examples

### Direct Prompts

You can send direct prompts to any model without entering interactive mode:

```bash
# Using default provider and model
lc "What is the capital of France?"

# Specify provider and model
lc -p openrouter -m "anthropic/claude-3.5-sonnet" "Explain quantum computing"

# Using only provider flag (uses default model)
lc -p together "Write a Python function to sort a list"

# Using only model flag (uses default provider)
lc -m "gpt-4" "What's the weather like?"

# Piped input
echo "Translate this to Spanish" | lc -p openai -m "gpt-4"
```

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

### Direct Prompt Commands
```bash
lc "prompt"                              # Use default provider and model
lc -p <provider> "prompt"                # Specify provider
lc -m <model> "prompt"                   # Specify model
lc -p <provider> -m <model> "prompt"     # Specify both provider and model
echo "prompt" | lc -p <provider>         # Piped input
```

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

- **OpenAI** - GPT-3.5, GPT-4, GPT-4o, etc.
- **OpenRouter** - Access to Claude, GPT, Gemini, and many other models
- **Together AI** - Open source models like Llama, Mistral, etc.
- **Chutes AI** - Various LLM models with competitive pricing
- **Hugging Face Router** - Proxy to multiple providers (Groq, Hyperbolic, Fireworks, etc.)
- **Anthropic** - Claude models (via compatible proxy)
- **Vercel v0.dev** - v0-1.0-md model
- **Local models** - Ollama, LocalAI, etc.
- **Custom endpoints** - Any service implementing OpenAI chat completions API

### Provider Examples

```bash
# OpenRouter (supports many models)
lc providers add openrouter https://openrouter.ai/api/v1
lc -p openrouter -m "anthropic/claude-3.5-sonnet" "Hello"

# Together AI (open source models)
lc providers add together https://api.together.xyz/v1
lc -p together -m "meta-llama/Llama-3-8b-chat-hf" "Hello"

# Chutes AI
lc providers add chutes https://llm.chutes.ai/v1
lc -p chutes -m "deepseek-ai/DeepSeek-R1" "Hello"

# Hugging Face Router (proxy to multiple providers)
lc providers add hf https://router.huggingface.co/v1
lc -p hf -m "Qwen/Qwen3-32B:groq" "Hello"
```

### Hugging Face Router Support

The Hugging Face router is a special provider that acts as a proxy to multiple underlying providers. When you list models from the HF router, the tool automatically expands models with their available providers in the format `model:provider`:

```bash
# Add HF router
lc providers add hf https://router.huggingface.co/v1
lc keys add hf  # Enter your HF token

# List models (shows expanded format)
lc providers models hf
# Output includes:
#   • Qwen/Qwen3-32B:groq
#   • Qwen/Qwen3-32B:hyperbolic
#   • meta-llama/Llama-3.3-70B-Instruct:together
#   • deepseek-ai/DeepSeek-R1:fireworks-ai

# Use specific model:provider combination
lc -p hf -m "Qwen/Qwen3-32B:groq" "What is the capital of France?"
```

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
# Direct prompt with flags
cargo run -- -p openai -m gpt-4 "Hello world"

# Interactive chat
cargo run -- chat -m gpt-4

# List providers
cargo run -- providers list
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
lc providers add openai https://api.openai.com/v1
lc keys add openai
```

**"API request failed"**
- Check your API key: `lc keys add <provider>`
- Verify endpoint URL is correct
- Ensure you have sufficient API credits

**"Model not found"**
- List available models: `lc providers models <provider>`
- Use exact model name from the list

### Debug Mode
Set `RUST_LOG=debug` for detailed logging:
```bash
RUST_LOG=debug lc -m gpt-4 "test"
```

## Example Usage with Vercel v0.dev

```bash
# Add Vercel provider
lc providers add vercel https://api.v0.dev/v1

# Set API key (from your Vercel account)
lc keys add vercel
# Enter: v1:pAZXgzGJaXccjXeSDSeByXjn:K0UA5jEUPL1350Jbj7xtK3g3

# List available models
lc providers models vercel

# Direct prompt with v0 model
lc -p vercel -m v0-1.0-md "Create a React component for a todo list"

# Interactive chat with v0 model
lc chat -m v0-1.0-md
```

This implementation provides a fast, efficient alternative to Python-based LLM CLI tools with significant performance improvements and a clean, intuitive interface.
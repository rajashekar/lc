---
id: faq
title: FAQ
sidebar_position: 101
---

# Frequently Asked Questions

## General

### What is LLM Client (lc)?

LLM Client is a fast, Rust-based command-line tool for interacting with Large Language Models through OpenAI-compatible APIs. It's designed to be significantly faster than Python alternatives while providing advanced features like vector databases and RAG support.

### Why is it called "lc"?

"lc" stands for "LLM Client" - a short, easy-to-type command that follows Unix naming conventions.

### How is lc different from other LLM tools?

- **Speed**: ~3ms cold start vs ~150ms for Python tools
- **Memory**: Uses 6x less memory
- **Features**: Built-in vector database and RAG
- **Compatibility**: Works with any OpenAI-compatible API
- **Simplicity**: Intuitive commands with short aliases

### Is lc free to use?

Yes, lc itself is free and open source (MIT license). However, you'll need API keys for the LLM providers you want to use, which may have their own costs.

## Installation

### What are the system requirements?

- **OS**: Linux, macOS, or Windows
- **Rust**: 1.70 or higher (for building from source)
- **Disk**: ~10MB for the binary
- **RAM**: Minimal (~6MB runtime)

### Do I need to install Rust?

Only if you're building from source. Pre-built binaries will be available in future releases.

### Can I install lc without admin privileges?

Yes, you can install lc in your user directory:

```bash
cp target/release/lc ~/.local/bin/
```

## Providers

### What providers are supported?

Any OpenAI-compatible API, including:

- OpenAI (GPT-4, GPT-3.5)
- Anthropic Claude
- OpenRouter
- Together AI
- GitHub Models
- Vercel v0
- Local models (Ollama)
- Custom endpoints

### Can I use local models?

Yes! You can use Ollama or any local OpenAI-compatible server:

```bash
lc providers add ollama http://localhost:11434/v1
```

### How do I use multiple providers?

Add as many providers as you want:

```bash
lc providers add openai https://api.openai.com/v1
lc providers add claude https://api.anthropic.com/v1
lc providers add together https://api.together.xyz/v1

# Use specific provider
lc -p openai "prompt"
lc -p claude "prompt"
```

### What if my provider needs special headers?

Use the headers command:

```bash
lc providers headers <provider> add <header> <value>
```

## API Keys

### Where are API keys stored?

API keys are stored in your platform's config directory:

- Linux: `~/.config/lc/config.toml`
- macOS: `~/Library/Application Support/lc/config.toml`
- Windows: `%APPDATA%\lc\config.toml`

### Are API keys encrypted?

Keys are stored in plain text in the config file, but the file has restricted permissions (user-only access). For additional security, you can use the sync feature with encryption.

### Can I use environment variables for API keys?

Currently, API keys must be set through the `lc keys add` command. Environment variable support may be added in future versions.

## Usage

### How do I set default provider and model?

```bash
lc config set provider openai
lc config set model gpt-4
```

### Can I pipe input to lc?

Yes:

```bash
echo "Translate to Spanish: Hello" | lc
cat file.txt | lc "Summarize this"
```

### How do I extract code from responses?

```bash
lc logs recent answer code
# or
lc l r a c
```

### Can I continue previous conversations?

Yes, use the chat command with a session ID:

```bash
# Get recent session ID
lc logs recent session

# Continue that session
lc chat -m gpt-4 --cid <session-id>
```

## Vector Databases

### What are vector databases used for?

Vector databases store text embeddings for:

- Semantic search
- RAG (Retrieval-Augmented Generation)
- Building knowledge bases
- Context-aware responses

### How much space do vector databases use?

It depends on content volume:

- Small (100 docs): ~10MB
- Medium (1,000 docs): ~100MB
- Large (10,000 docs): ~1GB

### Can I share vector databases?

Yes, vector databases are SQLite files that can be copied and shared. They're stored in:

- Linux: `~/.config/lc/embeddings/`
- macOS: `~/Library/Application Support/lc/embeddings/`
- Windows: `%APPDATA%\lc\embeddings\`

### What file types can be embedded?

Text files including:

- Documents: `.txt`, `.md`, `.pdf`, `.docx`
- Code: `.py`, `.js`, `.rs`, `.java`, etc.
- Config: `.json`, `.yaml`, `.toml`
- Web: `.html`, `.css`, `.xml`

## Performance

### Why is lc so fast?

- Written in Rust (compiled language)
- Minimal dependencies
- Efficient SQLite usage
- No runtime interpreter needed

### How can I make responses faster?

1. Use faster models (e.g., gpt-3.5-turbo)
2. Use providers with lower latency
3. Keep prompts concise
4. Use local models for offline work

### Does lc cache responses?

All responses are logged to SQLite, but not cached for reuse. Each prompt generates a new API request.

## Troubleshooting

### How do I enable debug mode?

```bash
export RUST_LOG=debug
lc "test prompt"
```

### Where are logs stored?

In the SQLite database at:

- Linux: `~/.config/lc/logs.db`
- macOS: `~/Library/Application Support/lc/logs.db`
- Windows: `%APPDATA%\lc\logs.db`

### How do I reset everything?

Remove the config directory:

- Linux: `rm -rf ~/.config/lc`
- macOS: `rm -rf ~/Library/Application Support/lc`
- Windows: `rmdir /s %APPDATA%\lc`

## Advanced Features

### What is RAG?

RAG (Retrieval-Augmented Generation) enhances LLM responses by automatically including relevant context from your vector databases.

### Can I sync settings across machines?

Yes, use the sync feature:

```bash
# Setup
lc sync configure s3 setup

# Sync to cloud
lc sync to s3 --encrypted

# Sync from cloud (on another machine)
lc sync from s3 --encrypted
```

### What are MCP servers?

MCP (Model Context Protocol) servers extend lc's functionality with additional tools and resources. Support is currently in development.

## Contributing

### How can I contribute?

1. Report bugs via GitHub Issues
2. Submit pull requests
3. Improve documentation
4. Share your use cases

### Where can I get help?

- Documentation: https://lc.viwq.dev
- GitHub Issues: https://github.com/rajashekar/lc/issues
- Community discussions: [Coming soon]

### Is there a roadmap?

Check the GitHub repository for:

- Current issues
- Planned features
- Release notes

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
- 🔄 **Multiple API formats** - Handles various response formats (OpenAI, Cohere, Llama, etc.)
- 🏷️ **Custom headers** - Per-provider custom header support for specialized APIs (e.g., Anthropic)
- 🧠 **Vector Database & RAG** - Built-in vector storage with similarity search and retrieval-augmented generation
- 📚 **Embedding Support** - Generate and store text embeddings from any compatible provider
- 🔍 **Semantic Search** - Find similar content using cosine similarity across your knowledge base
- 🤖 **RAG-Enhanced Chat** - Augment conversations with relevant context from vector databases
- ☁️ **Configuration Sync** - Sync your configuration files to/from cloud providers with optional AES256 encryption
- 🔐 **Secure Cloud Storage** - Store configurations in S3-compatible services with strong encryption
- 🔄 **Cross-Platform Sync** - Keep your settings synchronized across multiple machines

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
# Add OpenAI (standard endpoints)
lc providers add openai https://api.openai.com/v1
# or using alias
lc p a openai https://api.openai.com/v1

# Add GitHub with custom endpoints
lc providers add github https://models.github.ai -m /catalog/models -c /inference/chat/completions
# or using alias
lc p a github https://models.github.ai -m /catalog/models -c /inference/chat/completions

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

## Vector Database & RAG (Retrieval-Augmented Generation)

LLM Client includes a powerful built-in vector database system that enables you to store text embeddings and perform semantic search. This allows for Retrieval-Augmented Generation (RAG), where your conversations can be enhanced with relevant context from your knowledge base.

### Quick Start with Vector Database

```bash
# 1. List available embedding models
lc models embed
# or using alias
lc m e

# 2. Generate and store embeddings in a vector database
lc embed -m text-embedding-3-small -v knowledge "Machine learning is a subset of AI"
lc embed -m text-embedding-3-small -v knowledge "Deep learning uses neural networks"
lc embed -m text-embedding-3-small -v knowledge "Python is popular for data science"

# 3. Search for similar content
lc similar -v knowledge "What is neural network programming?"

# 4. Use RAG-enhanced chat (context from vector database)
lc chat -v knowledge -m gpt-4
lc -v knowledge "Explain the relationship between AI and programming languages"
```

### Vector Database Features

- **🗄️ SQLite Storage** - Efficient local storage in `~/Library/Application Support/lc/embeddings/`
- **📐 Cosine Similarity** - Fast similarity search with proper dimension validation
- **🔄 Model Consistency** - Automatic model/provider resolution from database metadata
- **🎯 Context Filtering** - Smart relevance filtering (similarity > 0.3) for RAG
- **⚡ Performance** - Optimized for fast retrieval and context generation
- **🔧 Database Management** - Full CRUD operations for vector databases

### Embedding Commands

```bash
# List embedding models with metadata
lc models embed
lc m e

# Generate embeddings (output to console)
lc embed -m text-embedding-3-small "Your text here"

# Store embeddings in vector database
lc embed -m text-embedding-3-small -v database_name "Your text here"

# Use different providers for embeddings
lc embed -p openai -m text-embedding-3-large -v docs "Technical documentation"
lc embed -p cohere -m embed-english-v3.0 -v knowledge "Knowledge base entry"

# Embed files with intelligent chunking
lc embed -m text-embedding-3-small -v docs -f README.md
lc embed -m text-embedding-3-small -v docs -f "*.md"
lc embed -m text-embedding-3-small -v docs -f "/path/to/docs/*.txt"
```

### File Embedding

The system supports embedding files directly with intelligent text chunking and metadata storage:

```bash
# Embed a single file
lc embed -m text-embedding-3-small -v knowledge -f document.txt

# Embed multiple files using glob patterns
lc embed -m text-embedding-3-small -v knowledge -f "*.md"
lc embed -m text-embedding-3-small -v knowledge -f "docs/*.txt"
lc embed -m text-embedding-3-small -v knowledge -f "/path/to/files/*.py"

# Embed multiple specific files
lc embed -m text-embedding-3-small -v knowledge -f file1.txt,file2.md,file3.py

# Combine with provider specification
lc embed -p openai -m text-embedding-3-large -v docs -f "documentation/*.md"
```

#### File Processing Features

- **🧠 Intelligent Chunking** - Automatically splits large files into 1200-character chunks with 200-character overlap
- **📄 Smart Boundaries** - Breaks at sentence, paragraph, or line boundaries when possible
- **🔍 Binary Detection** - Automatically filters out binary files, processing only text content
- **📁 Glob Pattern Support** - Full glob syntax support for flexible file selection
- **📊 File Metadata** - Stores file path, chunk index, and total chunks for each embedding
- **🎯 Progress Tracking** - Shows processing progress for each file and chunk

#### Supported File Types

The system automatically detects and processes text files including:

**Text Files:** `.txt`, `.md`, `.markdown`, `.rst`, `.org`, `.tex`, `.rtf`

**Code Files:** `.rs`, `.py`, `.js`, `.ts`, `.java`, `.cpp`, `.c`, `.h`, `.hpp`, `.go`, `.rb`, `.php`, `.swift`, `.kt`, `.scala`, `.sh`, `.bash`, `.zsh`, `.fish`, `.ps1`, `.bat`, `.cmd`

**Web Files:** `.html`, `.css`, `.scss`, `.sass`, `.less`, `.xml`, `.json`, `.yaml`, `.yml`

**Config Files:** `.toml`, `.ini`, `.cfg`, `.conf`, `.sql`, `.dockerfile`, `.makefile`, `.cmake`, `.gradle`

**Other:** `.log`, `.out`, `.err`, `.r`, `.m`, `.mm`, `.pl`, `.pm`, `.lua`, `.vim`

Binary files (images, executables, archives, etc.) are automatically filtered out.

#### File Embedding Examples

```bash
# Embed documentation files
lc embed -m text-embedding-3-small -v docs -f "docs/*.md"

# Embed source code for a project
lc embed -m text-embedding-3-small -v codebase -f "src/**/*.rs"

# Embed configuration files
lc embed -m text-embedding-3-small -v configs -f "*.toml,*.yaml,*.json"

# Embed all text files in a directory
lc embed -m text-embedding-3-small -v knowledge -f "/path/to/data/*"
```

#### Viewing File Metadata

When you view vector database information, file metadata is displayed:

```bash
lc vectors info docs
# Output shows:
# 📝 Recent entries:
#   1. This is the content of the file... [README.md:1/3] (2025-01-28 01:21)
#   2. More content from the same file... [README.md:2/3] (2025-01-28 01:21)
#   3. Final chunk of the file...        [README.md:3/3] (2025-01-28 01:21)
```

The format `[filename:chunk/total]` shows:
- **filename** - Original file path
- **chunk** - Current chunk number (1-based)
- **total** - Total number of chunks for this file
```

### Vector Database Management

```bash
# List all vector databases
lc vectors list
lc v l

# Show database information (count, model, provider)
lc vectors info database_name
lc v i database_name

# Delete a vector database
lc vectors delete database_name
lc v d database_name
```

### Similarity Search

```bash
# Search for similar content
lc similar -v database_name "your search query"
lc s -v database_name "your search query"

# Limit number of results (default: 5)
lc similar -v docs -l 10 "search query"

# The system automatically uses the same embedding model/provider as stored in the database
```

### RAG-Enhanced Conversations

#### Interactive Chat with Vector Context
```bash
# Start RAG-enhanced chat session
lc chat -v knowledge_base -m gpt-4
lc c -v knowledge_base -m gpt-4

# Inside chat, your messages are automatically enhanced with relevant context
# The system retrieves similar content and includes it in the prompt
```

#### Direct RAG Prompts
```bash
# Single prompt with vector context
lc -v knowledge_base "Explain machine learning concepts"

# Specify both vector database and chat model
lc -v docs -m claude-3-5-sonnet "How do I implement this feature?"

# The system automatically:
# 1. Generates embeddings for your query
# 2. Finds similar content in the vector database
# 3. Includes relevant context in the prompt to the LLM
# 4. Returns an enhanced response
```

### RAG Workflow Example

```bash
# 1. Create a knowledge base about AI/ML
lc embed -m text-embedding-3-small -v ai_knowledge "Machine learning is a method of data analysis that automates analytical model building"
lc embed -m text-embedding-3-small -v ai_knowledge "Deep learning is part of a broader family of machine learning methods based on neural networks"
lc embed -m text-embedding-3-small -v ai_knowledge "Python is widely used in machine learning due to libraries like scikit-learn, TensorFlow, and PyTorch"
lc embed -m text-embedding-3-small -v ai_knowledge "Neural networks are computing systems inspired by biological neural networks"

# 2. Check what's in your database
lc vectors info ai_knowledge
lc similar -v ai_knowledge "neural networks"

# 3. Use RAG for enhanced conversations
lc -v ai_knowledge "What programming language should I use for machine learning and why?"

# The response will be enhanced with relevant context from your knowledge base
```

### Advanced RAG Features

#### Model Consistency
The system ensures embedding consistency by:
- Storing the embedding model/provider with each vector database
- Automatically using the same model for similarity search
- Validating vector dimensions match the stored model

#### Context Filtering
RAG implementation includes smart filtering:
- Only includes content with similarity > 0.3 threshold
- Formats context with bullet points for clarity
- Limits context length to prevent token overflow

#### Provider Separation
You can use different providers for embeddings vs chat:
```bash
# Use OpenAI for embeddings, Claude for chat
lc embed -p openai -m text-embedding-3-small -v docs "content"
lc -v docs -p claude -m claude-3-5-sonnet "query"
```

### Vector Database Storage

Vector databases are stored in platform-appropriate locations:

| Platform | Vector Database Directory |
|----------|---------------------------|
| **Linux** | `~/.config/lc/embeddings/` |
| **macOS** | `~/Library/Application Support/lc/embeddings/` |
| **Windows** | `%APPDATA%\lc\embeddings\` |

Each database is a SQLite file containing:
- **vectors** table - Text content, embeddings, and metadata
- **model_info** table - Embedding model and provider information
- **Indexes** - Optimized for fast similarity search

## Configuration Sync

LLM Client includes a powerful sync feature that allows you to synchronize your configuration files to and from cloud providers with optional AES256 encryption. This enables you to keep your settings synchronized across multiple machines securely.

### Quick Start with Sync

```bash
# 1. List supported cloud providers
lc sync providers
# or using alias
lc sy p

# 2. Configure S3 provider (interactive setup)
lc sync configure s3 setup
# or using aliases
lc sy c s3 s

# 3. Sync your configuration to cloud with encryption
lc sync to s3 --encrypted
# or using aliases
lc sy to s3 -e

# 4. Later, sync from cloud on another machine
lc sync from s3 --encrypted
# or using aliases
lc sy from s3 -e
```

### Sync Features

- **🌩️ Cloud Provider Support** - Currently supports Amazon S3 and S3-compatible services (Backblaze B2, Cloudflare R2, MinIO, etc.)
- **🔐 AES256-GCM Encryption** - Optional strong encryption for secure cloud storage
- **⚙️ Configuration Management** - Store provider credentials locally for easy reuse
- **🔄 Cross-Platform** - Works seamlessly on macOS, Linux, and Windows
- **📁 Automatic Discovery** - Syncs all `.toml` configuration files automatically
- **🏷️ Multiple Aliases** - Short commands (`sy`, `c`, `p`) for faster usage

### Sync Commands

```bash
# Provider management
lc sync providers                        # List supported providers (alias: lc sy p)
lc sync configure s3 setup               # Set up S3 configuration (alias: lc sy c s3 s)
lc sync configure s3 show                # Show current configuration (alias: lc sy c s3 sh)
lc sync configure s3 remove              # Remove configuration (alias: lc sy c s3 r)

# Sync operations
lc sync to s3                            # Sync to cloud (alias: lc sy to s3)
lc sync to s3 --encrypted                # Sync with encryption (alias: lc sy to s3 -e)
lc sync from s3                          # Sync from cloud (alias: lc sy from s3)
lc sync from s3 --encrypted              # Sync with decryption (alias: lc sy from s3 -e)
```

### What Gets Synced

The sync feature automatically discovers and syncs all `.toml` configuration files in your lc configuration directory:

- `config.toml` - Main lc configuration (providers, API keys, defaults)
- `mcp.toml` - MCP server configurations
- `sync.toml` - Sync provider configurations
- Any other `.toml` files in the config directory

### Supported Cloud Providers

#### Amazon S3
Full support with three configuration methods (in priority order):

1. **Stored Configuration** (Recommended):
   ```bash
   lc sync configure s3 setup
   ```

2. **Environment Variables**:
   ```bash
   export LC_S3_BUCKET=your-bucket-name
   export LC_S3_REGION=us-east-1
   export AWS_ACCESS_KEY_ID=your-access-key
   export AWS_SECRET_ACCESS_KEY=your-secret-key
   export LC_S3_ENDPOINT=https://s3.amazonaws.com  # Optional
   ```

3. **Interactive Prompts** (Fallback when no config/env vars found)

#### S3-Compatible Services
Supports any S3-compatible service by setting a custom endpoint:

- **Backblaze B2**: `https://s3.us-west-004.backblazeb2.com`
- **Cloudflare R2**: `https://your-account-id.r2.cloudflarestorage.com`
- **MinIO**: `https://your-minio-server.com:9000`
- **DigitalOcean Spaces**: `https://your-region.digitaloceanspaces.com`
- **Wasabi**: `https://s3.your-region.wasabisys.com`

### Encryption

The sync feature uses **AES256-GCM encryption** for secure storage:

- **Algorithm**: AES256-GCM (Galois/Counter Mode) with authentication
- **Key Derivation**: PBKDF2 with SHA-256 and random salt
- **Security**: Each file gets unique salt and nonce for maximum security
- **Password**: You'll be prompted for a password during encryption/decryption
- **File Names**: Encrypted files get `.enc` extension in cloud storage

### Sync Configuration Storage

Sync configurations are stored in platform-appropriate locations:

| Platform | Sync Config File |
|----------|------------------|
| **Linux** | `~/.config/lc/sync.toml` |
| **macOS** | `~/Library/Application Support/lc/sync.toml` |
| **Windows** | `%APPDATA%\lc\sync.toml` |

### Complete Sync Workflow Example

```bash
# 1. Set up S3 configuration
lc sync configure s3 setup
# Enter: bucket name, region, access key, secret key, endpoint (optional)

# 2. Verify configuration
lc sync configure s3 show

# 3. Sync to cloud with encryption
lc sync to s3 --encrypted
# Enter encryption password when prompted

# 4. On another machine, sync from cloud
lc sync from s3 --encrypted
# Enter the same decryption password

# 5. Your configurations are now synchronized!
```

For detailed documentation, see [docs/SYNC_FEATURE.md](docs/SYNC_FEATURE.md).

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
| **Linux** | `~/.config/lc/` | `config.toml`, `logs.db`, `embeddings/` |
| **macOS** | `~/Library/Application Support/lc/` | `config.toml`, `logs.db`, `embeddings/` |
| **Windows** | `%APPDATA%\lc\` | `config.toml`, `logs.db`, `embeddings/` |

### Files Stored:
- **`config.toml`** - Provider configurations and API keys
- **`logs.db`** - SQLite database with complete chat history and session state
- **`embeddings/`** - Directory containing vector databases (SQLite files with embeddings)

### Automatic Directory Creation
The tool automatically creates the necessary directories on first run, ensuring proper permissions and OS integration.

### Example config.toml:
```toml
default_provider = "openai"

[providers.openai]
endpoint = "https://api.openai.com/v1"
api_key = "sk-..."
models = []
models_path = "/models"
chat_path = "/chat/completions"

[providers.claude]
endpoint = "https://api.anthropic.com/v1"
api_key = "sk-ant-api03-..."
models = []
models_path = "/models"
chat_path = "/messages"

[providers.claude.headers]
x-api-key = "sk-ant-api03-..."
anthropic-version = "2023-06-01"

[providers.vercel]
endpoint = "https://api.v0.dev/v1"
api_key = "v1:..."
models = []
models_path = "/models"
chat_path = "/chat/completions"
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
lc providers add <name> <url>                              # Add provider (alias: lc p a)
lc providers add <name> <url> -m <path> -c <path>          # Add provider with custom endpoints
lc providers update <name> <url>                           # Update provider (alias: lc p u)
lc providers remove <name>                                 # Remove provider (alias: lc p r)
lc providers list                                          # List providers (alias: lc p l)
lc providers models <name>                                 # List models (alias: lc p m)
lc providers headers <name> add <header> <value>           # Add custom header (alias: lc p h <name> a)
lc providers headers <name> delete <header>                # Remove custom header (alias: lc p h <name> d)
lc providers headers <name> list                           # List custom headers (alias: lc p h <name> l)
```

### Models Commands
```bash
lc models                                                  # List all models with metadata (alias: lc m)
lc models -q <query>                                       # Search models by name (alias: lc m -q)
lc models --tools                                          # Filter models with function calling support
lc models --reasoning                                      # Filter models with reasoning capabilities
lc models --vision                                         # Filter models with vision/image support
lc models --audio                                          # Filter models with audio support
lc models --code                                           # Filter models optimized for code generation
lc models --ctx <length>                                   # Filter by minimum context length (e.g., 128k, 200k)
lc models --input <length>                                 # Filter by minimum input token length
lc models --output <length>                                # Filter by minimum output token length
lc models --input-price <price>                            # Filter by maximum input price per million tokens
lc models --output-price <price>                           # Filter by maximum output price per million tokens
lc models refresh                                          # Refresh models cache (alias: lc m r)
lc models info                                             # Show cache information (alias: lc m i)
lc models dump                                             # Dump raw provider responses (alias: lc m d)
lc models embed                                            # List embedding models (alias: lc m e)
```

### Embedding Commands
```bash
lc embed -m <model> "text"                                 # Generate embeddings (alias: lc e)
lc embed -m <model> -v <database> "text"                   # Store embeddings in vector database
lc embed -p <provider> -m <model> -v <database> "text"     # Specify provider for embeddings
lc embed -m <model> -v <database> -f <file>                # Embed single file with chunking
lc embed -m <model> -v <database> -f "*.ext"               # Embed files using glob patterns
lc embed -m <model> -v <database> -f "file1,file2"         # Embed multiple specific files
```

### Vector Database Commands
```bash
lc vectors list                                            # List all vector databases (alias: lc v l)
lc vectors info <database>                                 # Show database information (alias: lc v i)
lc vectors delete <database>                               # Delete vector database (alias: lc v d)
```

### Similarity Search Commands
```bash
lc similar -v <database> "query"                           # Search for similar content (alias: lc s)
lc similar -v <database> -l <limit> "query"                # Limit number of results
```

### RAG-Enhanced Commands
```bash
lc -v <database> "prompt"                                  # Direct prompt with vector context
lc chat -v <database> -m <model>                           # Interactive chat with vector context
lc c -v <database> -m <model>                              # Alias for RAG-enhanced chat
```

**Model Filtering Examples:**
```bash
# Find all models with function calling support
lc models --tools

# Find vision models with at least 128k context
lc models --vision --ctx 128k

# Find reasoning models from Claude
lc models -q claude --reasoning

# Find code generation models with tools support
lc models --code --tools

# Combine multiple filters
lc models --tools --vision --ctx 200k
```

**Model Metadata Display:**
The models listing shows rich metadata with capability indicators:
- 🔧 **tools** - Function calling/tool use support
- 👁 **vision** - Image processing capabilities
- 🧠 **reasoning** - Advanced reasoning capabilities
- 💻 **code** - Optimized for code generation
- 🔊 **audio** - Audio processing support
- **(200k ctx)** - Context length information
- **(Model Display Name)** - Human-readable model names

**Custom Endpoint Options:**
- `-m, --models-path <PATH>`: Custom models endpoint (default: `/models`)
- `-c, --chat-path <PATH>`: Custom chat completions endpoint (default: `/chat/completions`)

**Custom Headers:**
Some providers require additional headers beyond the standard Authorization header. You can add custom headers per provider:
- `x-api-key`: Alternative API key header (used by Anthropic/Claude)
- `anthropic-version`: API version header (required by Anthropic)
- Any other custom headers required by your provider

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

### Sync Commands
```bash
lc sync providers                        # List supported cloud providers (alias: lc sy p)
lc sync configure <provider> setup       # Set up provider configuration (alias: lc sy c <provider> s)
lc sync configure <provider> show        # Show current configuration (alias: lc sy c <provider> sh)
lc sync configure <provider> remove      # Remove provider configuration (alias: lc sy c <provider> r)
lc sync to <provider>                    # Sync configuration to cloud (alias: lc sy to <provider>)
lc sync to <provider> --encrypted        # Sync with encryption (alias: lc sy to <provider> -e)
lc sync from <provider>                  # Sync configuration from cloud (alias: lc sy from <provider>)
lc sync from <provider> --encrypted      # Sync with decryption (alias: lc sy from <provider> -e)
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
- **GitHub Models** - Microsoft, OpenAI, Meta, and other models via GitHub
- **Anthropic Claude** - Direct support with custom headers (x-api-key, anthropic-version)
- **Meta Llama** - Direct support for Llama API response format
- **Cohere** - Direct support for Cohere API response format
- **Vercel v0.dev** - v0-1.0-md model
- **Local models** - Ollama, LocalAI, etc.
- **Custom endpoints** - Any service implementing OpenAI chat completions API

### Supported Response Formats

The tool automatically detects and handles multiple API response formats:

1. **OpenAI Standard Format** (most providers):
   ```json
   {"choices": [{"message": {"role": "assistant", "content": "response"}}]}
   ```

2. **Llama API Format**:
   ```json
   {"completion_message": {"role": "assistant", "content": {"type": "text", "text": "response"}}}
   ```

3. **Cohere Format**:
   ```json
   {"message": {"role": "assistant", "content": [{"type": "text", "text": "response"}]}}
   ```

The client tries each format in sequence and uses the first one that successfully parses, ensuring compatibility with various API providers without requiring manual configuration.

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

# GitHub Models (custom endpoints)
lc providers add github https://models.github.ai -m /catalog/models -c /inference/chat/completions
lc -p github -m "microsoft/phi-4-mini-instruct" "Hello"

# Anthropic Claude (requires custom headers)
lc providers add claude https://api.anthropic.com/v1 -c /messages
lc providers headers claude add x-api-key sk-ant-api03-your-key-here
lc providers headers claude add anthropic-version 2023-06-01
lc -p claude -m "claude-3-5-sonnet-20241022" "Hello"
```

### Providers Requiring Custom Headers

Some providers require additional headers beyond the standard `Authorization: Bearer <token>` header. Here are examples:

#### Anthropic Claude
```bash
# Add Claude provider with custom chat endpoint
lc providers add claude https://api.anthropic.com/v1 -c /messages

# Add required headers
lc providers headers claude add x-api-key sk-ant-api03-your-key-here
lc providers headers claude add anthropic-version 2023-06-01

# List available models
lc providers models claude

# Use Claude
lc -p claude -m "claude-3-5-sonnet-20241022" "Explain quantum computing"
```

#### Custom Provider with Special Headers
```bash
# Add provider
lc providers add custom https://api.example.com/v1

# Add custom headers as needed
lc providers headers custom add x-custom-auth your-auth-token
lc providers headers custom add x-api-version v2.1
lc providers headers custom add user-agent MyApp/1.0

# Use the provider
lc -p custom -m "some-model" "Hello world"
```

#### Managing Headers
```bash
# List all headers for a provider
lc providers headers claude list
lc p h claude l

# Add a header
lc providers headers claude add header-name header-value
lc p h claude a header-name header-value

# Remove a header
lc providers headers claude delete header-name
lc p h claude d header-name
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

### Custom Endpoints Support

Some providers use different endpoint paths for models and chat completions. The tool supports custom endpoint paths for such providers:

```bash
# Standard provider (uses /models and /chat/completions)
lc providers add openai https://api.openai.com/v1

# Provider with custom endpoints
lc providers add github https://models.github.ai \
  --models-path /catalog/models \
  --chat-path /inference/chat/completions

# Short form with aliases
lc p a github https://models.github.ai -m /catalog/models -c /inference/chat/completions
```

**Default Endpoints:**
- Models: `/models` (can be overridden with `-m` or `--models-path`)
- Chat: `/chat/completions` (can be overridden with `-c` or `--chat-path`)

**Examples of providers with custom endpoints:**
- **GitHub Models**: Uses `/catalog/models` for models and `/inference/chat/completions` for chat
- **Custom APIs**: May use different paths based on their implementation

This feature ensures compatibility with any OpenAI-compatible API regardless of their endpoint structure.

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
│   ├── vector_db.rs     # Vector database with embeddings and similarity search
│   ├── chat.rs          # Chat request handling
│   └── error.rs         # Error types and handling
├── tests/
│   ├── embed_commands.rs    # Embedding functionality tests
│   ├── vector_commands.rs   # Vector database operation tests
│   ├── similar_commands.rs  # Similarity search tests
│   └── rag_commands.rs      # RAG functionality tests
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

**"Vector database not found"**
- List available databases: `lc vectors list`
- Create database by storing embeddings: `lc embed -m model -v database "text"`

**"Dimension mismatch in similarity search"**
- Ensure you're using the same embedding model as stored in the database
- Check database model info: `lc vectors info <database>`
- The system automatically uses the correct model from database metadata

**"No similar content found"**
- Check if database has content: `lc vectors info <database>`
- Try different search terms or lower similarity threshold
- Add more content to your vector database

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
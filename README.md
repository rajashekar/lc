[![Documentation](https://img.shields.io/badge/docs-lc.viwq.dev-blue)](https://lc.viwq.dev)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org)

<p align="center">
<h1>LLM Client (lc)</h1>
<img src="docs-site/static/img/social-card.png" alt="LLM Client" width="450" />
</p>


A fast, Rust-based command-line tool for interacting with Large Language Models through OpenAI-compatible APIs. This tool is inspired by Simon Willison's [llm](https://github.com/simonw/llm).

## Quick Start

```bash
# Install from source
git clone <repository-url>
cd lc
cargo build --release

# Add a provider
lc providers add openai https://api.openai.com/v1

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

## Key Features

- 🚀 **Lightning Fast** - ~3ms cold start (50x faster than Python alternatives)
- 🔧 **Universal** - Works with any OpenAI-compatible API
- 🧠 **Smart** - Built-in vector database and RAG support
- 🛠️ **Tools** - Model Context Protocol (MCP) support for extending LLM capabilities
- 🔍 **Web Search** - Integrated web search with multiple providers (Brave, Exa, Serper) for enhanced context
- 📄 **PDF Support** - Read and process PDF files with optional dependency
- 🔐 **Secure** - Encrypted configuration sync
- 💬 **Intuitive** - Simple commands with short aliases

### Important: Capability Display Policy

**⚠️ Note**: Starting with recent versions, `lc` uses a stricter policy for displaying model capability icons (🔧 tools, 👁 vision, 🧠 reasoning, etc.). Icons are now only shown when capabilities are explicitly provided by the provider's API response. Previously, some icons were displayed based on assumptions derived from model names or patterns.

### Troubleshooting

If you notice that responses are slower than expected or formatted differently, please check your network proxy settings. Some proxies may re-compress responses, causing delays or unexpected behavior. Disable such features if streaming output is required in real-time.

**Impact**: You may notice that some capability icons you previously saw have disappeared. This doesn't mean the models have lost these capabilities - it simply means the provider's API doesn't explicitly advertise them. The actual model capabilities remain unchanged.

## Documentation

For comprehensive documentation, visit **[lc.viwq.dev](https://lc.viwq.dev)**

### Quick Links

- [Installation Guide](https://lc.viwq.dev/getting-started/installation)
- [Quick Start Tutorial](https://lc.viwq.dev/getting-started/quick-start)
- [Command Reference](https://lc.viwq.dev/commands/overview)
- [Provider Setup](https://lc.viwq.dev/features/providers)
- [Vector Database & RAG](https://lc.viwq.dev/advanced/vector-database)
- [Model Context Protocol (MCP)](https://lc.viwq.dev/advanced/mcp)

## Supported Providers
Any OpenAI-compatible API can be used with `lc`. Here are some popular providers:
Anthropic and Gemini also supported.
  - ai21 - https://api.ai21.com/studio/v1 (API Key: ✓)
  - cerebras - https://api.cerebras.ai/v1 (API Key: ✓)
  - chub - https://inference.chub.ai/v1 (API Key: ✓)
  - chutes - https://llm.chutes.ai/v1 (API Key: ✓)
  - claude - https://api.anthropic.com/v1 (API Key: ✓)
  - cohere - https://api.cohere.com/v2 (API Key: ✓)
  - deepinfra - https://api.deepinfra.com/v1/openai (API Key: ✓)
  - digitalocean - https://inference.do-ai.run/v1 (API Key: ✓)
  - fireworks - https://api.fireworks.ai/inference/v1 (API Key: ✓)
  - gemini - https://generativelanguage.googleapis.com (API Key: ✓)
  - github - https://models.github.ai (API Key: ✓)
  - github-copilot - https://api.individual.githubcopilot.com (API Key: ✓)
  - grok - https://api.x.ai/v1 (API Key: ✓)
  - groq - https://api.groq.com/openai/v1 (API Key: ✓)
  - hf - https://router.huggingface.co/v1 (API Key: ✓)
  - hyperbolic - https://api.hyperbolic.xyz/v1 (API Key: ✓)
  - kilo - https://kilocode.ai/api/openrouter (API Key: ✓)
  - meta - https://api.llama.com/v1 (API Key: ✓)
  - mistral - https://api.mistral.ai/v1 (API Key: ✓)
  - nebius - https://api.studio.nebius.com/v1 (API Key: ✓)
  - novita - https://api.novita.ai/v3/openai (API Key: ✓)
  - nscale - https://inference.api.nscale.com/v1 (API Key: ✓)
  - nvidia - https://integrate.api.nvidia.com/v1 (API Key: ✓)
  - ollama - http://localhost:11434/v1 (API Key: ✓)
  - openai - https://api.openai.com/v1 (API Key: ✓)
  - openrouter - https://openrouter.ai/api/v1 (API Key: ✓)
  - perplexity - https://api.perplexity.ai (API Key: ✓)
  - poe - https://api.poe.com/v1 (API Key: ✓)
  - requesty - https://router.requesty.ai/v1 (API Key: ✓)
  - sambanova - https://api.sambanova.ai/v1 (API Key: ✓)
  - together - https://api.together.xyz/v1 (API Key: ✓)
  - venice - https://api.venice.ai/api/v1 (API Key: ✓)
  - vercel - https://ai-gateway.vercel.sh/v1 (API Key: ✓)

## Example Usage

```bash
# Direct prompt with specific model
lc -m openai:gpt-4 "Explain quantum computing"

# Interactive chat session
lc chat -m anthropic:claude-3.5-sonnet

# Create embeddings
lc embed -m openai:text-embedding-3-small -v knowledge "Important information"

# Search similar content
lc similar -v knowledge "related query"

# RAG-enhanced chat
lc -v knowledge "What do you know about this topic?"

# Use MCP tools for internet access
lc -t fetch "What's the latest news about AI?"

# Multiple MCP tools
lc -t fetch,playwright "Navigate to example.com and analyze its content"

# Web search integration
lc --use-search brave "What are the latest developments in quantum computing?"

# Search with specific query
lc --use-search "brave:quantum computing 2024" "Summarize the findings"
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
```

**Note:** PDF support requires the `pdf` feature (enabled by default). To build without PDF support:

```bash
cargo build --release --no-default-features
```

To explicitly enable PDF support:

```bash
cargo build --release --features pdf
```

## Contributing

Contributions are welcome! Please see our [Contributing Guide](https://lc.viwq.dev/development/contributing).

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

For detailed documentation, examples, and guides, visit **[lc.viwq.dev](https://lc.viwq.dev)**
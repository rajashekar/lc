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
- 🔐 **Secure** - Encrypted configuration sync
- 💬 **Intuitive** - Simple commands with short aliases

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

## Contributing

Contributions are welcome! Please see our [Contributing Guide](https://lc.viwq.dev/development/contributing).

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

For detailed documentation, examples, and guides, visit **[lc.viwq.dev](https://lc.viwq.dev)**
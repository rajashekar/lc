[![Documentation](https://img.shields.io/badge/docs-lc.viwq.dev-blue)](https://lc.viwq.dev)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org)

<p align="center">
<h1>LLM Client (lc)</h1>
<img src="docs-site/static/img/social-card.png" alt="LLM Client" width="450" />
</p>


A fast, Rust-based command-line tool for interacting with Large Language Models. 

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
- 👁️ **Vision Support** - Process and analyze images with vision-capable models
- � **PDF Support** - Read and process PDF files with optional dependency
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
Anthropic, Gemini, and Amazon Bedrock also supported.
  - ai21 - https://api.ai21.com/studio/v1 (API Key: ✓)
  - bedrock - https://bedrock-runtime.us-east-1.amazonaws.com (API Key: ✓) - See [Bedrock Setup](#amazon-bedrock-setup)
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

### Amazon Bedrock Setup

Amazon Bedrock requires a special configuration due to its different endpoints for model listing and chat completions:

```bash
# Add Bedrock provider with different endpoints
lc providers add bedrock https://bedrock-runtime.us-east-1.amazonaws.com \
  -m /foundation-models \
  -c "https://bedrock-runtime.us-east-1.amazonaws.com/model/{model_name}/converse"

# Set your AWS Bearer Token
lc keys add bedrock

# List available models
lc providers models bedrock

# Use Bedrock models
lc -m bedrock:amazon.nova-pro-v1:0 "Hello, how are you?"

# Interactive chat with Bedrock
lc chat -m bedrock:amazon.nova-pro-v1:0
```

**Key differences for Bedrock:**
- **Models endpoint**: Uses `https://bedrock.us-east-1.amazonaws.com/foundation-models`
- **Chat endpoint**: Uses `https://bedrock-runtime.us-east-1.amazonaws.com/model/{model_name}/converse`
- **Authentication**: Requires AWS Bearer Token for Bedrock
- **Model names**: Use full Bedrock model identifiers (e.g., `amazon.nova-pro-v1:0`)

The `{model_name}` placeholder in the chat URL is automatically replaced with the actual model name when making requests.

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

## Contributing

Contributions are welcome! Please see our [Contributing Guide](https://lc.viwq.dev/development/contributing).

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

For detailed documentation, examples, and guides, visit **[lc.viwq.dev](https://lc.viwq.dev)**
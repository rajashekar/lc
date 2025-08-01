---
id: models
title: Models Command
sidebar_position: 3
---

# Models Command

List and filter available models across all providers with rich metadata.

## Command: `lc models` (alias: `lc m`)

### Basic Usage

List all available models with metadata:

```bash
lc models
lc m
```

Output shows models with capability indicators:
- 🔧 **tools** - Function calling support
- 👁 **vision** - Image processing
- 🧠 **reasoning** - Advanced reasoning
- 💻 **code** - Code generation
- 🔊 **audio** - Audio processing
- Context length (e.g., "200k ctx")
- Human-readable names

### Filtering Options

#### Search by Name
```bash
lc models -q <query>
lc m -q claude
```

#### Filter by Capabilities
```bash
# Models with function calling
lc models --tools

# Vision models
lc models --vision

# Reasoning models
lc models --reasoning

# Code generation models
lc models --code

# Audio models
lc models --audio
```

#### Filter by Context Length
```bash
# Minimum 128k context
lc models --ctx 128k

# Minimum 200k context
lc models --ctx 200k
```

#### Filter by Token Limits
```bash
# Minimum input tokens
lc models --input 100k

# Minimum output tokens
lc models --output 32k
```

#### Filter by Price
```bash
# Max input price per million tokens
lc models --input-price 10

# Max output price per million tokens
lc models --output-price 20
```

### Combining Filters

Filters can be combined for precise results:

```bash
# Vision models with 128k+ context
lc models --vision --ctx 128k

# Code models with function calling
lc models --code --tools

# Claude models with reasoning
lc models -q claude --reasoning

# Affordable models with large context
lc models --ctx 100k --input-price 5
```

### Subcommands

#### Refresh Cache
Update the models cache:

```bash
lc models refresh
lc m r
```

#### Show Cache Info
Display cache statistics:

```bash
lc models info
lc m i
```

Output shows:
- Cache location
- Last update time
- Number of providers
- Total models cached

#### Dump Raw Data
Export raw provider responses:

```bash
lc models dump
lc m d
```

Outputs JSON data for debugging or analysis.

#### List Embedding Models
Show only embedding models:

```bash
lc models embed
lc m e
```

## Model Metadata

Each model displays rich metadata when available:

```
openai:
  • gpt-4-turbo-preview 🔧 👁 💻 (128k ctx) (GPT-4 Turbo Preview)
  • gpt-4 🔧 💻 (8k ctx) (GPT-4)
  • gpt-3.5-turbo 🔧 (16k ctx) (GPT-3.5 Turbo)
```

### Capability Indicators

- **🔧 tools** - Supports function calling/tool use
- **👁 vision** - Can process images
- **🧠 reasoning** - Advanced reasoning capabilities
- **💻 code** - Optimized for code generation
- **🔊 audio** - Can process audio input

### Context Information

Shows maximum context window:
- `(8k ctx)` - 8,000 tokens
- `(128k ctx)` - 128,000 tokens
- `(200k ctx)` - 200,000 tokens

### Display Names

Human-readable names in parentheses:
- `(GPT-4 Turbo)` - Marketing name
- `(Claude 3.5 Sonnet)` - Version info

## Examples

### Find Specific Models

```bash
# All GPT models
lc models -q gpt

# Claude models
lc models -q claude

# Llama models
lc models -q llama
```

### Find Models by Use Case

```bash
# For code review (code + reasoning)
lc models --code --reasoning

# For image analysis
lc models --vision

# For long documents
lc models --ctx 100k

# For production (with tools)
lc models --tools
```

### Budget-Conscious Selection

```bash
# Cheap models for testing
lc models --input-price 1 --output-price 2

# Best value for large context
lc models --ctx 32k --input-price 5
```

## Cache Management

The models command uses a local cache to improve performance:

- **Location**: Platform-specific config directory
- **Automatic refresh**: When cache is stale
- **Manual refresh**: `lc models refresh`
- **Cache duration**: 24 hours (configurable)

## Troubleshooting

### "No models found"

1. Refresh the cache:
   ```bash
   lc models refresh
   ```

2. Check provider configuration:
   ```bash
   lc providers list
   ```

3. Verify API keys are set

### "Cache error"

1. Clear cache and refresh:
   ```bash
   rm ~/.config/lc/models_cache.json
   lc models refresh
   ```

2. Check disk space and permissions

### Missing Models

Some providers may not expose all models via API:
- Check provider documentation
- Use `lc providers models <provider>` for direct query
- Some models may require special access

## Next Steps

- [Providers Command](/commands/providers) - Manage model providers
- [Chat Command](/commands/chat) - Use models interactively
- [Direct Prompts](/features/direct-prompts) - Quick model usage
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

#### Model Metadata Configuration

Manage how model metadata is extracted from provider APIs:

##### Add Model Path

Add a new JSON path for extracting models from provider responses:

```bash
lc models add-path ".results[]"
lc models add-path ".data.models[]"
```

##### Remove Model Path

Remove a JSON path from the extraction configuration:

```bash
lc models remove-path ".results[]"
```

##### List Model Paths

Show all configured model extraction paths:

```bash
lc models list-paths
```

Output shows the JQ-style paths used to extract model arrays from different provider API responses.

##### Add Tag Rule

Add a new tag extraction rule for model metadata:

```bash
# Add a boolean tag
lc models add-tag "supports_streaming" ".streaming_enabled,.features.streaming" "bool"

# Add a numeric tag with transform
lc models add-tag "max_tokens" ".limits.max_tokens" "u32"

# Add a price tag with million multiplier
lc models add-tag "input_cost" ".pricing.input" "f64" "multiply_million"
```

Parameters:
- `name`: Tag name (e.g., "supports_streaming")
- `paths`: Comma-separated JSON paths to check
- `type`: Value type ("bool", "u32", "f64", "string")
- `transform`: Optional transform ("multiply_million")

##### List Tag Rules

Show all configured tag extraction rules:

```bash
lc models list-tags
```

Output shows:
- Tag names and types
- JSON paths for each tag
- Transform functions (if any)

## Model Metadata Configuration

The models command uses a configurable metadata extraction system that can be customized for different providers and API formats.

### Configuration Files

Two configuration files control metadata extraction:

#### `model_paths.toml`

Defines JSON paths for extracting model arrays from provider API responses:

```toml
paths = [
    ".data[]",      # OpenAI format
    ".models[]",    # Anthropic format
    ".results[]",   # Custom provider format
    "."             # Single model response
]
```

#### `tags.toml`

Defines rules for extracting metadata fields from model objects:

```toml
[tags.context_length]
paths = [".context_length", ".context_window", ".max_context_length"]
value_type = "u32"

[tags.supports_vision]
paths = [".supports_vision", ".capabilities.vision"]
value_type = "bool"

[tags.input_price_per_m]
paths = [".pricing.prompt", ".pricing.input.usd"]
value_type = "f64"
transform = "multiply_million"
```

### Configuration Location

Configuration files are automatically created in:

- **Linux/macOS**: `~/.config/lc/`
- **Windows**: `%APPDATA%\lc\`

### Adding New Providers

To support a new provider's API format:

1. **Add model extraction path**:
   ```bash
   lc models add-path ".your_provider_models[]"
   ```

2. **Add custom metadata tags** (if needed):
   ```bash
   lc models add-tag "custom_field" ".provider_specific_field" "string"
   ```

3. **Test extraction**:
   ```bash
   lc models refresh
   lc models -q your_provider
   ```

### HuggingFace Support

The system includes special handling for HuggingFace models that have multiple providers. Models with a `providers` array are automatically expanded into separate entries for each provider.

## Model Metadata Display

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

### Configuration Examples

#### Adding Support for a New Provider

```bash
# 1. Add the provider's model extraction path
lc models add-path ".models.available[]"

# 2. Add custom metadata fields
lc models add-tag "max_context" ".context.maximum" "u32"
lc models add-tag "supports_json" ".features.json_mode" "bool"

# 3. Refresh and test
lc models refresh
lc models list-tags
```

#### Customizing Existing Tags

```bash
# Add alternative paths for context length
lc models add-tag "context_length" ".ctx_len,.context_size,.max_tokens" "u32"

# Add pricing with custom transform
lc models add-tag "cost_per_token" ".pricing.per_token" "f64" "multiply_million"
```

#### Managing Configuration

```bash
# View current model paths
lc models list-paths

# View current tag rules
lc models list-tags

# Remove unused paths
lc models remove-path ".deprecated_format[]"
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

4. Check model extraction paths:

   ```bash
   lc models list-paths
   ```

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

### Missing Metadata

If models appear but lack metadata (no capability icons):

1. **Check tag configuration**:
   ```bash
   lc models list-tags
   ```

2. **Add missing tag rules**:
   ```bash
   lc models add-tag "supports_tools" ".tools_enabled,.capabilities.functions" "bool"
   ```

3. **Refresh cache**:
   ```bash
   lc models refresh
   ```

### New Provider Not Working

If a new provider's models aren't appearing:

1. **Check API response format** (use `lc models dump`):
   ```bash
   lc models dump | jq '.your_provider'
   ```

2. **Add appropriate model path**:
   ```bash
   # If models are in .data.models array
   lc models add-path ".data.models[]"
   
   # If models are in .results array
   lc models add-path ".results[]"
   ```

3. **Test extraction**:
   ```bash
   lc models refresh
   lc models -q your_provider
   ```

### Configuration Issues

1. **Reset to defaults**:
   ```bash
   rm ~/.config/lc/model_paths.toml
   rm ~/.config/lc/tags.toml
   lc models refresh  # Will recreate with defaults
   ```

2. **Check configuration location**:
   - Linux/macOS: `~/.config/lc/`
   - Windows: `%APPDATA%\lc\`

3. **Validate TOML syntax**:
   ```bash
   # Check if files are valid TOML
   cat ~/.config/lc/tags.toml
   ```

## See Also

- [Providers Command](providers.md)
- [Chat Command](chat.md)
- [Alias Command](alias.md)

## Next Steps

- [Providers Command](/commands/providers) - Manage model providers
- [Chat Command](/commands/chat) - Use models interactively
- [Chat Command](/commands/chat) - Quick model usage

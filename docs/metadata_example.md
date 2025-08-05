# Model Metadata Extraction Example

## Overview

The new model metadata system provides a generic, configurable approach to extracting model information from provider APIs.

## Key Features

### 1. Configurable Model Paths

Define JQ-style paths to extract models from different API response structures:

```bash
# List current paths
lc models path list

# Add a new path
lc models path add ".models[]"
lc models path add ".data[].models"

# Remove a path
lc models path delete ".models[]"
```

### 2. Tag-Based Metadata Extraction

Define extraction rules for metadata tags using JQ-style paths:

```bash
# List all tags and their rules
lc models tags list

# Add a new rule to a tag
lc models tags add tools "supports_function_calling"
lc models tags add vision "image, capabilities[]"
```

### 3. Model Filtering by Tags

Filter models based on their extracted tags:

```bash
# Filter models with specific tags
lc models filter --tag tools,vision

# Use tags in the main models command
lc models --tag tools,vision,reasoning
```

### 4. Custom Model Addition

Add custom models with specific metadata:

```bash
# Add a custom model to a provider
lc provider models openai add gpt-5-turbo --tag "ctx=200k,out=4096,tools,vision"
```

## Configuration Files

The system uses two main configuration files:

1. **model_paths.toml** - Defines paths for extracting models from API responses
2. **tags.toml** - Defines extraction rules for each metadata tag

These files are stored in:
- macOS: `~/Library/Application Support/lc/`
- Linux: `~/.config/lc/`
- Windows: `%APPDATA%\lc\`

## Example Tag Rules

```toml
[tags]
ctx = [
    "model.context_length",
    "model.context_window",
    "model.max_context_length",
    "context_length",
    "max_model_len"
]

tools = [
    "model.supports_tools",
    "supports_function_calling",
    "tool, capabilities[]",
    "tool, features[]"
]

vision = [
    "image, model.id",
    "supports_image_input",
    "supports_vision",
    "image, capabilities[]"
]
```

## Special Handling for HuggingFace

HuggingFace models with multiple providers are automatically expanded:

- `Qwen/Qwen3-Coder-480B:novita`
- `Qwen/Qwen3-Coder-480B:fireworks-ai`
- `Qwen/Qwen3-Coder-480B:together`

Each variant includes provider-specific metadata like pricing and context length.
# Model Metadata System Rewrite Summary

## Overview

The model metadata system has been completely rewritten to provide a generic, configurable approach for extracting model information from provider APIs, replacing the previous hardcoded provider-specific extraction methods.

## Key Changes

### 1. New `model_metadata.rs` Implementation

- **Generic Extraction**: Replaced provider-specific extraction functions with a configurable system
- **JQ-style Path Support**: Uses JQ-like paths to extract data from JSON responses
- **Tag-based Metadata**: Metadata is extracted using configurable tag rules
- **Custom Model Support**: Ability to add custom models with specific metadata

### 2. Configuration Files

Two new configuration files are automatically created:

- **`model_paths.toml`**: Defines paths to extract models from API responses
- **`tags.toml`**: Defines extraction rules for metadata tags

### 3. CLI Updates

New commands added:

```bash
# Model paths management
lc models path list              # List extraction paths
lc models path add <path>        # Add a new path
lc models path delete <path>     # Remove a path

# Tags management
lc models tags list              # List all tags and rules
lc models tags add <tag> <rule>  # Add a rule to a tag

# Model filtering
lc models filter --tag <tags>    # Filter by tags
lc models --tag <tags>           # Use tags in main command

# Custom models
lc provider models <provider> add <model> --tag <tags>
```

### 4. HuggingFace Special Handling

Models from HuggingFace with multiple providers are automatically expanded:
- Each provider variant gets its own entry (e.g., `model:provider`)
- Provider-specific metadata is preserved

### 5. Tag System

Default tags include:
- `ctx`: Context length
- `out`: Output tokens
- `input_price`: Input pricing
- `output_price`: Output pricing
- `tools`: Tool/function calling support
- `vision`: Vision/image support
- `reasoning`: Reasoning capabilities
- `audio`: Audio support
- `embed`: Embedding support

## Benefits

1. **Flexibility**: Easy to adapt to new providers without code changes
2. **Maintainability**: No more hardcoded extraction logic
3. **Extensibility**: Users can add custom extraction rules
4. **Consistency**: Uniform approach across all providers
5. **User Control**: Users can customize extraction for their needs

## Migration

The system is backward compatible - existing cached models will continue to work. The new extraction will be used when models are refreshed.

## Example Usage

```bash
# Add a custom extraction path
lc models path add ".models[].data"

# Add a custom tag rule
lc models tags add "multimodal" "multimodal, capabilities[]"

# Filter models by multiple tags
lc models filter --tag tools,vision,ctx

# Add a custom model
lc provider models openai add gpt-5 --tag "ctx=200k,tools,vision"
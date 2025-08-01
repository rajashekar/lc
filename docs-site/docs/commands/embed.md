---
id: embed
title: Embed Command
sidebar_position: 8
---

# Embed Command

Generate embeddings for text using various embedding models. The embed command converts text into high-dimensional vectors that can be stored in vector databases for semantic search and RAG applications.

## Overview

Text embeddings are essential for building semantic search systems, recommendation engines, and RAG (Retrieval-Augmented Generation) workflows. The embed command supports multiple embedding models and can process both direct text input and files.

## Usage

```bash
# Generate embeddings for text
lc embed "Your text here"

# Specify model and provider
lc embed -m text-embedding-3-small --provider openai "Important information"

# Store in vector database
lc embed -v knowledge-base "Document content"

# Process files
lc embed -f document.txt,data.pdf

# Using aliases
lc e "Sample text"
```

## Subcommands

The `embed` command is a standalone command without subcommands. All functionality is controlled through options and arguments.

## Options

| Short | Long          | Description                           | Default |
|-------|---------------|---------------------------------------|---------|
| `-m`  | `--model`     | Embedding model to use                | None    |
| `-p`  | `--provider`  | Provider for the embedding model      | None    |
| `-v`  | `--vectordb`  | Vector database to store embeddings   | None    |
| `-f`  | `--files`     | Files to process (comma-separated)    | None    |
| `-d`  | `--debug`     | Enable debug output                   | False   |
| `-h`  | `--help`      | Print help                            | False   |

## Examples

### Basic Text Embedding

```bash
# Simple text embedding
lc embed "Machine learning is transforming software development"

# With specific model
lc embed -m text-embedding-3-large "Complex technical documentation"
```

### Store in Vector Database

```bash
# Create embeddings and store in database
lc embed -v docs "Important company policy information"

# Add multiple pieces of content
lc embed -v knowledge "User manual section 1"
lc embed -v knowledge "User manual section 2"
lc embed -v knowledge "FAQ answers"
```

### Process Files

```bash
# Single file
lc embed -f documentation.md -v docs

# Multiple files
lc embed -f "manual.pdf,guide.txt,readme.md" -v knowledge

# With specific model
lc embed -m text-embedding-ada-002 -f data.txt -v research
```

### RAG Workflow

```bash
# Step 1: Create embeddings from documents
lc embed -f "docs/*.md" --vectordb project-docs

# Step 2: Use in chat with vector context
lc -v project-docs "How do I deploy the application?"

# Step 3: Find similar content
lc similar -v project-docs "deployment process"
```

## See Also

- [Vectors Command](vectors.md)
- [Similar Command](similar.md)

## Troubleshooting

### Common Issues

#### "Embedding model not found"

- **Error**: Specified embedding model doesn't exist
- **Solution**: Use `lc models embed` to list available embedding models
- **Solution**: Check provider supports the model

#### "File not found"

- **Error**: Specified file doesn't exist
- **Solution**: Check file paths and permissions
- **Solution**: Use absolute paths for clarity

#### "Vector database error"

- **Error**: Cannot connect to or create vector database
- **Solution**: Ensure database name is valid
- **Solution**: Check disk space and permissions

#### "Rate limiting"

- **Error**: Too many embedding requests
- **Solution**: Add delays between requests
- **Solution**: Use batch processing for large files

### Best Practices

1. **Choose appropriate models**: Larger models provide better quality but cost more
2. **Chunk large texts**: Break long documents into smaller sections
3. **Consistent models**: Use the same embedding model for search and storage
4. **Batch processing**: Process multiple texts together for efficiency

### Embedding Models

```bash
# List available embedding models
lc models embed

# Common embedding models:
# - text-embedding-3-small (OpenAI)
# - text-embedding-3-large (OpenAI)
# - text-embedding-ada-002 (OpenAI)
# - embed-english-v3.0 (Cohere)
```

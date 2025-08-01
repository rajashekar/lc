---
id: embeddings
title: Embeddings
sidebar_position: 2
---

# Embeddings

Generate and store text embeddings for semantic search and RAG applications.

## Overview

Embeddings are numerical representations of text that capture semantic meaning. LLM Client can:

- Generate embeddings using any compatible model
- Store embeddings in vector databases
- Process files with intelligent chunking
- Enable semantic search and RAG

## Quick Start

```bash
# Generate embeddings (output to console)
lc embed -m text-embedding-3-small "Your text here"

# Store in vector database
lc embed -m text-embedding-3-small -v knowledge "Important information"

# Embed a file
lc embed -m text-embedding-3-small -v docs -f README.md
```

## Command: `lc embed` (alias: `lc e`)

### Options

- `-m, --model <MODEL>` - Embedding model to use (required)
- `-v, --vector-db <NAME>` - Vector database to store in (optional)
- `-f, --file <PATH>` - File(s) to embed (optional)
- `-p, --provider <NAME>` - Provider to use (optional)

### Embedding Models

List available embedding models:

```bash
lc models embed
lc m e
```

Common models:

- `text-embedding-3-small` (OpenAI) - Fast, efficient
- `text-embedding-3-large` (OpenAI) - Higher quality
- `embed-english-v3.0` (Cohere) - English-optimized
- `embed-multilingual-v3.0` (Cohere) - Multi-language

## Text Embedding

### Direct Text

Embed a single piece of text:

```bash
# Output embeddings to console
lc embed -m text-embedding-3-small "Machine learning is fascinating"

# Store in database
lc embed -m text-embedding-3-small -v ml-knowledge "Neural networks use layers"
```

### Multiple Texts

Store multiple embeddings in a database:

```bash
# Build a knowledge base
lc embed -m text-embedding-3-small -v knowledge "Fact 1: Python is popular for ML"
lc embed -m text-embedding-3-small -v knowledge "Fact 2: TensorFlow is a framework"
lc embed -m text-embedding-3-small -v knowledge "Fact 3: GPUs accelerate training"
```

## File Embedding

### Single File

Embed an entire file with automatic chunking:

```bash
lc embed -m text-embedding-3-small -v docs -f README.md
```

### Multiple Files

Use glob patterns or comma-separated lists:

```bash
# Glob pattern
lc embed -m text-embedding-3-small -v docs -f "*.md"
lc embed -m text-embedding-3-small -v docs -f "docs/**/*.txt"

# Multiple specific files
lc embed -m text-embedding-3-small -v docs -f "file1.md,file2.txt,file3.py"
```

### How File Chunking Works

1. **Chunk Size**: ~1200 characters per chunk
2. **Overlap**: 200 characters between chunks
3. **Boundaries**: Respects sentences and paragraphs
4. **Metadata**: Stores file path and chunk info

Example output:

```
Processing file: README.md
  ✓ Chunk 1/5 embedded
  ✓ Chunk 2/5 embedded
  ✓ Chunk 3/5 embedded
  ✓ Chunk 4/5 embedded
  ✓ Chunk 5/5 embedded
Successfully embedded 5 chunks from README.md
```

## Supported File Types

### Text Documents

- `.txt`, `.md`, `.markdown`
- `.rst`, `.org`, `.tex`
- `.rtf`

### Code Files

- `.py`, `.js`, `.ts`, `.java`
- `.cpp`, `.c`, `.h`, `.hpp`
- `.go`, `.rs`, `.rb`, `.php`
- `.swift`, `.kt`, `.scala`

### Web Files

- `.html`, `.css`, `.scss`
- `.xml`, `.json`, `.yaml`, `.yml`

### Configuration

- `.toml`, `.ini`, `.cfg`, `.conf`
- `.env`, `.properties`

### Other

- `.sql`, `.graphql`
- `.dockerfile`, `.makefile`
- `.sh`, `.bash`, `.zsh`

Binary files are automatically filtered out.

## Provider Selection

### Default Provider

Uses the embedding model's default provider:

```bash
# Uses OpenAI for text-embedding-3-small
lc embed -m text-embedding-3-small -v docs "content"
```

### Specific Provider

Override with a different provider:

```bash
# Use Cohere
lc embed -p cohere -m embed-english-v3.0 -v docs "content"

# Use local provider
lc embed -p ollama -m nomic-embed-text -v local "content"
```

## Best Practices

### 1. Model Selection

Choose based on your needs:

- **Speed**: `text-embedding-3-small`
- **Quality**: `text-embedding-3-large`
- **Multilingual**: `embed-multilingual-v3.0`
- **Local/Private**: Ollama models

### 2. Chunk Size Optimization

Default chunking works well, but consider:

- **Technical docs**: Smaller chunks (better precision)
- **Narrative text**: Larger chunks (better context)
- **Code**: Respect function boundaries

### 3. Database Organization

Create purpose-specific databases:

```bash
# Separate databases by content type
lc embed -v technical-docs -f "docs/api/*.md"
lc embed -v tutorials -f "docs/tutorials/*.md"
lc embed -v code-examples -f "examples/**/*.py"
```

### 4. Regular Updates

Keep embeddings current:

```bash
# Re-embed changed files
lc embed -v docs -f "updated-doc.md"

# Process new files
lc embed -v docs -f "new-docs/*.md"
```

## Viewing Embeddings

### Check Database Content

```bash
lc vectors info docs
```

Shows:

- Total entries
- Model used
- Recent additions
- File metadata

### Search Similar Content

```bash
lc similar -v docs "search query"
```

## Performance Tips

### Batch Processing

Process multiple files efficiently:

```bash
# Good: Single command
lc embed -m text-embedding-3-small -v docs -f "*.md"

# Less efficient: Multiple commands
lc embed -m text-embedding-3-small -v docs -f "file1.md"
lc embed -m text-embedding-3-small -v docs -f "file2.md"
```

### Large Files

For very large files:

1. Consider splitting beforehand
2. Monitor memory usage
3. Use streaming processing (automatic)

### API Rate Limits

- Respect provider rate limits
- Use smaller models for bulk processing
- Consider local models for large volumes

## Troubleshooting

### "Model not found"

```bash
# List available embedding models
lc models embed

# Use exact model name
lc embed -m text-embedding-3-small "text"
```

### "File not found"

- Check file path relative to current directory
- Use quotes for paths with spaces
- Verify glob patterns with `ls`

### "Binary file skipped"

This is normal - binary files are automatically filtered. Only text files are processed.

### "Chunk too large"

Rare, but can happen with files containing very long lines. The system will automatically handle this.

## Next Steps

- [Vector Database Guide](/advanced/vector-database) - Managing embeddings
- [Similarity Search](/commands/similar) - Finding related content
- [RAG Guide](/advanced/rag) - Using embeddings for enhanced chat

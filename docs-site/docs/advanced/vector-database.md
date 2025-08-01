---
id: vector-database
title: Vector Database
sidebar_position: 1
---

# Vector Database

LLM Client includes a powerful built-in vector database system that enables you to store text embeddings and perform semantic search. This is the foundation for Retrieval-Augmented Generation (RAG).

## Overview

The vector database allows you to:

- Store text content with embeddings
- Perform similarity searches
- Build knowledge bases
- Enable RAG-enhanced conversations

## Quick Start

```bash
# 1. Generate and store embeddings
lc embed -m text-embedding-3-small -v knowledge "Machine learning is a subset of AI"

# 2. Search for similar content
lc similar -v knowledge "What is neural network programming?"

# 3. Use in conversations
lc -v knowledge "Explain the relationship between AI and ML"
```

## Creating a Vector Database

Vector databases are created automatically when you store your first embedding:

```bash
# This creates a database named "docs" and stores the embedding
lc embed -m text-embedding-3-small -v docs "Your first document"
```

## Managing Databases

### List All Databases

```bash
lc vectors list
lc v l
```

Output:

```
Vector databases:
  • knowledge (1,234 entries, model: text-embedding-3-small, provider: openai)
  • docs (567 entries, model: text-embedding-3-large, provider: openai)
  • codebase (890 entries, model: embed-english-v3.0, provider: cohere)
```

### View Database Information

```bash
lc vectors info <database>
lc v i <database>
```

Output shows:

- Total number of entries
- Embedding model used
- Provider used
- Recent entries with timestamps

Example:

```bash
lc vectors info knowledge
# Output:
# Vector database: knowledge
# Entries: 1,234
# Model: text-embedding-3-small
# Provider: openai
# 
# 📝 Recent entries:
#   1. Machine learning is a subset of AI... (2025-01-28 10:15)
#   2. Deep learning uses neural networks... (2025-01-28 10:16)
#   3. Python is popular for data science... (2025-01-28 10:17)
```

### Delete a Database

```bash
lc vectors delete <database>
lc v d <database>
```

## File Embedding

One of the most powerful features is the ability to embed entire files or directories:

### Single File

```bash
lc embed -m text-embedding-3-small -v docs -f README.md
```

### Multiple Files

```bash
# Using glob patterns
lc embed -m text-embedding-3-small -v docs -f "*.md"
lc embed -m text-embedding-3-small -v docs -f "src/**/*.rs"

# Multiple specific files
lc embed -m text-embedding-3-small -v docs -f "file1.txt,file2.md"
```

### How File Embedding Works

1. **Intelligent Chunking**: Files are split into ~1200 character chunks with 200 character overlap
2. **Smart Boundaries**: Chunks break at sentence, paragraph, or line boundaries
3. **Metadata Storage**: Each chunk stores file path, chunk index, and total chunks
4. **Binary Detection**: Automatically filters out binary files

### Supported File Types

The system automatically detects and processes text files:

**Documentation**: `.txt`, `.md`, `.markdown`, `.rst`, `.org`

**Code**: `.rs`, `.py`, `.js`, `.ts`, `.java`, `.cpp`, `.go`, `.rb`, `.php`

**Web**: `.html`, `.css`, `.xml`, `.json`, `.yaml`, `.yml`

**Config**: `.toml`, `.ini`, `.cfg`, `.conf`

## Storage Details

### Location

Vector databases are stored in platform-specific directories:

| Platform | Directory |
|----------|-----------|
| **Linux** | `~/.config/lc/embeddings/` |
| **macOS** | `~/Library/Application Support/lc/embeddings/` |
| **Windows** | `%APPDATA%\lc\embeddings\` |

### Database Structure

Each database is a SQLite file containing:

- `vectors` table: Text content, embeddings, and metadata
- `model_info` table: Embedding model and provider information
- Optimized indexes for fast similarity search

## Model Consistency

The vector database ensures consistency by:

- Storing the model and provider used for embeddings
- Automatically using the same model for similarity searches
- Validating vector dimensions match the stored model

## Best Practices

### 1. Choose the Right Model

- Smaller models (e.g., `text-embedding-3-small`) are faster and cheaper
- Larger models (e.g., `text-embedding-3-large`) provide better accuracy
- Use the same model consistently within a database

### 2. Organize by Purpose

Create separate databases for different types of content:

```bash
lc embed -v technical-docs -f "docs/technical/*.md"
lc embed -v user-guides -f "docs/guides/*.md"
lc embed -v api-reference -f "docs/api/*.md"
```

### 3. Regular Updates

Keep your databases current:

```bash
# Re-embed updated files
lc embed -v docs -f "updated-file.md"
```

### 4. Chunk Size Considerations

The default chunk size (1200 chars) works well for most content. For specialized needs:

- Smaller chunks: Better for precise retrieval
- Larger chunks: Better for maintaining context

## Performance

The vector database is optimized for:

- **Fast Retrieval**: Indexed similarity searches
- **Efficient Storage**: Compressed embeddings
- **Scalability**: Handles thousands of documents
- **Low Memory**: Streaming processing for large files

## Integration with RAG

Vector databases seamlessly integrate with chat:

```bash
# Direct prompt with context
lc -v knowledge "What do you know about Python?"

# Interactive chat with context
lc chat -v knowledge -m gpt-4
```

The system automatically:

1. Generates embeddings for your query
2. Finds similar content (similarity > 0.3)
3. Includes relevant context in the prompt
4. Returns an enhanced response

## Troubleshooting

### "Database not found"

- Check spelling: `lc vectors list`
- Ensure you've stored at least one embedding

### "Dimension mismatch"

- The database uses a specific embedding model
- Check model info: `lc vectors info <database>`
- Use the same model for all operations

### "No similar content found"

- Verify database has content: `lc vectors info <database>`
- Try different search terms
- Check if content is relevant to your query

## Next Steps

- [Embeddings Guide](/advanced/embeddings) - Detailed embedding operations
- [RAG Guide](/advanced/rag) - Using vector databases for enhanced chat
- [Similarity Search](/commands/similar) - Advanced search techniques

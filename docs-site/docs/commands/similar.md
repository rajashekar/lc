---
id: similar
title: Similar Command
sidebar_position: 9
---

# Similar Command

Find similar text using vector similarity search. The similar command searches vector databases for content semantically related to your query, enabling powerful document retrieval and knowledge discovery.

## Overview

Vector similarity search finds documents based on semantic meaning rather than exact text matches. This enables finding related content even when different words are used, making it essential for RAG applications and knowledge retrieval.

## Usage

```bash
# Search for similar content
lc similar -v <database> "<query>"

# Limit results
lc similar -v docs -l 3 "deployment process"

# With specific model
lc similar -v knowledge -m text-embedding-3-large "machine learning"

# Using aliases
lc s -v docs "search query"
```

## Subcommands

The `similar` command is a standalone command without subcommands. All functionality is controlled through options and the query argument.

## Options

| Short | Long         | Description                                         | Default |
|-------|--------------|-----------------------------------------------------|---------|
| `-m`  | `--model`    | Model to use for embeddings (optional if database has existing model) | None    |
| `-p`  | `--provider` | Provider to use for embeddings (optional if database has existing model) | None    |
| `-v`  | `--vectordb` | Vector database name to search                      | None    |
| `-l`  | `--limit`    | Number of similar results to return                 | 5       |
| `-h`  | `--help`     | Print help                                          | False   |

## Examples

### Basic Similarity Search

**Simple Query**

```bash
lc similar -v project-docs "How to deploy"
# Output:
# Similarity: 0.89
# Text: "To deploy the application, follow these steps..."
# Source: deployment-guide.md
# 
# Similarity: 0.85
# Text: "The deployment process involves building..."
# Source: build-process.md
```

**Limit Results**

```bash
lc similar -v docs -l 3 "authentication error"
# Returns top 3 most similar results

lc s -v docs -l 1 "quick start"
# Returns only the most similar result
```

**Specify Model**

```bash
# Use specific embedding model (if different from database default)
lc similar -v research -m text-embedding-3-large "neural networks"

# With provider
lc similar -v knowledge --provider openai -m text-embedding-ada-002 "API usage"
```

### Research and Discovery

```bash
# Find related documentation
lc similar -v docs "configuration settings"

# Discover related concepts
lc similar -v research-papers "transformer architecture"

# Find troubleshooting guides
lc similar -v support-kb "connection timeout"
```

## See Also

- [Embed Command](embed.md)
- [Vectors Command](vectors.md)

### Integration with RAG Workflow

```bash
# Step 1: Find relevant context
lc similar -v project-docs "API rate limiting" -l 3

# Step 2: Use results to inform chat
lc -v project-docs "How do I handle API rate limits?"

# Or combine both in one step
lc -v project-docs "Tell me about rate limiting based on our docs"
```

### Advanced Queries

```bash
# Technical concepts
lc similar -v codebase "error handling patterns"

# Natural language queries
lc similar -v docs "What happens when the server is down?"

# Comparative queries
lc similar -v research "differences between methods"
```

## Troubleshooting

### Common Issues

#### "Vector database not found"

- **Error**: Specified database doesn't exist
- **Solution**: Use `lc vectors list` to see available databases
- **Create**: Use `lc embed -v <name>` to create database first

#### "No similar results found"

- **Error**: Query returns no matches
- **Solution**: Try broader or different terms
- **Check**: Verify database contains relevant content
- **Adjust**: Increase similarity threshold or result limit

#### "Model mismatch"

- **Error**: Specified model different from database model
- **Solution**: Use same model as when creating embeddings
- **Check**: Use `lc vectors info <db>` to see database model

#### "Empty database"

- **Error**: Database has no vectors
- **Solution**: Add content with `lc embed -v <db> "content"`
- **Verify**: Check with `lc vectors info <db>`

### Best Practices

1. **Consistent Models**: Use same embedding model for indexing and searching
2. **Query Phrasing**: Try different phrasings for better results
3. **Result Limits**: Adjust `-l` parameter based on needs
4. **Quality Content**: Better source content gives better similarity results

### Optimizing Search Results

**Query Techniques**:

```bash
# Specific terms
lc similar -v docs "Docker container deployment"

# Questions
lc similar -v docs "How do I configure SSL?"

# Concepts
lc similar -v docs "security best practices"

# Error messages (partial)
lc similar -v logs "connection refused"
```

**Result Analysis**:

- Similarity scores range from 0.0 to 1.0 (higher = more similar)
- Scores above 0.8 typically indicate very relevant content
- Scores below 0.6 may be less useful

### Performance Considerations

- Large databases may have slower search times
- Lower result limits (`-l`) improve speed
- Consider database size when setting expectations
- Query complexity affects processing time

```bash
# Fast searches
lc similar -v small-db -l 1 "quick query"

# Comprehensive searches
lc similar -v large-db -l 10 "detailed analysis"
```

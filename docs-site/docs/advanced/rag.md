---
id: rag
title: RAG (Retrieval-Augmented Generation)
sidebar_position: 3
---

# RAG (Retrieval-Augmented Generation)

RAG enhances LLM responses by automatically including relevant context from your vector databases. This allows the model to provide more accurate, contextual answers based on your specific knowledge base.

## How RAG Works

1. **Query Processing**: Your question is converted to embeddings
2. **Similarity Search**: Relevant content is retrieved from the vector database
3. **Context Injection**: Retrieved content is added to your prompt
4. **Enhanced Response**: The LLM responds with knowledge from your data

## Quick Start

```bash
# 1. Build a knowledge base
lc embed -m text-embedding-3-small -v knowledge -f "docs/*.md"

# 2. Use RAG in direct prompts
lc -v knowledge "How does the authentication system work?"

# 3. Use RAG in interactive chat
lc chat -v knowledge -m gpt-4
```

## RAG in Direct Prompts

The simplest way to use RAG:

```bash
# Basic usage
lc -v <database> "your question"

# With specific model
lc -v docs -m gpt-4 "Explain the API endpoints"

# With specific provider
lc -v knowledge -p openai -m gpt-4 "What are the best practices?"
```

## RAG in Interactive Chat

Start a RAG-enhanced chat session:

```bash
lc chat -v <database> -m <model>
```

In this mode:
- Every message is automatically enhanced with relevant context
- The system maintains conversation history
- Context is refreshed for each new message

## How Context is Selected

The RAG system uses intelligent filtering:

1. **Similarity Threshold**: Only includes content with >0.3 similarity score
2. **Relevance Ranking**: Most similar content appears first
3. **Token Limits**: Prevents context overflow
4. **Deduplication**: Avoids redundant information

## Building Effective Knowledge Bases

### 1. Domain-Specific Databases

Create focused databases for better retrieval:

```bash
# Technical documentation
lc embed -v tech-docs -f "docs/technical/*.md"

# API reference
lc embed -v api-ref -f "docs/api/*.json"

# Code examples
lc embed -v examples -f "examples/**/*.py"
```

### 2. Chunking Strategy

Files are automatically chunked for optimal retrieval:
- **Chunk Size**: ~1200 characters
- **Overlap**: 200 characters
- **Boundaries**: Respects sentences and paragraphs

### 3. Regular Updates

Keep your knowledge base current:

```bash
# Re-embed updated files
lc embed -v docs -f "updated-file.md"

# Add new content
lc embed -v docs "New important information"
```

## Advanced RAG Patterns

### Multi-Database RAG

Use multiple databases for comprehensive context:

```bash
# Create specialized databases
lc embed -v code -f "src/**/*.rs"
lc embed -v docs -f "docs/**/*.md"
lc embed -v issues -f "issues/**/*.txt"

# Query across databases (one at a time)
lc -v code "How is authentication implemented?"
lc -v docs "What does the documentation say about auth?"
```

### Iterative Refinement

Build knowledge incrementally:

```bash
# Start with high-level concepts
lc embed -v knowledge "Machine learning is a subset of AI"

# Add specific details
lc embed -v knowledge "Supervised learning uses labeled data"

# Add examples
lc embed -v knowledge "Example: spam detection uses supervised learning"
```

### Context-Aware Prompting

Structure prompts for better RAG results:

```bash
# Good: Specific question
lc -v docs "What are the authentication endpoints in the API?"

# Better: Context + question
lc -v docs "In the context of our REST API, what are the authentication endpoints?"

# Best: Structured query
lc -v docs "List all authentication-related API endpoints with their methods and parameters"
```

## RAG Response Format

When using RAG, the system automatically formats context:

```
Based on the following relevant information:
• [Relevant content 1]
• [Relevant content 2]
• [Relevant content 3]

[Your question]
```

This helps the LLM understand what information to prioritize.

## Performance Optimization

### 1. Model Selection

- Use the same embedding model consistently
- Smaller models (text-embedding-3-small) are faster
- Larger models provide better semantic understanding

### 2. Database Size

- Optimal: 100-10,000 documents
- Large databases may slow retrieval
- Consider splitting very large corpora

### 3. Query Optimization

- Be specific in your questions
- Use domain terminology consistently
- Avoid overly broad queries

## Troubleshooting RAG

### "No relevant context found"

- Check database content: `lc vectors info <database>`
- Try different query terms
- Ensure database has related content

### "Context seems unrelated"

- Review similarity threshold
- Check if embedding model matches
- Consider re-embedding with better chunking

### "Response ignores context"

- Ensure model has sufficient context window
- Try a more capable model (e.g., gpt-4)
- Restructure your question

## Best Practices

1. **Start Small**: Build focused databases first
2. **Test Retrieval**: Use `lc similar` to verify quality
3. **Iterate**: Refine your knowledge base based on results
4. **Monitor**: Check which content is being retrieved
5. **Update**: Keep databases current with new information

## Example Workflow

```bash
# 1. Create a project knowledge base
lc embed -v project -f "README.md"
lc embed -v project -f "docs/*.md"
lc embed -v project -f "src/**/*.py"

# 2. Test retrieval
lc similar -v project "authentication flow"

# 3. Use in development
lc -v project "How do I add a new API endpoint?"

# 4. Interactive problem-solving
lc chat -v project -m gpt-4
> How can I optimize the database queries?
> What's the current caching strategy?
```

## Next Steps

- [Vector Database Guide](/advanced/vector-database) - Managing your knowledge bases
- [Embeddings Guide](/advanced/embeddings) - Creating effective embeddings
- [Similarity Search](/commands/similar) - Testing your retrieval
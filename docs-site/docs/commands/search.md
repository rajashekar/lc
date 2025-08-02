---
sidebar_position: 17
---

# Search Commands

The `search` command allows you to integrate web search capabilities into your LLM prompts, providing real-time information and context.

## Overview

```bash
lc search [SUBCOMMAND]
```

The search functionality supports multiple search providers (currently Brave Search, Exa, and Serper) and can be used both as a standalone search tool and integrated into your LLM prompts.

## Subcommands

### Provider Management

#### Add a Search Provider

```bash
lc search provider add <NAME> <URL> [-t <TYPE>]
# or
lc search p a <NAME> <URL> [-t <TYPE>]
```

Add a new search provider with the specified API endpoint.

**Options:**
- `-t, --type <TYPE>`: Provider type (`brave`, `exa`, or `serper`, default: `brave`)

**Examples:**

```bash
# Add Brave Search
lc search provider add brave https://api.search.brave.com/res/v1/web/search -t brave

# Add Exa Search
lc search provider add exa https://api.exa.ai -t exa

# Add Serper (Google Search API)
lc search provider add serper https://google.serper.dev -t serper
```

#### List Search Providers

```bash
lc search provider list
# or
lc search p l
```

Display all configured search providers and their status.

#### Delete a Search Provider

```bash
lc search provider delete <NAME>
# or
lc search p d <NAME>
```

Remove a search provider from your configuration.

#### Set Provider Headers

```bash
lc search provider set <PROVIDER> <HEADER_NAME> <HEADER_VALUE>
# or
lc search p s <PROVIDER> <HEADER_NAME> <HEADER_VALUE>
```

Configure authentication headers for a search provider.

**Examples:**

```bash
# For Brave Search
lc search provider set brave X-Subscription-Token YOUR_API_KEY

# For Exa
lc search provider set exa x-api-key YOUR_API_KEY

# For Serper
lc search provider set serper X-API-KEY YOUR_API_KEY
```

### Direct Search

```bash
lc search query <PROVIDER> <QUERY> [OPTIONS]
```

Perform a direct search using the specified provider.

**Options:**

- `-f, --format <FORMAT>`: Output format (`json` or `md`/`markdown`, default: `md`)
- `-n, --count <COUNT>`: Number of results to return (default: 5)

**Examples:**

```bash
# Markdown output (default)
lc search query brave "rust programming language"

# JSON output
lc search query brave "quantum computing" -f json

# Get 10 results
lc search query brave "AI research papers" -n 10
```

## Integration with LLM Prompts

### Using --use-search Flag

The `--use-search` flag allows you to automatically include search results as context in your LLM prompts:

```bash
lc --use-search <PROVIDER> "Your prompt here"
```

**Examples:**

```bash
# Use default search query (your prompt)
lc --use-search brave "What are the latest developments in quantum computing?"

# Specify custom search query
lc --use-search "brave:quantum computing 2024" "Summarize the recent breakthroughs"
```

### Search Query Formats

When using `--use-search`, you can specify the search in two ways:

1. **Provider only**: Uses your prompt as the search query

   ```bash
   lc --use-search brave "What is happening with AI regulation?"
   ```

2. **Provider:query**: Uses a specific search query

   ```bash
   lc --use-search "brave:AI regulation EU 2024" "Analyze the implications"
   ```

## Configuration

### Set Default Search Provider

```bash
lc config set search <PROVIDER>
```

Set a default search provider for use with `--use-search`:

```bash
lc config set search brave
```

### Get Default Search Provider

```bash
lc config get search
```

### Delete Default Search Provider

```bash
lc config delete search
```

## Complete Setup Examples

### Brave Search Setup

```bash
# 1. Add Brave as a search provider
lc search provider add brave https://api.search.brave.com/res/v1/web/search -t brave

# 2. Set your API key
lc search provider set brave X-Subscription-Token YOUR_BRAVE_API_KEY

# 3. Set as default provider (optional)
lc config set search brave

# 4. Test direct search
lc search query brave "OpenAI GPT-4" -f json

# 5. Use in LLM prompts
lc --use-search brave "What are the latest AI safety developments?"

# 6. Use with custom search query
lc --use-search "brave:AI safety research 2024" "Provide a comprehensive summary"
```

### Exa Setup

```bash
# 1. Add Exa as a search provider
lc search provider add exa https://api.exa.ai -t exa

# 2. Set your API key
lc search provider set exa x-api-key YOUR_EXA_API_KEY

# 3. Set as default provider (optional)
lc config set search exa

# 4. Test direct search
lc search query exa "machine learning best practices" -f json

# 5. Use in LLM prompts
lc --use-search exa "What are the latest developments in neural networks?"

# 6. Use with custom search query
lc --use-search "exa:transformer architecture improvements 2024" "Explain the recent advances"
```

### Serper Setup

```bash
# 1. Add Serper as a search provider
lc search provider add serper https://google.serper.dev -t serper

# 2. Set your API key
lc search provider set serper X-API-KEY YOUR_SERPER_API_KEY

# 3. Set as default provider (optional)
lc config set search serper

# 4. Test direct search
lc search query serper "latest AI developments" -f json

# 5. Use in LLM prompts
lc --use-search serper "What are the current trends in artificial intelligence?"

# 6. Use with custom search query
lc --use-search "serper:GPT-4 alternatives 2024" "Compare the latest language models"
```

## Supported Providers

Currently supported search providers:

- **Brave Search**: Fast, independent search engine with good API support
  - API Documentation: [Brave Search API](https://brave.com/search/api/)
  - Requires API key (subscription required)
  - Header: `X-Subscription-Token`

- **Exa**: AI-powered search engine optimized for finding high-quality, relevant content
  - API Documentation: [Exa API](https://exa.ai/)
  - Requires API key
  - Header: `x-api-key`
  - Features: Neural search, content extraction, similarity search

- **Serper**: Google Search API providing comprehensive search results
  - API Documentation: [Serper API](https://serper.dev/)
  - Requires API key
  - Header: `X-API-KEY`
  - Features: Google search results, rich snippets, comprehensive metadata

Future providers may include:

- DuckDuckGo
- SerpAPI
- Google Custom Search
- Bing Search

## Tips and Best Practices

1. **API Keys**: Store your API keys securely and never commit them to version control
2. **Result Count**: For general queries, 3-5 results are usually sufficient. For research, consider 10+
3. **Query Optimization**: Be specific with your search queries for better results
4. **Context Integration**: The search results are automatically formatted and prepended to your prompt
5. **Rate Limits**: Be aware of your search provider's rate limits and pricing

## Troubleshooting

### Common Issues

1. **401 Unauthorized**: Check your API key is correctly set

   ```bash
   lc search provider set brave X-Subscription-Token YOUR_KEY
   ```

2. **No results found**: Verify your search query and provider URL

   ```bash
   lc search provider list
   ```

3. **Provider not found**: Ensure the provider is added

   ```bash
   lc search provider add brave https://api.search.brave.com/res/v1/web/search
   ```

## Examples

### Research Assistant

```bash
lc --use-search "brave:latest machine learning papers arxiv 2024" \
  "Summarize the most important recent developments in machine learning"
```

### News Analysis

```bash
lc --use-search brave "What happened in tech news today?"
```

### Fact Checking

```bash
lc --use-search "brave:climate change statistics 2024" \
  "Verify and explain the latest climate data"
```

### Competitive Analysis

```bash
lc --use-search "brave:OpenAI competitors 2024" \
  "Analyze the current competitive landscape in AI"

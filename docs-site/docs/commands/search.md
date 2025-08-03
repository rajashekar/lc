---
sidebar_position: 17
---

# Search Commands

The `search` command allows you to integrate web search capabilities into your LLM prompts, providing real-time information and context from 6 different search providers.

## Overview

```bash
lc search [SUBCOMMAND]
```

The search functionality supports **6 search providers** with automatic type detection and can be used both as a standalone search tool and integrated into your LLM prompts.

## Supported Search Providers

| Provider | URL Pattern | API Key Required | Special Features |
|----------|-------------|------------------|------------------|
| **Brave** | `api.search.brave.com` | ✅ Yes | Fast, independent search |
| **Exa** | `api.exa.ai` | ✅ Yes | AI-powered neural search |
| **Serper** | `google.serper.dev` | ✅ Yes | Google search results |
| **SerpApi** | `serpapi.com` | ✅ Yes | Rich metadata, comprehensive results |
| **DuckDuckGo** | `api.duckduckgo.com` | ❌ **Free** | No API key needed |
| **Jina AI** | `s.jina.ai` | ✅ Yes | AI search + full content reading |

## Subcommands

### Provider Management

#### Add a Search Provider

```bash
lc search provider add <NAME> <URL>
# or
lc search p a <NAME> <URL>
```

Add a new search provider with **automatic type detection** from the URL pattern.

**Examples:**

```bash
# Add Brave Search (auto-detected as 'brave')
lc search provider add brave https://api.search.brave.com/res/v1/web/search

# Add Exa Search (auto-detected as 'exa')
lc search provider add exa https://api.exa.ai/search

# Add Serper (auto-detected as 'serper')
lc search provider add serper https://google.serper.dev/search

# Add SerpApi (auto-detected as 'serpapi')
lc search provider add serpapi https://serpapi.com/search

# Add DuckDuckGo (auto-detected as 'duckduckgo')
lc search provider add ddg https://api.duckduckgo.com/

# Add Jina AI (auto-detected as 'jina')
lc search provider add jina https://s.jina.ai/
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

Configure authentication headers and options for a search provider.

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
lc search query jina "quantum computing" -f json

# Get 10 results
lc search query ddg "AI research papers" -n 10
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
lc --use-search "jina:quantum computing 2024" "Summarize the recent breakthroughs"
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
# 1. Add Brave as a search provider (auto-detected)
lc search provider add brave https://api.search.brave.com/res/v1/web/search

# 2. Set your API key
lc search provider set brave X-Subscription-Token YOUR_BRAVE_API_KEY

# 3. Set as default provider (optional)
lc config set search brave

# 4. Test direct search
lc search query brave "OpenAI GPT-4" -f json

# 5. Use in LLM prompts
lc --use-search brave "What are the latest AI safety developments?"
```

### Exa Setup

```bash
# 1. Add Exa as a search provider (auto-detected)
lc search provider add exa https://api.exa.ai/search

# 2. Set your API key
lc search provider set exa x-api-key YOUR_EXA_API_KEY

# 3. Test direct search
lc search query exa "machine learning best practices" -f json

# 4. Use in LLM prompts
lc --use-search exa "What are the latest developments in neural networks?"
```

### Serper Setup

```bash
# 1. Add Serper as a search provider (auto-detected)
lc search provider add serper https://google.serper.dev/search

# 2. Set your API key
lc search provider set serper X-API-KEY YOUR_SERPER_API_KEY

# 3. Test direct search
lc search query serper "latest AI developments" -f json

# 4. Use in LLM prompts
lc --use-search serper "What are the current trends in artificial intelligence?"
```

### SerpApi Setup

```bash
# 1. Add SerpApi as a search provider (auto-detected)
lc search provider add serpapi https://serpapi.com/search

# 2. Set your API key
lc search provider set serpapi api_key YOUR_SERPAPI_KEY

# 3. Test direct search
lc search query serpapi "machine learning research 2024" -f json

# 4. Use in LLM prompts
lc --use-search serpapi "Summarize recent ML breakthroughs"
```

### DuckDuckGo Setup (Free!)

```bash
# 1. Add DuckDuckGo as a search provider (auto-detected)
lc search provider add ddg https://api.duckduckgo.com/

# 2. No API key required! ✅

# 3. Test direct search
lc search query ddg "rust programming tutorials" -f json

# 4. Use in LLM prompts
lc --use-search ddg "What are good resources for learning Rust?"
```

### Jina AI Setup (Advanced Features)

```bash
# 1. Add Jina AI as a search provider (auto-detected)
lc search provider add jina https://s.jina.ai/

# 2. Set your API key
lc search provider set jina Authorization YOUR_JINA_API_KEY

# 3. Test basic search
lc search query jina "rust async programming" -f json

# 4. Enable full content reading (X-Engine: direct)
lc search provider set jina X-Engine direct

# 5. Test with full content (much richer results!)
lc search query jina "rust async programming" -f json

# 6. Enable JSON format for structured responses
lc search provider set jina Accept application/json

# 7. Use in LLM prompts with rich content
lc --use-search jina "Explain Rust async programming concepts"
```

## Provider-Specific Features

### Jina AI Advanced Features

Jina AI offers unique capabilities beyond standard search:

#### Full Content Reading

Enable `X-Engine: direct` to get complete page content instead of just snippets:

```bash
# Enable full content reading
lc search provider set jina X-Engine direct

# Now searches return full page content (much richer!)
lc search query jina "topic" -f json
```

**Benefits:**
- ✅ Complete article content (thousands of characters)
- ✅ No need to visit individual URLs
- ✅ Perfect for research and AI analysis
- ⚠️ Slower response times
- ⚠️ Higher API costs

#### Response Formats

```bash
# Text format (default)
lc search query jina "topic"

# JSON format
lc search provider set jina Accept application/json
lc search query jina "topic" -f json
```

### DuckDuckGo (Free Option)

DuckDuckGo is the only provider that requires **no API key**:

```bash
# Just add and use - no authentication needed!
lc search provider add ddg https://api.duckduckgo.com/
lc search query ddg "your search query"
```

Perfect for:
- ✅ Testing search functionality
- ✅ Users without API budgets
- ✅ Privacy-focused searches
- ⚠️ Limited to basic instant answers

## Provider Comparison

| Feature | Brave | Exa | Serper | SerpApi | DuckDuckGo | Jina AI |
|---------|-------|-----|--------|---------|------------|---------|
| **Cost** | Paid | Paid | Paid | Paid | **Free** | Paid |
| **Search Quality** | High | AI-Enhanced | Google Results | Google Results | Basic | AI-Enhanced |
| **Speed** | Fast | Fast | Fast | Fast | Fast | Fast/Slow* |
| **Rich Snippets** | ✅ | ✅ | ✅ | ✅ | Limited | ✅ |
| **Full Content** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Metadata** | ✅ | ✅ | ✅ | ✅ | Limited | ✅ |

*With X-Engine: direct enabled

## Authentication Headers

Each provider uses different authentication methods:

| Provider | Header Name | Format | Example |
|----------|-------------|--------|---------|
| **Brave** | `X-Subscription-Token` | Direct | `YOUR_API_KEY` |
| **Exa** | `x-api-key` | Direct | `YOUR_API_KEY` |
| **Serper** | `X-API-KEY` | Direct | `YOUR_API_KEY` |
| **SerpApi** | `api_key` | Query Param | `YOUR_API_KEY` |
| **DuckDuckGo** | None | N/A | No auth required |
| **Jina AI** | `Authorization` | Bearer | `Bearer YOUR_API_KEY` |

## Tips and Best Practices

1. **Start with DuckDuckGo**: Test search functionality for free before getting API keys
2. **API Keys**: Store your API keys securely and never commit them to version control
3. **Result Count**: For general queries, 3-5 results are sufficient. For research, consider 10+
4. **Query Optimization**: Be specific with your search queries for better results
5. **Provider Selection**: 
   - Use **DuckDuckGo** for free basic searches
   - Use **Brave/Serper** for general web search
   - Use **Exa** for AI-enhanced content discovery
   - Use **Jina AI** for research requiring full content
6. **Rate Limits**: Be aware of your search provider's rate limits and pricing
7. **Jina Full Content**: Only enable `X-Engine: direct` when you need comprehensive content

## Troubleshooting

### Common Issues

1. **401 Unauthorized**: Check your API key and header format

   ```bash
   # Check current providers
   lc search provider list
   
   # Set correct API key for each provider
   lc search provider set brave X-Subscription-Token YOUR_KEY
   lc search provider set jina Authorization YOUR_KEY
   ```

2. **Auto-detection failed**: Ensure you're using the correct URL pattern

   ```bash
   # These URLs will auto-detect correctly:
   lc search provider add brave https://api.search.brave.com/res/v1/web/search
   lc search provider add jina https://s.jina.ai/
   lc search provider add ddg https://api.duckduckgo.com/
   ```

3. **No results found**: Try a different provider or check your query

   ```bash
   # Test with free DuckDuckGo first
   lc search query ddg "test query"
   ```

4. **Jina parsing errors**: Check if you need both headers for JSON + full content

   ```bash
   lc search provider set jina Accept application/json
   lc search provider set jina X-Engine direct
   ```

## Examples

### Research Assistant with Full Content

```bash
# Use Jina with full content reading for comprehensive research
lc search provider set jina X-Engine direct
lc --use-search "jina:latest machine learning papers arxiv 2024" \
  "Summarize the most important recent developments in machine learning"
```

### Free News Analysis

```bash
# Use free DuckDuckGo for basic news queries
lc --use-search ddg "What happened in tech news today?"
```

### Fact Checking with Google Results

```bash
# Use Serper for Google-quality fact checking
lc --use-search "serper:climate change statistics 2024" \
  "Verify and explain the latest climate data"
```

### AI-Enhanced Content Discovery

```bash
# Use Exa for AI-powered content discovery
lc --use-search "exa:best practices machine learning 2024" \
  "What are the current ML best practices?"
```

### Competitive Analysis

```bash
# Use SerpApi for comprehensive competitive research
lc --use-search "serpapi:OpenAI competitors 2024" \
  "Analyze the current competitive landscape in AI"
```

## Getting API Keys

### Free Option
- **DuckDuckGo**: No API key required! ✅

### Paid Options
- **Brave Search**: [Get API Key](https://brave.com/search/api/)
- **Exa**: [Get API Key](https://exa.ai/)
- **Serper**: [Get API Key](https://serper.dev/)
- **SerpApi**: [Get API Key](https://serpapi.com/)
- **Jina AI**: [Get API Key](https://jina.ai/)

Start with DuckDuckGo to test the functionality, then choose paid providers based on your specific needs!

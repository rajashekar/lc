# LC Search Feature Documentation

## Overview

The LC tool now includes a comprehensive search feature that allows users to integrate web search results as context for LLM prompts. The implementation follows a modular architecture supporting multiple search providers.

## Architecture

### Module Structure
```
src/search/
├── mod.rs           # Main search module with provider routing
├── providers.rs     # Provider management and configuration
├── brave.rs         # Brave search provider implementation
├── exa.rs           # Exa search provider implementation
└── serper.rs        # Serper search provider implementation
```

### Key Components

1. **Provider Trait** (`SearchProvider`)
   - Defines the interface for all search providers
   - Methods: `search()`, `name()`, `provider_type()`

2. **Provider Types** (`SearchProviderType`)
   - Enum with variants: `Brave`, `Exa`, `Serper`
   - Serializable for TOML configuration

3. **Search Results** (`SearchResult`)
   - Unified structure for all providers
   - Fields: title, url, snippet, score

4. **Provider Configuration**
   - TOML-based storage
   - Support for headers and authentication
   - Default provider selection

## Supported Providers

### 1. Brave Search
- **Type**: `brave`
- **API**: GET requests to `https://api.search.brave.com/res/v1/web/search`
- **Authentication**: `X-Subscription-Token` header
- **Features**: Web search with snippets, relevance scores

### 2. Exa Search
- **Type**: `exa`
- **API**: POST requests to `https://api.exa.ai/search`
- **Authentication**: `x-api-key` header
- **Features**: AI-powered search, full text content, neural search capabilities

### 3. Serper (Google Search API)
- **Type**: `serper`
- **API**: POST requests to `https://google.serper.dev/search`
- **Authentication**: `X-API-KEY` header
- **Features**: Google search results, rich snippets, comprehensive metadata

## CLI Commands

### Provider Management
```bash
# Add a new provider
lc search provider add <name> <url> -t <type>

# List all providers
lc search provider list

# Set authentication
lc search provider set <name> <header> <value>

# Remove a provider
lc search provider remove <name>

# Set default provider
lc search provider default <name>
```

### Search Commands
```bash
# Search using default provider
lc search <query>

# Search with specific provider
lc search <query> -p <provider>

# Output formats
lc search <query> -f json
lc search <query> -f markdown

# Limit results
lc search <query> -l <limit>
```

### Integration with Prompts
```bash
# Use search results as context
lc prompt "Your question" --use-search "search query"

# With specific provider
lc prompt "Your question" --use-search "search query" --search-provider brave

# With result limit
lc prompt "Your question" --use-search "search query" --search-limit 5
```

## Configuration

The search configuration is stored in `~/.config/lc/search_config.toml`:

```toml
default_provider = "brave"

[providers.brave]
name = "brave"
url = "https://api.search.brave.com/res/v1"
provider_type = "brave"

[providers.brave.headers]
X-Subscription-Token = "YOUR_API_KEY"

[providers.exa]
name = "exa"
url = "https://api.exa.ai"
provider_type = "exa"

[providers.exa.headers]
x-api-key = "YOUR_API_KEY"

[providers.serper]
name = "serper"
url = "https://google.serper.dev"
provider_type = "serper"

[providers.serper.headers]
X-API-KEY = "YOUR_API_KEY"
```

## Implementation Details

### Adding a New Provider

1. Create a new file in `src/search/` (e.g., `google.rs`)
2. Implement the `SearchProvider` trait
3. Add a new variant to `SearchProviderType` enum
4. Update the provider routing in `src/search/mod.rs`
5. Handle provider-specific API requirements

### Error Handling

- Invalid API keys return appropriate error messages
- Network errors are gracefully handled
- Missing providers return helpful error messages
- Invalid configurations are detected and reported

### Testing

Comprehensive test scripts are provided:
- `test_providers.sh` - Tests provider management
- `test_search_integration.sh` - Tests search and prompt integration
- `test_multi_provider.sh` - Tests multiple provider support
- `test_search_complete.sh` - Complete feature demonstration

## Examples

### Setting up Brave Search
```bash
# Add Brave provider
lc search provider add brave https://api.search.brave.com/res/v1 -t brave

# Set API key
lc search provider set brave X-Subscription-Token YOUR_BRAVE_API_KEY

# Set as default
lc search provider default brave

# Search
lc search "rust programming best practices"
```

### Setting up Exa Search
```bash
# Add Exa provider
lc search provider add exa https://api.exa.ai -t exa

# Set API key
lc search provider set exa x-api-key YOUR_EXA_API_KEY

# Search with JSON output
lc search "machine learning trends" -p exa -f json
```

### Setting up Serper Search
```bash
# Add Serper provider
lc search provider add serper https://google.serper.dev -t serper

# Set API key
lc search provider set serper X-API-KEY YOUR_SERPER_API_KEY

# Search with Markdown output
lc search "latest AI developments" -p serper -f md
```

### Using Search with Prompts
```bash
# Get latest information about a topic
lc prompt "Summarize the key points about" --use-search "quantum computing breakthroughs 2024"

# Research with specific provider
lc prompt "What are the environmental impacts of" --use-search "electric vehicles lifecycle" --search-provider exa

# Limited context search
lc prompt "Explain the main concepts from" --use-search "transformer architecture deep learning" --search-limit 3
```

## Future Enhancements

1. Additional search providers (Google, DuckDuckGo, Bing)
2. Search result caching
3. Advanced search operators
4. Search history tracking
5. Custom result formatting templates
6. Parallel search across multiple providers
7. Search result filtering and ranking
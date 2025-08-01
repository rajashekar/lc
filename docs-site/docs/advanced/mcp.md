---
sidebar_position: 5
---

# Model Context Protocol (MCP)

The Model Context Protocol (MCP) enables LLMs to interact with external tools and services through a standardized interface. `lc` supports MCP servers, allowing you to extend your LLM's capabilities with tools for web access, file manipulation, and more.

## Overview

MCP servers are external processes that provide tools (functions) that LLMs can call during conversations. These tools can:
- Fetch content from the internet
- Interact with local files
- Execute system commands
- Connect to APIs and databases
- And much more

## Managing MCP Servers

### Adding an MCP Server

To add a new MCP server to your configuration:

```bash
lc mcp add <name> <command> --type <type> [-e KEY=VALUE]...
```

**Parameters:**
- `<name>`: A unique identifier for the server
- `<command>`: The command to start the server or URL for remote servers
- `--type`: The server type (`stdio`, `sse`, or `streamable`)
- `-e, --env KEY=VALUE`: Set environment variables (can be used multiple times)

**Example:**
```bash
# Add the fetch server for internet access
lc mcp add fetch "uvx mcp-server-fetch" --type stdio

# Add the Playwright server for browser automation
lc mcp add playwright "npx @playwright/mcp@latest" --type stdio

# Add server with environment variables
lc mcp add exa "npx -y exa-mcp-server" --type stdio -e EXA_API_KEY=your_api_key

# Multiple environment variables
lc mcp add custom "uvx custom-mcp" --type stdio -e API_KEY=key -e DEBUG=true
```

### Listing MCP Servers

View all configured MCP servers:

```bash
lc mcp list
```

### Deleting an MCP Server

Remove an MCP server from your configuration:

```bash
lc mcp delete <name>
```

## Working with MCP Servers

### Viewing Available Functions

To see what functions a server provides:

```bash
lc mcp functions <server-name>
```

**Example:**
```bash
lc mcp functions fetch
# Output:
# Functions: Available functions:
#   • fetch - Fetches a URL from the internet and optionally extracts its contents as markdown.
#     Parameters: max_length, raw, start_index, url
```

### Invoking Functions Directly

You can test MCP functions directly:

```bash
lc mcp invoke <server-name> <function-name> [parameters...]
```

**Example:**
```bash
# Fetch content from a URL
lc mcp invoke fetch fetch url=http://httpbin.org/json

# With multiple parameters
lc mcp invoke fetch fetch url=http://example.com max_length=1000
```

### Managing Server Connections

MCP servers are managed automatically by `lc`'s daemon. They start when needed and persist for reuse. If you need to explicitly close a connection:

```bash
# Stop a running server connection
lc mcp stop <server-name>
```

**Note:** With the new SDK implementation, servers start automatically when you use them with the `-t` flag or invoke a function. The `stop` command is only needed if you want to explicitly close a persistent connection.

## Using MCP Tools in Prompts

The real power of MCP comes from using these tools in your LLM conversations. Use the `-t` or `--tools` flag to include MCP servers in your prompt:

### Direct Prompts with Tools

```bash
# Single tool
lc -t fetch "What's the current weather in San Francisco?"

# Multiple tools (comma-separated)
lc -t fetch,playwright "Navigate to example.com and fetch its content"
```

### Interactive Chat with Tools

```bash
# Start chat with MCP tools
lc chat -m claude-3-opus-20240229 -t fetch

# With multiple tools
lc chat -m gpt-4 -t fetch,playwright
```

## Popular MCP Servers

### 1. mcp-server-fetch
Provides internet access to fetch web content.

**Installation:**
```bash
lc mcp add fetch "uvx mcp-server-fetch" --type stdio
```

**Usage Example:**
```bash
lc -t fetch "Get the latest news headlines from a news website"
```

### 2. Playwright MCP
Enables browser automation and web scraping.

**Installation:**
STDIO version
```bash
lc mcp add playwright "npx @playwright/mcp@latest" --type stdio
```
sse version
```bash
lc mcp a playwright-sse http://localhost:8931/sse --type sse
```
streamable version
```bash
lc mcp add playwright-mcp http://localhost:8931/mcp --type streamable
```

**Usage Example:**
```bash
lc -t playwright "Go to https://news.ycombinator.com/ and get me the first post"

output:
The first post on Hacker News is titled **"Show HN: Draw a fish and watch it swim with the others"**. Here are the details:

- **Link**: [Draw a fish](https://drawafish.com)
- **Points**: 610 points
- **Submitted by**: hallak
- **Time ago**: 10 hours ago
- **Comments**: 180 comments

If you need more information or details about other posts, feel free to ask!
```

### 3. Context7 MCP
Provides access to library documentation and code examples.

**Installation:**
```bash
lc mcp add context7 "npx -y @upstash/context7-mcp" --type stdio
```

**Usage Example:**
```bash
lc -t context7 "Show me React hooks documentation"
lc -t context7 "How to use Express.js middleware"
```

### 4. File System MCP
Allows reading and writing local files.

**Installation:**
```bash
lc mcp add fs "uvx mcp-server-fs" --type stdio
```

### 5. Exa search MCP
Access Exa search capabilities for advanced web searches.

**Installation:**
```bash
lc mcp add exa "npx -y exa-mcp-server" --type stdio -e "EXA_API_KEY=your_exa_api_key"
```
**Usage Example:**
```bash
lc -t exa "Search for the latest AI research papers"

output:
Thinking... [EXA-MCP-DEBUG] [web_search_exa-1754081391442-9zlic] [web_search_exa] Starting search for query: "latest AI research papers"
[EXA-MCP-DEBUG] [web_search_exa-1754081391442-9zlic] [web_search_exa] Sending request to Exa API
[EXA-MCP-DEBUG] [web_search_exa-1754081391442-9zlic] [web_search_exa] Received response from Exa API
[EXA-MCP-DEBUG] [web_search_exa-1754081391442-9zlic] [web_search_exa] Found 5 results
[EXA-MCP-DEBUG] [web_search_exa-1754081391442-9zlic] [web_search_exa] Successfully completed request
Here are some of the latest resources and research papers related to AI:

1. **[AI Research Papers - Arize AI](https://arize.com/ai-research-papers/)**
   - Explore the latest technical papers with the Arize Community, which includes trending AI research and resources to stay updated with breakthroughs in AI.

2. **[Publications - Google DeepMind](https://deepmind.google/research/publications/)**
   - Discover the latest AI breakthroughs and updates from Google DeepMind. You can find a selection of recent research on complex and interesting challenges in AI.

3. **[Noteworthy AI Research Papers of 2024 (Part One) - Ahead of AI](https://magazine.sebastianraschka.com/p/ai-research-papers-2024-part-1)**
   - This article highlights significant AI research papers from January to June 2024, focusing on various topics, particularly LLM research.

4. **[Where To Find The Latest AI Research? Top 7 Sources to Stay Updated](https://learnprompting.org/blog/resources_latest_research_papers?srsltid=AfmBOoqNahYsL1aeuJH3ZA15JHlm26BU1agUyhpYqpFNvFSUOm2zWM6Q)**
   - This article discusses different journals, conferences, platforms, and communities for finding the latest AI research papers, along with strategies for staying informed.

5. **[Artificial Intelligence - arXiv](https://arxiv.org/list/cs.AI/recent)**
   - Access the most recent AI research papers published on arXiv. This resource provides a comprehensive list of the latest entries in AI research.

Feel free to explore these links for more detailed insights into the latest advancements in AI research!
```

## Configuration

MCP server configurations are stored in `~/Library/Application Support/lc/mcp.toml` (macOS) or the equivalent directory on other platforms.

**Example configuration:**
```toml
[servers.fetch]
name = "fetch"
server_type = "Stdio"
command_or_url = "uvx mcp-server-fetch"

[servers.fetch.env]

[servers.playwright]
name = "playwright"
server_type = "Stdio"
command_or_url = "npx @playwright/mcp@latest"

[servers.playwright.env]

[servers.kagi]
name = "kagi"
server_type = "Stdio"
command_or_url = "uvx kagimcp"

[servers.kagi.env]
KAGI_API_KEY = "your_api_key_here"
```

### Environment Variables

MCP servers can require environment variables for configuration, such as API keys or debug flags. These are stored securely in the configuration file and passed to the server when it starts.

**Security Note:** Be careful not to commit your `mcp.toml` file to version control if it contains sensitive API keys.

## Best Practices

1. **Tool Selection**: Only include the tools you need for a specific task to avoid confusion
2. **Testing**: Use `lc mcp invoke` to test tools before using them in prompts
3. **Error Handling**: Some tools may fail due to network issues or permissions - always have a fallback plan
4. **Security**: Be cautious with tools that can execute commands or access sensitive data

## Troubleshooting

### Connection Issues
If an MCP server fails to connect:
1. Check if the command is installed (`uvx`, `npx`, etc.)
2. Verify network connectivity for remote servers
3. Check environment variables are set correctly if required
4. Check server logs in `~/Library/Application Support/lc/`

### HTTPS/SSL Issues
Some MCP servers may have issues with HTTPS connections. Try:
- Using HTTP URLs when possible
- Checking proxy settings
- Updating the MCP server to the latest version

### Server Not Responding
If a server stops responding:
```bash
# Stop the server connection
lc mcp stop <server-name>

# The server will restart automatically when you use it again
lc mcp invoke <server-name> <function-name> <args>
```

## Examples

### Fetching and Analyzing Web Content
```bash
lc -t fetch "Analyze the content structure of https://example.com and summarize the main sections"
```

### Research Assistant
```bash
lc -t fetch "Research the latest developments in quantum computing and provide a summary with sources"
```

### Multi-Tool Automation
```bash
lc -t fetch,fs "Fetch the latest documentation from https://docs.example.com and save it to a local file"
```

## Advanced Usage

### Custom MCP Servers
You can create your own MCP servers. See the [MCP SDK documentation](https://github.com/modelcontextprotocol/sdk) for details.

### Remote MCP Servers
MCP supports Server-Sent Events (SSE) for remote servers:
```bash
lc mcp add remote-server "http://localhost:8080/sse" --type sse
```

This allows you to host MCP servers on remote machines or in the cloud.
---
sidebar_position: 4
---

# MCP Commands

The `mcp` command group manages Model Context Protocol servers, which extend your LLM's capabilities with external tools.

## Overview

```bash
lc mcp <subcommand> [options]
```

## Subcommands

### `add` - Add an MCP Server

Add a new MCP server to your configuration.

```bash
lc mcp add <name> <command> --type <type> [-e KEY=VALUE]...
```

**Arguments:**
- `<name>` - Unique identifier for the server
- `<command>` - Command to start the server or URL for remote servers
- `--type` - Server type: `stdio`, `sse`, or `streamable`

**Options:**
- `-e, --env KEY=VALUE` - Set environment variables (can be used multiple times)

**Examples:**
```bash
# Add fetch server for internet access
lc mcp add fetch "uvx mcp-server-fetch" --type stdio

# Add Playwright server for browser automation
lc mcp add playwright "npx @playwright/mcp@latest" --type stdio

# Add remote SSE server
lc mcp add remote "http://localhost:8080/sse" --type sse

# Add server with environment variables
lc mcp add kagi "uvx kagimcp" --type stdio -e KAGI_API_KEY=your_api_key

# Add server with multiple environment variables
lc mcp add myserver "uvx myserver" --type stdio -e API_KEY=key -e DEBUG=true
```

### `delete` (alias: `d`) - Delete an MCP Server

Remove an MCP server from your configuration.

```bash
lc mcp delete <name>
```

**Example:**
```bash
lc mcp delete fetch
```

### `list` (alias: `l`) - List MCP Servers

Display all configured MCP servers.

```bash
lc mcp list
```

**Example output:**
```
Servers: Configured MCP servers:
  • fetch - Stdio (uvx mcp-server-fetch)
  • playwright - Stdio (npx @playwright/mcp@latest)
```

### `functions` (alias: `f`) - List Server Functions

Show available functions/tools provided by an MCP server.

```bash
lc mcp functions <name>
```

**Example:**
```bash
lc mcp functions fetch
# Output:
# Functions: Available functions:
#   • fetch - Fetches a URL from the internet and optionally extracts its contents as markdown.
#     Parameters: max_length, raw, start_index, url
```

### `invoke` (alias: `i`) - Invoke a Function

Call a specific function from an MCP server directly.

```bash
lc mcp invoke <server> <function> [parameters...]
```

**Parameters format:** `key=value`

**Examples:**
```bash
# Fetch a URL
lc mcp invoke fetch fetch url=http://example.com

# With multiple parameters
lc mcp invoke fetch fetch url=http://example.com max_length=1000

# Browser automation
lc mcp invoke playwright navigate url=https://google.com
```

### `stop` (alias: `st`) - Stop an MCP Server

Close a persistent MCP server connection managed by the daemon.

```bash
lc mcp stop <name>
```

**Example:**
```bash
lc mcp stop fetch
```

**Note:** With the new SDK implementation, MCP servers are started automatically when needed. The `stop` command is only needed if you want to explicitly close a persistent connection.

## Using MCP Tools in Prompts

Once MCP servers are configured, use them in your prompts with the `-t` or `--tools` flag:

### Direct Prompts
```bash
# Single tool
lc -t fetch "What's the weather in Paris?"

# Multiple tools
lc -t fetch,playwright "Go to weather.com and get the forecast"
```

### Interactive Chat
```bash
# Start chat with tools
lc chat -m gpt-4 -t fetch

# Multiple tools in chat
lc chat -m claude-3-opus -t fetch,playwright
```

## Common MCP Servers

### mcp-server-fetch
Provides internet access to fetch web content.

```bash
# Install
lc mcp add fetch "uvx mcp-server-fetch" --type stdio

# Test
lc mcp invoke fetch fetch url=http://httpbin.org/json

# Use in prompt
lc -t fetch "Summarize the latest tech news"
```

### Playwright MCP
Browser automation and web scraping.

```bash
# Install
lc mcp add playwright "npx @playwright/mcp@latest" --type stdio

# Test
lc mcp invoke playwright screenshot url=https://example.com

# Use in prompt
lc -t playwright "Take a screenshot of google.com"
```

### Context7 MCP
Access library documentation and code examples.

```bash
# Install
lc mcp add context7 "npx -y @upstash/context7-mcp" --type stdio

# Test
lc mcp functions context7

# Use in prompt
lc -t context7 "Show me React useState examples"
lc -t context7 "How to use Express.js routing"
```

### File System MCP
Read and write local files.

```bash
# Install
lc mcp add fs "uvx mcp-server-fs" --type stdio

# Use in prompt
lc -t fs "Read the contents of config.json"
```

### Kagi MCP
Access Kagi search and summarization capabilities.

```bash
# Install (requires Kagi API key)
lc mcp add kagi "uvx kagimcp" --type stdio -e KAGI_API_KEY=your_kagi_api_key

# Test
lc mcp functions kagi

# Use in prompt
lc -t kagi "Search for the latest AI developments"
lc -t kagi "Summarize this article: https://example.com/article"
```

**Note:** Requires Python 3.12+ and a valid Kagi API key from https://kagi.com/settings?p=api

## Configuration

MCP configurations are stored in:
- **macOS/Linux**: `~/.config/lc/mcp.toml` or `~/Library/Application Support/lc/mcp.toml`
- **Windows**: `%APPDATA%\lc\mcp.toml`

Example configuration:
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

## Troubleshooting

### Server Won't Start
- Ensure the command is installed (`uvx`, `npx`, etc.)
- Check if the command works when run directly
- Check environment variables are set correctly if required
- The server starts automatically when you use it with `-t` flag or invoke a function

### Connection Issues
- For HTTPS issues, try HTTP URLs when possible
- Check firewall/proxy settings
- Verify network connectivity

### Function Invocation Fails
- Check parameter names and types with `lc mcp functions <name>`
- Ensure boolean parameters are not quoted
- The server will be started automatically when needed

## Examples

### Web Research
```bash
# Add fetch server
lc mcp add fetch "uvx mcp-server-fetch" --type stdio

# Research a topic
lc -t fetch "Research the latest developments in quantum computing"
```

### Automated Testing
```bash
# Add browser automation
lc mcp add playwright "npx @playwright/mcp@latest" --type stdio

# Test a website
lc -t playwright "Navigate to my website at example.com and check if the login button is visible"
```

### Multi-Tool Workflows
```bash
# Use multiple tools together
lc -t fetch,fs "Fetch the latest documentation from https://docs.example.com and save it to docs.md"
```

## See Also

- [Advanced MCP Guide](/advanced/mcp) - Detailed MCP documentation
- [Provider Commands](/commands/providers) - Managing LLM providers
- [Model Commands](/commands/models) - Working with models
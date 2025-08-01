---
id: web-chat-proxy
title: Web Chat Proxy Command
sidebar_position: 14
---

# Web Chat Proxy Command

Start a web chat proxy server for non-OpenAI compatible services. This proxy translates chat requests to provider-specific formats and enables web clients to communicate with various LLM providers.

## Overview

The web-chat-proxy bridges the gap between web applications expecting OpenAI-compatible APIs and providers using different formats. It provides translation layers, authentication handling, and unified interfaces for web clients.

## Usage

```bash
# List supported providers
lc web-chat-proxy providers

# Start proxy for a provider
lc web-chat-proxy start <provider>

# Start with custom settings
lc web-chat-proxy start anthropic --port 8080 --host 0.0.0.0

# Using aliases
lc w providers
lc w start claude
```

## Subcommands

| Name        | Alias | Description                      |
|-------------|-------|----------------------------------|
| `providers` | `p`   | List supported providers         |
| `start`     | `s`   | Start proxy server for provider  |
| `stop`      | -     | Stop proxy server               |
| `list`      | `ps`  | List running proxy servers       |

## Options

| Short | Long            | Description                    | Default |
|-------|-----------------|--------------------------------|---------|
| `-p`  | `--port`        | Port to listen on              | 8080    |
|       | `--host`        | Host to bind to                | 127.0.0.1|
| `-k`  | `--key`         | API key for authentication     | None    |
| `-g`  | `--generate-key`| Generate a random API key      | False   |
| `-d`  | `--daemon`      | Run in daemon mode             | False   |
| `-h`  | `--help`        | Print help                     | False   |

## Examples

### Start Web Chat Proxy

**Basic Usage**

```bash
# Start proxy for Claude
lc web-chat-proxy start anthropic

# Start with custom port
lc web-chat-proxy start anthropic --port 3000

# Start in daemon mode
lc web-chat-proxy start anthropic --daemon

# Using aliases
lc w start anthropic
lc w s anthropic --port 3000
```

**With Authentication**

```bash
# Generate API key
lc web-chat-proxy start anthropic --generate-key

# Use custom API key
lc web-chat-proxy start anthropic --key my-secret-key
```

### Management

```bash
# List supported providers
lc web-chat-proxy providers
# Output:
#   • anthropic (Claude)
#   • cohere
#   • groq
#   • together

# List running proxies
lc web-chat-proxy list
# Output:
#   • anthropic:8080 (running)
#   • together:8081 (running)

# Stop a proxy
lc web-chat-proxy stop anthropic
```

### Client Integration

```javascript
// Use proxy in web applications
const response = await fetch('http://localhost:8080/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer your-api-key',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'claude-3.5-sonnet',
    messages: [{ role: 'user', content: 'Hello!' }]
  })
});
```

## Troubleshooting

### Common Issues

#### "Port already in use"

- **Error**: Address already in use
- **Solution**: Use different port with `--port` flag
- **Check**: `netstat -tlnp | grep :8080`

#### "Provider not supported"

- **Error**: Provider doesn't have web chat proxy support
- **Solution**: Use `lc web-chat-proxy providers` to see supported providers
- **Alternative**: Use standard `lc proxy` command instead

#### "Authentication failed"

- **Error**: Invalid API key for proxy
- **Solution**: Verify proxy API key (not provider API key)
- **Generate**: Use `--generate-key` flag to create new key

### Provider-Specific Notes

**Anthropic Claude**:

- Translates OpenAI format to Anthropic Messages API
- Handles system prompts correctly
- Supports streaming responses

**Cohere**:

- Maps OpenAI chat format to Cohere Chat API
- Handles conversation history
- Supports model parameters translation

### Security Considerations

- Use authentication for production deployments
- Bind to localhost (127.0.0.1) for development only
- Use HTTPS in production with reverse proxy
- Rotate API keys regularly

```bash
# Secure production setup
lc web-chat-proxy start anthropic \
  --host 127.0.0.1 \
  --port 8080 \
  --key $(openssl rand -hex 32) \
  --daemon
```

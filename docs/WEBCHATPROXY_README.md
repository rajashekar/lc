# WebChatProxy - OpenAI-Compatible Proxy for Non-OpenAI Services

The `lc webchatproxy` (or `lc w`) feature provides an OpenAI-compatible API proxy for services that don't natively support the OpenAI API format. This allows you to use non-OpenAI services with any OpenAI-compatible client.

## Supported Providers

Currently supported providers:
- **Kagi** - Kagi Assistant API

## Quick Start

### 1. List Supported Providers

```bash
lc w providers list
# or
lc webchatproxy providers list
```

### 2. Set Authentication for a Provider

```bash
# Set Kagi authentication token
lc w providers set kagi auth <your-kagi-token>

# Or set interactively (will prompt for token)
lc w providers set kagi auth
```

### 3. Start the Proxy Server

```bash
# Start proxy for Kagi on default port 8080
lc w start kagi

# Start on custom port and host
lc w start kagi --port 8080 --host 0.0.0.0

# Start with authentication (requires API key for clients)
lc w start kagi --key sk-1234
lc w start kagi -k sk-1234

# Start with generated authentication key
lc w start kagi --generate-key
lc w start kagi -g
```

## Usage Examples

Once the proxy server is running, you can use it with any OpenAI-compatible client:

### Using curl

```bash
# Basic request
curl -X POST http://localhost:8080/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "kagi-assistant",
    "messages": [
      {"role": "user", "content": "What is 2+2?"}
    ]
  }'

# With authentication (if proxy was started with --key)
curl -X POST http://localhost:8080/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-1234" \
  -d '{
    "model": "kagi-assistant",
    "messages": [
      {"role": "user", "content": "What is 2+2?"}
    ]
  }'
```

### Using OpenAI Python Client

```python
import openai

# Configure client to use the proxy
client = openai.OpenAI(
    base_url="http://localhost:8080",
    api_key="sk-1234"  # Only needed if proxy has auth enabled
)

# Make a request
response = client.chat.completions.create(
    model="kagi-assistant",
    messages=[
        {"role": "user", "content": "What is 2+2?"}
    ]
)

print(response.choices[0].message.content)
```

## Available Endpoints

The proxy provides these OpenAI-compatible endpoints:

- `POST /chat/completions` - Chat completions
- `POST /v1/chat/completions` - Chat completions (v1 API)

## Provider-Specific Details

### Kagi

The Kagi provider:
- Uses the Kagi Assistant API (`https://kagi.com/assistant/prompt`)
- Requires a Kagi authentication token (get from Kagi settings)
- Maps OpenAI chat format to Kagi's focus/profile format
- Uses `llama-4-scout` model by default
- Enables internet access and disables personalizations

**Kagi API Request Format:**
```json
{
  "focus": {
    "thread_id": null,
    "branch_id": "00000000-0000-4000-0000-000000000000",
    "prompt": "user message"
  },
  "profile": {
    "id": null,
    "personalizations": false,
    "internet_access": true,
    "model": "llama-4-scout",
    "lens_id": null
  }
}
```

## Configuration

WebChatProxy stores its configuration in `~/.config/lc/webchatproxy.toml`:

```toml
[providers.kagi]
auth_token = "your-kagi-token"
```

## Command Reference

### Main Commands

- `lc w providers list` - List all supported providers
- `lc w providers set <provider> auth [token]` - Set authentication for a provider
- `lc w start <provider> [options]` - Start proxy server for a provider

### Start Command Options

- `--port, -p` - Port to listen on (default: 8080)
- `--host` - Host to bind to (default: 127.0.0.1)
- `--key, -k` - API key for client authentication
- `--generate-key, -g` - Generate a random API key

### Aliases

- `lc w` is an alias for `lc webchatproxy`
- `lc w p` is an alias for `lc w providers`
- `lc w s` is an alias for `lc w start`

## Error Handling

The proxy handles various error conditions:

- **401 Unauthorized** - Invalid or missing authentication
- **400 Bad Request** - Invalid request format or unsupported provider
- **500 Internal Server Error** - Provider API errors or parsing failures

## Security Notes

- Store authentication tokens securely
- Use HTTPS in production environments
- Consider using authentication (`--key`) for public deployments
- The proxy doesn't validate or sanitize provider responses beyond basic parsing

## Extending Support

To add support for new providers:

1. Add provider-specific request/response structures
2. Implement the provider handler in `handle_<provider>_request()`
3. Add response parsing logic in `parse_<provider>_response()`
4. Update the provider list in CLI handlers

## Troubleshooting

### Common Issues

1. **"Unsupported provider" error**
   - Check that the provider name is correct (currently only 'kagi' is supported)

2. **"Authentication required" error**
   - Set the provider auth token: `lc w providers set kagi auth <token>`

3. **"Could not parse response" error**
   - The provider's response format may have changed
   - Check the provider's API documentation for updates

4. **Connection refused**
   - Ensure the proxy server is running
   - Check the correct host and port

### Debug Mode

Run with debug logging to see detailed request/response information:

```bash
lc w start kagi --debug
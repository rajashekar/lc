# LC Proxy Server

The LC proxy server provides an OpenAI-compatible API endpoint that unifies access to all configured LLM providers. This allows you to use any OpenAI-compatible client to access models from different providers through a single interface.

## Quick Start

Start the proxy server with default settings:
```bash
lc proxy
# or use the short alias
lc pr
```

This starts the server on `127.0.0.1:6789` with no authentication.

## Command Options

```bash
lc proxy [OPTIONS]

Options:
  -p, --port <PORT>          Port to listen on [default: 6789]
  -h, --host <HOST>          Host to bind to [default: 127.0.0.1]
      --provider <PROVIDER>  Filter by provider
  -m, --model <MODEL>        Filter by specific model (can be provider:model or alias)
  -k, --key <API_KEY>        API key for authentication
  -g, --generate-key         Generate a random API key
```

## Examples

### Basic Usage

Start server on default port (6789):
```bash
lc pr
```

### Custom Port and Host

Start server on port 8081, accessible from any IP:
```bash
lc pr -p 8081 -h 0.0.0.0
```

### Provider Filtering

Only expose models from the "ollama" provider:
```bash
lc pr --provider ollama
```

### Model Filtering

Only expose a specific model:
```bash
lc pr -m ollama:llama3:8b
# or using an alias
lc pr -m my-alias
```

### With Authentication

Use a custom API key:
```bash
lc pr -k sk-my-secret-key
```

Generate a random API key:
```bash
lc pr -g
```

## API Endpoints

The proxy server exposes the following OpenAI-compatible endpoints:

### List Models
- `GET /models`
- `GET /v1/models`

Returns all available models in OpenAI format:
```json
{
  "object": "list",
  "data": [
    {
      "id": "ollama:llama3:8b",
      "object": "model",
      "created": 1640995200,
      "owned_by": "ollama"
    }
  ]
}
```

### Chat Completions
- `POST /chat/completions`
- `POST /v1/chat/completions`

Accepts OpenAI-compatible chat completion requests:
```json
{
  "model": "ollama:llama3:8b",
  "messages": [
    {
      "role": "user",
      "content": "What is 2+2?"
    }
  ],
  "max_tokens": 1024,
  "temperature": 0.7
}
```

## Usage Examples

### Using curl

List all models:
```bash
curl http://localhost:6789/v1/models
```

Send a chat completion request:
```bash
curl -X POST "http://localhost:6789/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ollama:llama3:8b",
    "messages": [
      {"role": "user", "content": "what is 2+2?"}
    ]
  }'
```

With authentication:
```bash
curl -X POST "http://localhost:6789/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-your-api-key" \
  -d '{
    "model": "ollama:llama3:8b",
    "messages": [
      {"role": "user", "content": "what is 2+2?"}
    ]
  }'
```

### Using Python OpenAI Client

```python
import openai

# Configure the client to use your proxy
client = openai.OpenAI(
    base_url="http://localhost:6789/v1",
    api_key="sk-your-api-key"  # or "dummy" if no auth
)

# List models
models = client.models.list()
print([model.id for model in models.data])

# Chat completion
response = client.chat.completions.create(
    model="ollama:llama3:8b",
    messages=[
        {"role": "user", "content": "What is 2+2?"}
    ]
)
print(response.choices[0].message.content)
```

### Using Node.js OpenAI Client

```javascript
import OpenAI from 'openai';

const openai = new OpenAI({
  baseURL: 'http://localhost:6789/v1',
  apiKey: 'sk-your-api-key', // or 'dummy' if no auth
});

// List models
const models = await openai.models.list();
console.log(models.data.map(m => m.id));

// Chat completion
const completion = await openai.chat.completions.create({
  model: 'ollama:llama3:8b',
  messages: [
    { role: 'user', content: 'What is 2+2?' }
  ],
});
console.log(completion.choices[0].message.content);
```

## Model Specification

Models can be specified in several formats:

1. **Provider:Model format**: `ollama:llama3:8b`
2. **Alias**: If you've created an alias with `lc alias add my-model ollama:llama3:8b`
3. **Model name only**: Uses the default provider (if configured)

## Performance

The proxy server uses LC's built-in models cache for fast response times. The cache:
- Refreshes automatically every 24 hours
- Updates in the background without blocking requests
- Can be manually refreshed with `lc models refresh`

## Authentication

When authentication is enabled:
- Include the API key in the `Authorization` header: `Bearer sk-your-key`
- All requests without valid authentication will return `401 Unauthorized`
- Use `-g` flag to generate a secure random API key

## Filtering

### Provider Filtering
When `--provider` is specified, only models from that provider are exposed.

### Model Filtering
When `--model` is specified, only that specific model (or models matching the filter) are exposed.

## Integration Examples

The proxy server makes it easy to integrate LC with existing OpenAI-compatible tools:

### LangChain
```python
from langchain.llms import OpenAI

llm = OpenAI(
    openai_api_base="http://localhost:6789/v1",
    openai_api_key="sk-your-key",
    model_name="ollama:llama3:8b"
)
```

### LlamaIndex
```python
from llama_index.llms import OpenAI

llm = OpenAI(
    api_base="http://localhost:6789/v1",
    api_key="sk-your-key",
    model="ollama:llama3:8b"
)
```

## Troubleshooting

### Slow Response Times
If `/models` endpoint is slow, refresh the models cache:
```bash
lc models refresh
```

### Connection Refused
- Check if the server is running
- Verify the correct port and host
- Check firewall settings if accessing remotely

### Authentication Errors
- Ensure the API key is correctly specified
- Check the `Authorization` header format: `Bearer sk-your-key`

### Model Not Found
- Verify the model exists with `lc models`
- Check provider configuration with `lc providers list`
- Ensure the provider has a valid API key

## Port Selection

The default port 6789 was chosen because:
- It's unlikely to conflict with common services
- It's easy to remember
- It's in the user port range (1024-65535)

If port 6789 is in use, specify a different port with `-p`:
```bash
lc pr -p 8080
# Authentication Headers in LC Providers

## Overview

LC supports flexible authentication mechanisms for different providers. While many providers use the standard `Authorization: Bearer {api_key}` header, some providers require custom headers for authentication.

## How It Works

### 1. Standard Authentication (Default)
Most providers like OpenAI use the standard Bearer token format:
```
Authorization: Bearer sk-your-api-key
```

### 2. Custom Authentication Headers
Some providers like Google Gemini require custom headers:
```
x-goog-api-key: your-api-key
```

## Configuration

### Provider Configuration (provider.toml)

In the provider's TOML configuration file, you can specify custom headers with a placeholder for the API key:

```toml
# Example: Gemini provider configuration
endpoint = "https://generativelanguage.googleapis.com"
models_path = "/v1beta/models/"
chat_path = "/v1beta/models/{model}:generateContent"

[headers]
x-goog-api-key = "${api_key}"
```

The `${api_key}` placeholder will be automatically replaced with the actual API key from `keys.toml`.

### Keys Configuration (keys.toml)

API keys are stored centrally in `keys.toml`:

```toml
[api_keys]
openai = "sk-proj-your-openai-key"
gemini = "your-gemini-api-key"
anthropic = "sk-ant-your-anthropic-key"
```

## Usage Flow

1. **Install a provider:**
   ```bash
   lc providers install gemini
   ```

2. **Add your API key:**
   ```bash
   lc keys add gemini
   ```
   This will prompt you to enter your API key, which will be stored in `keys.toml`.

3. **Use the provider:**
   ```bash
   lc chat gemini:gemini-pro "Hello, how are you?"
   ```

## How the System Determines Authentication Method

When loading a provider configuration, the system:

1. **Checks for custom headers** with `${api_key}` placeholder in the provider's `[headers]` section
2. **If found:** Replaces `${api_key}` with the actual key and uses the custom header
3. **If not found:** Uses the standard `Authorization: Bearer {api_key}` format

## Examples

### Provider with Standard Auth (OpenAI)
```toml
# openai.toml
endpoint = "https://api.openai.com/v1"
models_path = "/models"
chat_path = "/chat/completions"

[headers]
# No custom auth headers needed - will use standard Bearer token
```

### Provider with Custom Auth Header (Gemini)
```toml
# gemini.toml
endpoint = "https://generativelanguage.googleapis.com"
models_path = "/v1beta/models/"
chat_path = "/v1beta/models/{model}:generateContent"

[headers]
x-goog-api-key = "${api_key}"
```

### Provider with Multiple Custom Headers
```toml
# custom-provider.toml
endpoint = "https://api.custom-provider.com"
models_path = "/v1/models"
chat_path = "/v1/chat"

[headers]
x-api-key = "${api_key}"
x-api-version = "2024-01"
x-client-id = "lc-cli"
```

## Advanced: Multiple Authentication Types

For providers that support multiple authentication methods, you can use the `auth_type` field:

```toml
# vertex.toml
endpoint = "https://us-central1-aiplatform.googleapis.com"
auth_type = "google_sa_jwt"  # Uses service account JWT
```

Supported auth types:
- `api_key` - Standard API key (default)
- `service_account` - Service account JSON
- `oauth` - OAuth token
- `token` - Generic token
- `headers` - Custom headers

## Benefits of This Approach

1. **Flexibility:** Supports any header-based authentication scheme
2. **Security:** API keys are stored separately from provider configs
3. **Portability:** Provider configs can be shared without exposing secrets
4. **Simplicity:** Users only need to run `lc keys add <provider>` regardless of the authentication method

## Migration from Old Format

If you have existing provider configurations with embedded API keys, they will be automatically migrated to the centralized `keys.toml` format when you run any lc command.

## Troubleshooting

### Provider not authenticating correctly?

1. Check that your API key is set:
   ```bash
   lc keys list
   ```

2. Verify the provider's header configuration:
   ```bash
   cat ~/.config/lc/providers/gemini.toml
   ```

3. Ensure the `${api_key}` placeholder is correctly formatted in the headers section

### Need to update an API key?

```bash
lc keys add gemini  # Will overwrite the existing key
```

### Want to remove an API key?

```bash
lc keys remove gemini
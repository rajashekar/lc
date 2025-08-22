# LC Provider Registry

This repository contains the official provider configurations for the `lc` (LLM Client) tool. These configurations enable seamless integration with various LLM providers while keeping sensitive API keys separate and secure.

## Structure

```
provider-registry/
├── registry.json           # Provider metadata and discovery
├── providers/             # Provider configuration files
│   ├── openai.toml
│   ├── anthropic.toml
│   ├── gemini.toml
│   └── ...
└── README.md
```

## How It Works

1. **Provider Discovery**: The `registry.json` file contains metadata about all available providers, including their capabilities, supported models, and authentication requirements.

2. **Provider Installation**: Users can install providers using:
   ```bash
   lc providers install <provider>
   # or shorthand
   lc p i <provider>
   ```

3. **Automatic Updates**: When installing a provider that's already installed, the tool will check for updates and apply them if available.

4. **Secure Key Management**: API keys are stored separately in `keys.toml` and never included in provider configurations.

## Provider Configuration Format

Each provider configuration is a TOML file containing:

- **Basic Settings**: Endpoint URLs, API paths
- **Request/Response Transformations**: Jinja2 templates for API compatibility
- **Headers**: Required HTTP headers (excluding authentication)
- **Model Specifications**: Context lengths, token limits, capabilities

Example structure:
```toml
name = "provider_name"
endpoint = "https://api.example.com/v1"
models_path = "/models"
chat_path = "/chat/completions"

request_transform = """
{
  "model": "{{ model }}",
  "messages": {{ messages | json }}
}
"""

response_transform = """
{{ choices[0].message.content }}
"""

[headers]
Content-Type = "application/json"

[[models]]
id = "model-name"
context_length = 128000
max_output_tokens = 4096
```

## Adding a New Provider

To add a new provider:

1. Create a new TOML file in the `providers/` directory
2. Add the provider metadata to `registry.json`
3. Submit a pull request with your changes

### Provider Metadata Fields

In `registry.json`:
- `name`: Unique identifier (lowercase, underscores for spaces)
- `display_name`: Human-readable name
- `description`: Brief description of the provider
- `version`: Configuration version (semver)
- `auth_type`: Authentication method (`api_key`, `oauth`, `service_account`, `none`)
- `tags`: Capabilities array (e.g., `["chat", "embeddings", "vision"]`)
- `models`: List of supported model IDs

## Authentication Types

- **api_key**: Simple API key authentication
- **oauth**: OAuth 2.0 flow
- **service_account**: Google/AWS service account JSON
- **aws_credentials**: AWS IAM credentials
- **none**: No authentication required (e.g., local models)

## Version Management

Provider configurations are versioned to ensure compatibility:
- Configuration changes that don't break compatibility increment the patch version
- New required fields or structural changes increment the minor version
- Breaking changes increment the major version

## Testing Provider Configurations

Before submitting a new provider:

1. Test the configuration locally:
   ```bash
   # Copy the provider config to your lc config directory
   cp providers/your_provider.toml ~/.config/lc/providers/
   
   # Add your API key
   lc keys add your_provider
   
   # Test the provider
   lc -p your_provider -m model_name "Test prompt"
   ```

2. Verify all endpoints work correctly
3. Ensure request/response transformations handle edge cases

## Contributing

We welcome contributions! Please ensure:
- Provider configurations are tested and working
- Documentation is clear and complete
- Model lists are up-to-date
- Transformations handle streaming responses where applicable

## License

This registry is maintained as part of the `lc` project and follows the same licensing terms.
# Configuration Samples

This directory contains sanitized sample configuration files for the `lc` tool.

## Usage

1. Copy the relevant sample file(s) to your `~/Library/Application Support/lc/` directory
2. For provider files, copy them to `~/Library/Application Support/lc/providers/`
3. Rename them by removing the `_sample` suffix
4. Replace all `<your-*>` placeholders with your actual API keys and credentials

## Files

### Main Configuration Files

- `config_sample.toml` - Sample for `config.toml`
- `mcp_sample.toml` - Sample for `mcp.toml`
- `model_paths_sample.toml` - Sample for `model_paths.toml`
- `search_config_sample.toml` - Sample for `search_config.toml`
- `sync_sample.toml` - Sample for `sync.toml`
- `tags_sample.toml` - Sample for `tags.toml`
- `webchatproxy_sample.toml` - Sample for `webchatproxy.toml`
- `webchatproxy_daemons_sample.toml` - Sample for `webchatproxy_daemons.toml`

### Provider Configuration Files

- `providers/.toml_sample.toml` - Sample for `providers/.toml`
- `providers/ai21_sample.toml` - Sample for `providers/ai21.toml`
- `providers/alibaba_sample.toml` - Sample for `providers/alibaba.toml`
- `providers/amazon_bedrock_sample.toml` - Sample for `providers/amazon_bedrock.toml`
- `providers/cerebras_sample.toml` - Sample for `providers/cerebras.toml`
- `providers/chub_sample.toml` - Sample for `providers/chub.toml`
- `providers/chutes_sample.toml` - Sample for `providers/chutes.toml`
- `providers/claude_sample.toml` - Sample for `providers/claude.toml`
- `providers/cloudflare_sample.toml` - Sample for `providers/cloudflare.toml`
- `providers/cohere_sample.toml` - Sample for `providers/cohere.toml`
- `providers/deepinfra_sample.toml` - Sample for `providers/deepinfra.toml`
- `providers/deepseek_sample.toml` - Sample for `providers/deepseek.toml`
- `providers/digitalocean_sample.toml` - Sample for `providers/digitalocean.toml`
- `providers/fireworks_sample.toml` - Sample for `providers/fireworks.toml`
- `providers/gemini_sample.toml` - Sample for `providers/gemini.toml`
- `providers/github-copilot_sample.toml` - Sample for `providers/github-copilot.toml`
- `providers/github_sample.toml` - Sample for `providers/github.toml`
- `providers/grok_sample.toml` - Sample for `providers/grok.toml`
- `providers/groq_sample.toml` - Sample for `providers/groq.toml`
- `providers/huggingface_sample.toml` - Sample for `providers/huggingface.toml`
- `providers/hyperbolic_sample.toml` - Sample for `providers/hyperbolic.toml`
- `providers/inceptionlabs_sample.toml` - Sample for `providers/inceptionlabs.toml`
- `providers/kagi_sample.toml` - Sample for `providers/kagi.toml`
- `providers/kilo_sample.toml` - Sample for `providers/kilo.toml`
- `providers/lambda_sample.toml` - Sample for `providers/lambda.toml`
- `providers/litellm_sample.toml` - Sample for `providers/litellm.toml`
- `providers/meta_sample.toml` - Sample for `providers/meta.toml`
- `providers/mistral_sample.toml` - Sample for `providers/mistral.toml`
- `providers/moonshot_sample.toml` - Sample for `providers/moonshot.toml`
- `providers/nebius_sample.toml` - Sample for `providers/nebius.toml`
- `providers/novita_sample.toml` - Sample for `providers/novita.toml`
- `providers/nscale_sample.toml` - Sample for `providers/nscale.toml`
- `providers/nvidia_sample.toml` - Sample for `providers/nvidia.toml`
- `providers/ollama_sample.toml` - Sample for `providers/ollama.toml`
- `providers/openai_sample.toml` - Sample for `providers/openai.toml`
- `providers/openrouter_sample.toml` - Sample for `providers/openrouter.toml`
- `providers/perplexity_sample.toml` - Sample for `providers/perplexity.toml`
- `providers/poe_sample.toml` - Sample for `providers/poe.toml`
- `providers/requesty_sample.toml` - Sample for `providers/requesty.toml`
- `providers/sambanova_sample.toml` - Sample for `providers/sambanova.toml`
- `providers/together_sample.toml` - Sample for `providers/together.toml`
- `providers/venice_sample.toml` - Sample for `providers/venice.toml`
- `providers/vercel_sample.toml` - Sample for `providers/vercel.toml`
- `providers/vertex_google_sample.toml` - Sample for `providers/vertex_google.toml`
- `providers/vertex_llama_sample.toml` - Sample for `providers/vertex_llama.toml`
- `providers/wmt_sample.toml` - Sample for `providers/wmt.toml`
- `providers/wmtghco_sample.toml` - Sample for `providers/wmtghco.toml`
- `providers/zhipu_sample.toml` - Sample for `providers/zhipu.toml`

## Directory Structure

```
~/Library/Application Support/lc/
├── config.toml              # Main configuration
├── other_config.toml        # Other main configs
└── providers/               # Provider-specific configs
    ├── gemini.toml          # Google Gemini provider
    ├── vertex_google.toml   # Vertex AI Google provider
    ├── vertex_llama.toml    # Vertex AI Llama provider
    ├── cohere.toml          # Cohere provider
    ├── meta.toml            # Meta/Llama provider
    └── other_providers.toml # Other providers
```

## Authentication Headers

Providers can use custom authentication headers instead of the standard `Authorization: Bearer` format. This is useful for providers like Google Gemini that require specific header names.

### How It Works

In a provider configuration file, you can specify custom headers with the `${api_key}` placeholder:

```toml
# Example: gemini.toml
[headers]
x-goog-api-key = "${api_key}"
```

The `${api_key}` placeholder will be automatically replaced with the actual API key from `keys.toml` when the provider is used.

### Examples

**Standard Authentication (OpenAI):**
```toml
# No custom headers needed - uses Authorization: Bearer automatically
endpoint = "https://api.openai.com/v1"
```

**Custom Header (Gemini):**
```toml
endpoint = "https://generativelanguage.googleapis.com"
[headers]
x-goog-api-key = "${api_key}"
```

**Multiple Custom Headers:**
```toml
[headers]
x-api-key = "${api_key}"
x-api-version = "2024-01"
```

See `providers/gemini_sample.toml` for a complete example.

## Security Note

⚠️ **Never commit actual API keys or credentials to version control!**

These sample files have all sensitive values masked with placeholders. Always keep your actual configuration files private and secure.

# LC Template System Documentation

The LC template system provides a flexible way to transform requests and responses for different LLM providers without modifying the source code. It uses the Tera templating engine (Jinja2-like syntax) to define custom transformations.

## Overview

The template system allows you to:
- Transform request payloads before sending to providers
- Parse and extract data from provider responses
- Handle provider-specific formats and requirements
- Support different endpoints (chat, images, embeddings, models)
- Apply templates based on model patterns (regex matching)

## Configuration Structure

Templates are configured per provider and per endpoint in your `config.toml`:

```toml
[providers.provider_name.endpoint_name]
# Default template for all models on this endpoint
[providers.provider_name.endpoint_name.template]
request = "..."
response = "..."
stream_response = "..."

# Model-specific templates (exact match)
[providers.provider_name.endpoint_name.model_templates.model_name]
request = "..."
response = "..."

# Pattern-based templates (regex match)
[providers.provider_name.endpoint_name.model_template_patterns."pattern"]
request = "..."
response = "..."
```

### Endpoint Names

The following endpoint names are supported:
- `chat` - For chat completions
- `images` - For image generation
- `embeddings` - For text embeddings
- `models` - For listing models

### Template Priority

Templates are applied in the following order:
1. **Exact model match** - `model_templates.model_name`
2. **Pattern match** - `model_template_patterns.pattern` (first match wins)
3. **Default template** - `template`
4. **No template** - Falls back to default behavior

## Template Syntax

Templates use Tera syntax (similar to Jinja2):

### Variables
- `{{ variable }}` - Output a variable
- `{{ variable | filter }}` - Apply a filter to a variable

### Conditionals
```jinja
{% if condition %}
  ...
{% elif other_condition %}
  ...
{% else %}
  ...
{% endif %}
```

### Loops
```jinja
{% for item in items %}
  {{ item }}
{% endfor %}
```

## Available Variables

### Request Templates

- `model` - The model name
- `messages` - Array of message objects
- `max_tokens` - Maximum tokens to generate
- `temperature` - Temperature parameter
- `stream` - Whether streaming is enabled
- `tools` - Array of tool definitions
- `system_prompt` - Extracted system message content (if present)
- Provider-specific variables from `vars` section

### Response Templates

All fields from the provider's JSON response are available as variables.

## Built-in Filters

### `json`
Converts a value to JSON string:
```jinja
"messages": {{ messages | json }}
```

### `gemini_role`
Converts OpenAI roles to Gemini format:
- `assistant` → `model`
- `system` → `user`

### `system_to_user_role`
Converts system roles to user roles (for providers that don't support system roles):
- `system` → `user`
- Other roles remain unchanged

This filter is useful for providers like Bedrock that don't support system messages directly.

### `default`
Provides a default value if the variable is null or empty:
```jinja
{{ content | default(value="No content") }}
```

### `select_tool_calls`
Filters array items that contain tool calls:
```jinja
{{ parts | select_tool_calls(key="functionCall") }}
```

## Examples

### GPT-5 Models with Different Token Parameter

```toml
[providers.openai.chat.model_template_patterns."gpt-5.*"]
request = '''
{
  "model": "{{ model }}",
  "messages": {{ messages | json }},
  {% if max_tokens %}"max_completion_tokens": {{ max_tokens }}{% endif %}
}
'''
```

### Gemini Format Transformation

```toml
[providers.gemini.chat.template]
request = '''
{
  "contents": [
    {% for msg in messages %}
    {
      "role": "{{ msg.role | gemini_role }}",
      "parts": [{"text": "{{ msg.content }}"}]
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ]
}
'''
```

### Custom Response Parsing

```toml
[providers.custom.chat.template]
response = '''
{
  "content": "{{ result.choices[0].text }}",
  "usage": {
    "tokens": {{ result.metadata.token_count }}
  }
}
'''
```

### Different Templates for Different Endpoints

```toml
# Chat endpoint
[providers.openai.chat.template]
request = '''
{
  "model": "{{ model }}",
  "messages": {{ messages | json }}
}
'''

# Images endpoint
[providers.openai.images.template]
request = '''
{
  "prompt": "{{ prompt }}",
  "model": "{{ model }}",
  "size": "{{ size | default(value='1024x1024') }}"
}
'''

# Embeddings endpoint
[providers.openai.embeddings.template]
request = '''
{
  "model": "{{ model }}",
  "input": {{ input | json }}
}
'''
```

### Model-Specific Templates

```toml
# Default for all models
[providers.vertex.chat.template]
request = '''
{
  "model": "{{ model }}",
  "messages": {{ messages | json }}
}
'''

# Specific template for Gemini models
[providers.vertex.chat.model_template_patterns."gemini-.*"]
request = '''
{
  "contents": [
    {% for msg in messages %}
    {
      "role": "{{ msg.role | gemini_role }}",
      "parts": [{"text": "{{ msg.content }}"}]
    }
    {% endfor %}
  ]
}
'''

# Specific template for Llama models
[providers.vertex.chat.model_template_patterns."llama-.*"]
request = '''
{
  "model": "{{ model }}",
  "messages": {{ messages | json }},
  "max_tokens": {{ max_tokens }}
}
'''
```

## Advanced Usage

### Handling Multimodal Content

```toml
[providers.openai.chat.template]
request = '''
{
  "messages": [
    {% for msg in messages %}
    {
      "role": "{{ msg.role }}",
      {% if msg.images %}
      "content": [
        {% if msg.content %}
        {"type": "text", "text": "{{ msg.content }}"},
        {% endif %}
        {% for img in msg.images %}
        {
          "type": "image_url",
          "image_url": {"url": "{{ img.url }}"}
        }{% if not loop.last %},{% endif %}
        {% endfor %}
      ]
      {% else %}
      "content": "{{ msg.content }}"
      {% endif %}
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ]
}
'''
```

### Conditional Field Inclusion

```toml
[providers.custom.chat.template]
request = '''
{
  "prompt": "{{ messages[-1].content }}",
  {% if temperature %}"temperature": {{ temperature }},{% endif %}
  {% if max_tokens %}"max_length": {{ max_tokens }},{% endif %}
  {% if tools %}
  "functions": [
    {% for tool in tools %}
    {
      "name": "{{ tool.function.name }}",
      "description": "{{ tool.function.description }}",
      "parameters": {{ tool.function.parameters | json }}
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ],
  {% endif %}
  "stream": {{ stream | default(value=false) }}
}
'''
```

### Complex Response Extraction

```toml
[providers.custom.chat.template]
response = '''
{
  {% if response.type == "text" %}
  "content": "{{ response.text }}"
  {% elif response.type == "tool_call" %}
  "tool_calls": [
    {% for call in response.calls %}
    {
      "id": "{{ call.id }}",
      "type": "function",
      "function": {
        "name": "{{ call.function }}",
        "arguments": {{ call.args | json }}
      }
    }{% if not loop.last %},{% endif %}
    {% endfor %}
  ]
  {% endif %}
}
'''
```

## Debugging Templates

1. **Enable debug logging**: Set `LC_DEBUG=1` to see template processing details
2. **Test incrementally**: Start with simple templates and add complexity
3. **Validate JSON**: Ensure your templates produce valid JSON
4. **Check variable names**: Use the exact field names from the provider's API

## Best Practices

1. **Start with minimal templates**: Only include fields you need to transform
2. **Use conditionals for optional fields**: Prevent invalid JSON from missing values
3. **Test with different models**: Ensure patterns match correctly
4. **Document your templates**: Add comments explaining transformations
5. **Keep templates readable**: Use proper indentation and formatting

## Migration Guide

If you're migrating from the old template system:

### Old Format
```toml
[providers.openai.model_template_patterns."gpt-5.*"]
request = "..."
```

### New Format
```toml
[providers.openai.chat.model_template_patterns."gpt-5.*"]
request = "..."
```

The key difference is that templates are now organized by endpoint (`chat`, `images`, etc.) to support different transformations for different API endpoints.
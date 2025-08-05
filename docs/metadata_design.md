# Providers
Design metadata for each provider to ensure consistent structure and easy access to model information.

## Identifying the models path
Since each provider has its own API response structure, we need to identify the path to the models in the response. This will allow us to extract model information consistently.

lc models path (should list all the jq paths for each provider)
```bash
lc models path
lc mo p

output:
paths:
   - .data[]
   - .models[]
```

let users add paths as needed
```bash
lc models path add <path>
lc mo p a <path>
```
and also given option to remove paths
```bash
lc models path delete <path>
lc mo p d <path>
```
Using above paths, lc should scan for the models
One we got the models, next step is to extract metadata from the models.

## Metadata Extraction
Since each provider has different metadata fields, we need to define a consistent way to extract and store metadata for each model. By default lc should extract the following metadata fields:
- `ctx (context length)`
- `out (output tokens)`
- `input_price`
- `output_price`
- `tools`
- `vision`
- `reasoning`
- `audio`

and above metadata should be extracted by configurable jq type paths.
Users can customize or add these paths based on their needs.

lc models tags (should list all the tags)

By default, 
lc mo t l
should display all tags and their rules.
These rules exist in ~/Library/Application\ Support/lc/tags.toml
when models are scanned, lc should apply these rules to the models and add tags accordingly.
Note: if there is "," in the rule, ex: `tool, model.capabilities[]`, it means search for the tool in the model.capabilities array and add the tag if found.
Example output:

```
Tags:
  - ctx 
    - model.context_length
    - model.context_window
    - model.max_context_length
    - model.max_model_len
    - model.inputTokenLimit
    - model.providers[].context_length
    - model.limits.max_input_tokens
    - model.top_provider.context_length
  - output
    - model.max_completion_tokens
    - model.max_output_tokens
    - model.outputTokenLimit
    - model.limits.max_output_tokens
    - model.top_provider.max_completion_tokens
    - model.max_tokens
  - input_price
    - model.price.input.usd
    - model.pricing.input
    - model.pricing.prompt
    - model.providers[].pricing.input
    - model.input_price
  - output_price
    - model.price.output.usd
    - model.pricing.output
    - model.pricing.completion
    - model.output_price
  - tools:
    - model.supports_tools
    - model.capabilities.function_calling 
    - tool, model.capabilities[]
    - tool, model.features[]
    - tool, model.supported_parameters[]
  - vision:
    - image, model.id
    - flux, model.id
    - vision, model.id
    - model.supports_image_input
    - model.supports_vision 
    - model.capabilities.vision
    - image, model.capabilities[]
    - image, model.features[]
    - image, model.architecture.input_modalities[]
    - image, model.architecture.output_modalities[]
    - image, model.supported_input_modalities[]
    - image, model.supported_output_modalities[]
  - reasoning: 
    - reasoning, model.id
    - model.supports_reasoning
    - reasoning, model.capabilities[]
    - reasoning, model.features[]
    - reasoning, model.supported_parameters[]
  - audio:
    - audio, model.id
    - audio, model.capabilities[]
    - audio, model.features[]
    - audio, model.architecture.input_modalities[]
    - audio, model.architecture.output_modalities[]
    - audio, model.supported_input_modalities[]
    - audio, model.supported_output_modalities[]
  - embed
    - embed, model.id
    - embed, model.capabilities[]
    - embed, model.features[]
    - embed, model.supported_parameters[]
```

User can add tags with rules
lc models tags add <tag> <rule>
example:
lc mo t add tools "tool, model.capabilities[]"
above should look for "tool" in model.capabilities and add tag "tools" to the model

## Add models
Users can add models with specific metadata:
Since some providers may not have all the metadata fields, users can add models with specific metadata.

let users give option to add the models
```bash
lc provider m <provider> add <model>
lc p m <provider> a <model>
```
user can also add tags to the models
```bash
lc provider m <provider> add <model> --tag <tag1,tag2,...>
lc p m <provider> a <model> -t <tag1,tag2,...

Example:
lc provider m openai add gpt-4 --tag ctx=120k,out=1024,tools,vision
```

## only for huggingface (hf)
Only for huggingface, since it has multiple providers.
whie listing models for example like below
```
    {
      "created": 1753195958,
      "id": "Qwen/Qwen3-Coder-480B-A35B-Instruct",
      "object": "model",
      "owned_by": "Qwen",
      "providers": [
        {
          "context_length": 262144,
          "pricing": {
            "input": 0.95,
            "output": 5
          },
          "provider": "novita",
          "status": "live",
          "supports_structured_output": false,
          "supports_tools": false
        },
        {
          "provider": "fireworks-ai",
          "status": "live",
          "supports_structured_output": false,
          "supports_tools": true
        },
        {
          "provider": "hyperbolic",
          "status": "live",
          "supports_structured_output": false,
          "supports_tools": false
        },
        {
          "context_length": 262144,
          "pricing": {
            "input": 2,
            "output": 2
          },
          "provider": "together",
          "status": "live",
          "supports_structured_output": true,
          "supports_tools": false
        },
        {
          "provider": "cerebras",
          "status": "live",
          "supports_structured_output": false,
          "supports_tools": true
        }
      ]
    },
```
instead of one like this "Qwen/Qwen3-Coder-480B-A35B-Instruct"
it would be like below for each provider
Qwen/Qwen3-Coder-480B-A35B-Instruct:novita
Qwen/Qwen3-Coder-480B-A35B-Instruct:fireworks-ai
Qwen/Qwen3-Coder-480B-A35B-Instruct:hyperbolic
Qwen/Qwen3-Coder-480B-A35B-Instruct:together
Qwen/Qwen3-Coder-480B-A35B-Instruct:cerebras

and then metadata should be extracted based on the tags
metadata for each provider:
ctx = .providers[].context_length
supports_tools = if .providers[].supports_tools
input_price = .providers[].pricing.input
output_price = .providers[].pricing.output

## models filtering
Users can filter models based on tags:
```bash
lc models filter --tag tools,vision
lc mo f --tag tools,vision
``` 

since we are giving above options we can remove below
      --tools                        Filter models that support tools/function calling
      --reasoning                    Filter models that support reasoning
      --vision                       Filter models that support vision
      --audio                        Filter models that support audio
      --code                         Filter models that support code generation


# Models Data Collection Guide

This guide explains how to use the `dump_models.rs` script to collect fresh model data from all configured providers. This is useful when providers update their model offerings, change API responses, or when debugging model metadata issues.

## Overview

The `dump_models.rs` script fetches raw model data from each configured provider's `/models` endpoint and saves it as JSON files in the `models/` directory. This data is then used by the LC tool to extract model metadata like capabilities, context limits, and pricing information.

## Prerequisites

### 1. Provider Configuration
Ensure you have providers configured in your LC configuration file (`~/.config/lc/config.toml`) with valid API keys:

```toml
[providers.openrouter]
endpoint = "https://openrouter.ai/api/v1"
api_key = "sk-or-v1-..."
models_path = "/models"
chat_path = "/chat/completions"

[providers.venice]
endpoint = "https://api.venice.ai/api/v1"
api_key = "VEN-..."
models_path = "/models"
chat_path = "/chat/completions"

# Add other providers as needed
```

### 2. Dependencies
The script uses the existing LC codebase modules, so ensure your project compiles:

```bash
cargo check
```

## Running the Script

### Step 1: Execute the Script
From the project root directory, run:

```bash
cargo run --bin dump_models
```

Or compile and run directly:

```bash
rustc dump_models.rs --extern anyhow --extern reqwest --extern serde_json --extern tokio -L dependency=target/debug/deps
./dump_models
```

### Step 2: Monitor Output
The script will display progress for each provider:

```
🔍 Dumping /models for each provider...
📡 Fetching models from openrouter...
✅ Saved openrouter models data to models/openrouter.json
📡 Fetching models from venice...
✅ Saved venice models data to models/venice.json
⚠️  Skipping mistral (no API key)

📊 Summary:
   Total providers: 3
   Successful dumps: 2
   Models data saved to: ./models/

🎉 Model data collection complete!
   Next step: Analyze the JSON files to extract metadata patterns
```

### Step 3: Verify Output
Check that JSON files were created in the `models/` directory:

```bash
ls -la models/
# Should show files like:
# openrouter.json
# venice.json
# requesty.json
# etc.
```

## Output Format

Each JSON file contains the raw response from the provider's `/models` endpoint, formatted for readability:

### Example: OpenRouter Response
```json
{
  "data": [
    {
      "id": "anthropic/claude-3.5-sonnet",
      "name": "Claude 3.5 Sonnet",
      "description": "Claude 3.5 Sonnet by Anthropic",
      "context_length": 200000,
      "pricing": {
        "prompt": "0.000003",
        "completion": "0.000015"
      },
      "top_provider": {
        "context_length": 200000,
        "max_completion_tokens": 8192
      }
    }
  ]
}
```

### Example: Venice Response
```json
{
  "data": [
    {
      "id": "llama-3.3-70b",
      "object": "model",
      "created": 1234567890,
      "owned_by": "meta",
      "name": "Llama 3.3 70B",
      "description": "Meta's Llama 3.3 70B model",
      "context_window": 65536,
      "supported_parameters": ["tools", "temperature", "top_p"]
    }
  ]
}
```

## When to Use This Script

### 1. Provider Updates
- When a provider adds new models
- When model capabilities change (e.g., new tools support)
- When pricing information is updated
- When context limits are modified

### 2. Debugging
- When model metadata extraction seems incorrect
- When new providers are added to the system
- When investigating API response format changes

### 3. Development
- When implementing support for new providers
- When testing metadata extraction logic
- When validating provider API compatibility

## Troubleshooting

### Common Issues

#### 1. Missing API Keys
```
⚠️  Skipping provider_name (no API key)
```
**Solution:** Add the API key to your configuration file.

#### 2. Network Timeouts
```
❌ Failed to fetch models from provider_name: timeout
```
**Solution:** Check internet connection and provider API status.

#### 3. Authentication Errors
```
❌ Failed to fetch models from provider_name: API request failed with status 401
```
**Solution:** Verify API key is valid and has proper permissions.

#### 4. Rate Limiting
```
❌ Failed to fetch models from provider_name: API request failed with status 429
```
**Solution:** Wait and retry, or check provider rate limits.

### Debugging Tips

#### 1. Check Raw Response
If a provider fails, you can manually test the endpoint:

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
     -H "Content-Type: application/json" \
     "https://api.provider.com/v1/models"
```

#### 2. Validate JSON Output
Ensure the generated JSON files are valid:

```bash
jq . models/provider_name.json
```

#### 3. Compare with Previous Dumps
Keep previous dumps for comparison:

```bash
# Before running script
cp -r models models_backup_$(date +%Y%m%d)

# After running script
diff -r models_backup_20240125 models
```

## Integration with LC Tool

After collecting fresh model data, the LC tool will automatically use the updated information for:

### 1. Model Listings
```bash
lc p m openrouter --tools  # Shows models with tools support
```

### 2. Metadata Display
```bash
lc co  # Shows current model with updated metadata
```

### 3. Context Validation
The token counting system uses the updated context limits for validation.

### 4. Cost Estimation
Updated pricing information is used for real-time cost calculations.

## File Structure

```
project_root/
├── dump_models.rs          # The collection script
├── models/                 # Generated model data
│   ├── openrouter.json    # OpenRouter models
│   ├── venice.json        # Venice AI models
│   ├── requesty.json      # Requesty models
│   └── mistral.json       # Mistral models
└── src/
    ├── models_cache.rs    # Metadata extraction logic
    └── ...
```

## Best Practices

### 1. Regular Updates
- Run monthly to catch new models and pricing changes
- Run before major releases to ensure accurate metadata
- Run when troubleshooting model-related issues

### 2. Backup Previous Data
- Keep backups of previous dumps for comparison
- Use version control to track changes over time
- Document significant changes in model offerings

### 3. Validation
- Always verify the generated JSON files are valid
- Test model listings after updating data
- Check that metadata extraction still works correctly

### 4. Provider-Specific Notes
- **OpenRouter**: Updates frequently with new models
- **Venice**: Stable but may add new capabilities
- **Requesty**: Check for pricing changes
- **Mistral**: Monitor for new model releases

## Security Considerations

### 1. API Key Protection
- Never commit API keys to version control
- Use environment variables or secure config files
- Rotate keys regularly

### 2. Data Sensitivity
- Model data is generally public information
- Be cautious with provider-specific metadata
- Follow each provider's terms of service

## Future Enhancements

The script could be enhanced with:
- Automatic scheduling (cron job)
- Diff reporting between runs
- Integration with CI/CD pipelines
- Provider-specific customization options
- Incremental updates instead of full dumps

---

For questions or issues with this script, refer to the main project documentation or open an issue in the project repository.
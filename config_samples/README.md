# Configuration Samples

This directory contains sanitized sample configuration files for the `lc` tool.

## Usage

1. Copy the relevant sample file(s) to your `~/Library/Application Support/lc/` directory
2. Rename them by removing the `_sample` suffix
3. Replace all `<your-*>` placeholders with your actual API keys and credentials

## Files

- `config_sample.toml` - Sample for `config.toml`
- `mcp_sample.toml` - Sample for `mcp.toml`
- `model_paths_sample.toml` - Sample for `model_paths.toml`
- `search_config_sample.toml` - Sample for `search_config.toml`
- `sync_sample.toml` - Sample for `sync.toml`
- `tags_sample.toml` - Sample for `tags.toml`
- `webchatproxy_sample.toml` - Sample for `webchatproxy.toml`
- `webchatproxy_daemons_sample.toml` - Sample for `webchatproxy_daemons.toml`

## Security Note

⚠️ **Never commit actual API keys or credentials to version control!**

These sample files have all sensitive values masked with placeholders. Always keep your actual configuration files private and secure.

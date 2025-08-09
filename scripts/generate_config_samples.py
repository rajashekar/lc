#!/usr/bin/env python3
"""
Script to generate sanitized config samples from TOML files in ~/Library/Application Support/lc/
This script masks sensitive information like API keys, tokens, and credentials.
"""

import os
import re
import shutil
import sys
from pathlib import Path
import tomllib
import json

def mask_sensitive_value(key, value):
    """
    Mask sensitive values based on key patterns.
    Returns the masked value or original value if not sensitive.
    """
    sensitive_patterns = [
        r'.*api_key.*',
        r'.*token.*',
        r'.*auth.*',
        r'.*key.*',
        r'.*secret.*',
        r'.*password.*',
        r'.*credential.*',
        r'.*private_key.*',
        r'.*access_key.*',
        r'.*subscription.*',
        r'.*account_id.*',
        r'authorization',
        r'x-api-key',
        r'x-subscription-token',
    ]
    
    key_lower = key.lower()
    
    # Check if key matches any sensitive pattern
    for pattern in sensitive_patterns:
        if re.match(pattern, key_lower):
            if isinstance(value, str):
                if value.strip() == "":
                    return value  # Keep empty strings as-is
                # Special handling for JSON strings (like Google service account keys)
                if value.strip().startswith('{') and 'private_key' in value:
                    return mask_json_credentials(value)
                # Special handling for JWT tokens
                if '.' in value and len(value.split('.')) >= 3:
                    return "<your-jwt-token>"
                # Special handling for different token formats
                if value.startswith('sk-'):
                    return "<your-api-key>"
                elif value.startswith('gho_') or value.startswith('ghu_'):
                    return "<your-github-token>"
                elif value.startswith('xai-'):
                    return "<your-xai-api-key>"
                elif value.startswith('pplx-'):
                    return "<your-perplexity-api-key>"
                elif value.startswith('gsk_'):
                    return "<your-groq-api-key>"
                elif value.startswith('csk-'):
                    return "<your-cerebras-api-key>"
                elif value.startswith('fw_'):
                    return "<your-fireworks-api-key>"
                elif value.startswith('nvapi-'):
                    return "<your-nvidia-api-key>"
                elif value.startswith('hf_'):
                    return "<your-huggingface-token>"
                elif value.startswith('cpk_'):
                    return "<your-chutes-api-key>"
                elif value.startswith('CHK-'):
                    return "<your-chub-api-key>"
                elif value.startswith('ns_'):
                    return "<your-nscale-api-key>"
                elif value.startswith('LLM|'):
                    return "<your-meta-api-key>"
                elif value.startswith('AKIA'):
                    return "<your-aws-access-key-id>"
                elif value.startswith('ya29.'):
                    return "<your-google-oauth-token>"
                elif value.startswith('tid='):
                    return "<your-copilot-token>"
                elif value.startswith('eyJ'):  # JWT tokens
                    return "<your-jwt-token>"
                elif value.startswith('-----BEGIN'):
                    return "<your-private-key>"
                elif key_lower == 'account_id':
                    return "<your-account-id>"
                else:
                    return "<your-api-key>"
            return value
    
    return value

def mask_json_credentials(json_str):
    """
    Mask credentials within JSON strings (like Google service account keys).
    """
    try:
        import json
        data = json.loads(json_str)
        
        # Mask sensitive fields in the JSON
        sensitive_json_fields = [
            'private_key', 'private_key_id', 'client_email', 'client_id',
            'project_id', 'auth_uri', 'token_uri', 'client_x509_cert_url'
        ]
        
        for field in sensitive_json_fields:
            if field in data:
                if field == 'private_key':
                    data[field] = "<your-private-key>"
                elif field == 'project_id':
                    data[field] = "<your-project-id>"
                elif field == 'client_email':
                    data[field] = "<your-service-account-email>"
                elif field == 'client_id':
                    data[field] = "<your-client-id>"
                else:
                    data[field] = f"<your-{field.replace('_', '-')}>"
        
        return json.dumps(data, indent=2)
    except:
        return "<your-service-account-json>"

def mask_toml_data(data):
    """
    Recursively mask sensitive data in TOML structure.
    """
    if isinstance(data, dict):
        masked_data = {}
        for key, value in data.items():
            if isinstance(value, dict):
                masked_data[key] = mask_toml_data(value)
            elif isinstance(value, list):
                masked_data[key] = [mask_toml_data(item) if isinstance(item, dict) else mask_sensitive_value(key, item) for item in value]
            else:
                masked_data[key] = mask_sensitive_value(key, value)
        return masked_data
    elif isinstance(data, list):
        return [mask_toml_data(item) if isinstance(item, dict) else item for item in data]
    else:
        return data

def get_comment_for_key(key, value, parent_key=""):
    """
    Generate helpful comments for TOML keys based on their purpose.
    """
    key_lower = key.lower()
    parent_lower = parent_key.lower()
    
    # Provider-specific comments
    if parent_lower == "providers":
        return f"# Configuration for {key} AI provider"
    
    # Common configuration comments
    comments = {
        "default_provider": "# Default AI provider to use when none is specified",
        "default_model": "# Default model to use with the default provider",
        "max_tokens": "# Maximum number of tokens to generate in responses",
        "temperature": "# Controls randomness in responses (0.0 = deterministic, 1.0 = very random)",
        "endpoint": "# API endpoint URL for this provider",
        "api_key": "# API key for authentication - replace with your actual key",
        "models": "# List of available models (auto-populated by the tool)",
        "models_path": "# API path to fetch available models",
        "chat_path": "# API path for chat completions",
        "images_path": "# API path for image generation",
        "embeddings_path": "# API path for text embeddings",
        "token_url": "# URL to refresh authentication tokens",
        "auth_type": "# Authentication method used by this provider",
        "bucket_name": "# S3 bucket name for sync storage",
        "region": "# AWS region for S3 bucket",
        "access_key_id": "# AWS access key ID - replace with your actual key",
        "secret_access_key": "# AWS secret access key - replace with your actual key",
        "auth_token": "# Authentication token - replace with your actual token",
        "server_type": "# MCP server connection type (Stdio or Sse)",
        "command_or_url": "# Command to start the MCP server or SSE endpoint URL",
        "provider_type": "# Search provider implementation type",
        "url": "# Search API endpoint URL",
    }
    
    # Environment variable comments
    if parent_lower == "env" and "_api_key" in key_lower:
        return f"# API key environment variable for {key.replace('_API_KEY', '').lower()}"
    elif parent_lower == "env" and "_token" in key_lower:
        return f"# Token environment variable for {key.replace('_TOKEN', '').lower()}"
    
    # Header comments
    if parent_lower == "headers":
        if "api" in key_lower or "key" in key_lower:
            return "# API key header - replace with your actual key"
        elif "token" in key_lower:
            return "# Token header - replace with your actual token"
        elif "authorization" in key_lower:
            return "# Authorization header - replace with your actual credentials"
        else:
            return f"# HTTP header: {key}"
    
    # Cached token comments
    if parent_lower == "cached_token":
        if key_lower == "token":
            return "# Cached authentication token (auto-refreshed)"
        elif key_lower == "expires_at":
            return "# Token expiration timestamp"
    
    # Aliases and templates
    if parent_lower == "aliases":
        return f"# Shortcut alias: use '{key}' instead of '{value}'"
    elif parent_lower == "templates":
        return f"# Prompt template for {key} tasks"
    
    return comments.get(key_lower, "")

def write_toml(data, file_handle, indent=0, parent_key=""):
    """
    Simple TOML writer function with comments.
    """
    indent_str = "  " * indent
    
    # Write top-level key-value pairs first
    for key, value in data.items():
        if not isinstance(value, dict):
            # Add comment if available
            comment = get_comment_for_key(key, value, parent_key)
            if comment:
                file_handle.write(f'{indent_str}{comment}\n')
            
            if isinstance(value, str):
                # Handle multiline strings
                if '\n' in value:
                    file_handle.write(f'{indent_str}{key} = """\n{value}\n"""\n')
                else:
                    # Escape quotes in strings
                    escaped_value = value.replace('"', '\\"')
                    file_handle.write(f'{indent_str}{key} = "{escaped_value}"\n')
            elif isinstance(value, bool):
                file_handle.write(f'{indent_str}{key} = {str(value).lower()}\n')
            elif isinstance(value, (int, float)):
                file_handle.write(f'{indent_str}{key} = {value}\n')
            elif isinstance(value, list):
                if all(isinstance(item, str) for item in value):
                    formatted_list = ', '.join(f'"{item}"' for item in value)
                    file_handle.write(f'{indent_str}{key} = [{formatted_list}]\n')
                else:
                    file_handle.write(f'{indent_str}{key} = {value}\n')
            
            # Add spacing after key-value pairs
            if comment:
                file_handle.write('\n')
    
    # Write sections (tables)
    for key, value in data.items():
        if isinstance(value, dict):
            # Add comment for section
            section_comment = get_comment_for_key(key, value, parent_key)
            if section_comment:
                file_handle.write(f'{indent_str}{section_comment}\n')
            
            if parent_key:
                file_handle.write(f'\n{indent_str}[{parent_key}.{key}]\n')
            else:
                file_handle.write(f'\n{indent_str}[{key}]\n')
            write_toml(value, file_handle, indent, key)

def process_toml_file(input_path, output_path):
    """
    Process a single TOML file and create a masked version.
    """
    try:
        print(f"Processing {input_path.name}...")
        
        # Read the original TOML file
        with open(input_path, 'rb') as f:
            data = tomllib.load(f)
        
        # Mask sensitive data
        masked_data = mask_toml_data(data)
        
        # Write the masked version
        with open(output_path, 'w', encoding='utf-8') as f:
            # Add header comment
            f.write(f"# Sample configuration file for {input_path.name}\n")
            f.write("# This is a sanitized version with sensitive values masked\n")
            f.write("# Replace <your-*> placeholders with your actual values\n")
            f.write("# Comments below explain what each setting does\n\n")
            
            write_toml(masked_data, f)
        
        print(f"✓ Created sample: {output_path.name}")
        
    except Exception as e:
        print(f"✗ Error processing {input_path.name}: {e}")

def main():
    """
    Main function to process all TOML files.
    """
    # Define paths
    lc_config_dir = Path.home() / "Library" / "Application Support" / "lc"
    providers_dir = lc_config_dir / "providers"
    # Create config_samples in the git repo directory
    git_repo_dir = Path("/Users/rchint1/Documents/workspace/lc")
    config_samples_dir = git_repo_dir / "config_samples"
    providers_samples_dir = config_samples_dir / "providers"
    
    # Ensure directories exist
    config_samples_dir.mkdir(exist_ok=True)
    providers_samples_dir.mkdir(exist_ok=True)
    
    print(f"Processing TOML files from: {lc_config_dir}")
    print(f"Processing provider TOML files from: {providers_dir}")
    print(f"Output directory: {config_samples_dir}")
    print("-" * 50)
    
    # Find all TOML files in main directory
    main_toml_files = list(lc_config_dir.glob("*.toml"))
    
    # Find all TOML files in providers directory
    provider_toml_files = []
    if providers_dir.exists():
        provider_toml_files = list(providers_dir.glob("*.toml"))
    
    all_files_count = len(main_toml_files) + len(provider_toml_files)
    
    if all_files_count == 0:
        print("No TOML files found!")
        return
    
    # Process main TOML files
    print("Processing main configuration files:")
    for toml_file in main_toml_files:
        output_file = config_samples_dir / f"{toml_file.stem}_sample.toml"
        process_toml_file(toml_file, output_file)
    
    # Process provider TOML files
    if provider_toml_files:
        print("\nProcessing provider configuration files:")
        for toml_file in provider_toml_files:
            output_file = providers_samples_dir / f"{toml_file.stem}_sample.toml"
            process_toml_file(toml_file, output_file)
    
    print("-" * 50)
    print(f"✓ Processed {len(main_toml_files)} main TOML files")
    print(f"✓ Processed {len(provider_toml_files)} provider TOML files")
    print(f"✓ Config samples created in: {config_samples_dir}")
    print(f"✓ Provider samples created in: {providers_samples_dir}")
    
    # Create a README file
    readme_path = config_samples_dir / "README.md"
    with open(readme_path, 'w', encoding='utf-8') as f:
        f.write("# Configuration Samples\n\n")
        f.write("This directory contains sanitized sample configuration files for the `lc` tool.\n\n")
        f.write("## Usage\n\n")
        f.write("1. Copy the relevant sample file(s) to your `~/Library/Application Support/lc/` directory\n")
        f.write("2. For provider files, copy them to `~/Library/Application Support/lc/providers/`\n")
        f.write("3. Rename them by removing the `_sample` suffix\n")
        f.write("4. Replace all `<your-*>` placeholders with your actual API keys and credentials\n\n")
        f.write("## Files\n\n")
        f.write("### Main Configuration Files\n\n")
        
        for toml_file in sorted(main_toml_files):
            sample_name = f"{toml_file.stem}_sample.toml"
            f.write(f"- `{sample_name}` - Sample for `{toml_file.name}`\n")
        
        if provider_toml_files:
            f.write("\n### Provider Configuration Files\n\n")
            for toml_file in sorted(provider_toml_files):
                sample_name = f"providers/{toml_file.stem}_sample.toml"
                f.write(f"- `{sample_name}` - Sample for `providers/{toml_file.name}`\n")
        
        f.write("\n## Directory Structure\n\n")
        f.write("```\n")
        f.write("~/Library/Application Support/lc/\n")
        f.write("├── config.toml              # Main configuration\n")
        f.write("├── other_config.toml        # Other main configs\n")
        f.write("└── providers/               # Provider-specific configs\n")
        f.write("    ├── gemini.toml          # Google Gemini provider\n")
        f.write("    ├── vertex_google.toml   # Vertex AI Google provider\n")
        f.write("    ├── vertex_llama.toml    # Vertex AI Llama provider\n")
        f.write("    ├── cohere.toml          # Cohere provider\n")
        f.write("    ├── meta.toml            # Meta/Llama provider\n")
        f.write("    └── other_providers.toml # Other providers\n")
        f.write("```\n\n")
        
        f.write("## Security Note\n\n")
        f.write("⚠️ **Never commit actual API keys or credentials to version control!**\n\n")
        f.write("These sample files have all sensitive values masked with placeholders. ")
        f.write("Always keep your actual configuration files private and secure.\n")
    
    print(f"✓ Created README.md with usage instructions")

if __name__ == "__main__":
    main()
# Sync Feature Documentation

The `lc` CLI tool includes a comprehensive sync feature that allows you to synchronize your configuration files to and from cloud providers with optional AES256 encryption.

## Overview

The sync feature provides:
- **Cloud Provider Support**: Currently supports Amazon S3 and S3-compatible services
- **Encryption**: Optional AES256-GCM encryption for secure storage
- **Configuration Management**: Store provider credentials locally for easy reuse
- **Cross-Platform**: Works on macOS, Linux, and Windows
- **Multiple Aliases**: Short commands for faster usage

## Quick Start

### 1. List Supported Providers
```bash
lc sync providers
# or use alias
lc sy p
```

### 2. Configure S3 Provider
```bash
lc sync configure s3 setup
# or use aliases
lc sy c s3 s
```

### 3. Sync to Cloud
```bash
# Basic sync
lc sync to s3

# With encryption
lc sync to s3 --encrypted

# Using aliases
lc sy to s3 -e
```

### 4. Sync from Cloud
```bash
# Basic sync
lc sync from s3

# With decryption
lc sync from s3 --encrypted

# Using aliases
lc sy from s3 -e
```

## Commands Reference

### Main Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `lc sync` | `lc sy` | Main sync command |
| `lc sync providers` | `lc sy p` | List supported cloud providers |
| `lc sync configure <provider>` | `lc sy c <provider>` | Configure provider settings |
| `lc sync to <provider>` | `lc sy to <provider>` | Sync configuration to cloud |
| `lc sync from <provider>` | `lc sy from <provider>` | Sync configuration from cloud |

### Configure Subcommands

| Command | Alias | Description |
|---------|-------|-------------|
| `lc sync configure <provider> setup` | `lc sy c <provider> s` | Set up provider configuration |
| `lc sync configure <provider> show` | `lc sy c <provider> sh` | Show current configuration |
| `lc sync configure <provider> remove` | `lc sy c <provider> r` | Remove provider configuration |

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--encrypted` | `-e` | Enable encryption/decryption |

## Configuration Management

### Storing Provider Settings

The sync feature includes a configuration management system that stores provider settings in TOML files:

```bash
# Set up S3 configuration interactively
lc sync configure s3 setup
```

This will prompt you for:
- S3 bucket name
- AWS region (default: us-east-1)
- AWS Access Key ID
- AWS Secret Access Key (hidden input)
- Custom S3 endpoint URL (optional, for S3-compatible services)

### Configuration Storage Location

Configurations are stored in:
- **macOS**: `~/Library/Application Support/lc/sync.toml`
- **Linux**: `~/.config/lc/sync.toml`
- **Windows**: `%APPDATA%\lc\sync.toml`

### Viewing Configuration

```bash
# Show current S3 configuration (credentials are masked)
lc sync configure s3 show
```

### Removing Configuration

```bash
# Remove stored S3 configuration
lc sync configure s3 remove
```

## Cloud Providers

### Amazon S3

The primary supported provider with full feature support.

#### Configuration Methods (Priority Order)

1. **Stored Configuration** (Recommended)
   ```bash
   lc sync configure s3 setup
   ```

2. **Environment Variables**
   ```bash
   export LC_S3_BUCKET=your-bucket-name
   export LC_S3_REGION=us-east-1
   export AWS_ACCESS_KEY_ID=your-access-key
   export AWS_SECRET_ACCESS_KEY=your-secret-key
   export LC_S3_ENDPOINT=https://s3.amazonaws.com  # Optional
   ```

3. **Interactive Prompts** (Fallback)
   - If no stored config or environment variables are found
   - Prompts for credentials during sync operations

#### S3-Compatible Services

The sync feature supports S3-compatible services by setting a custom endpoint:

**Backblaze B2:**
```bash
# During setup, use endpoint: https://s3.us-west-004.backblazeb2.com
# (Replace region code with your actual region)
```

**Cloudflare R2:**
```bash
# During setup, use endpoint: https://your-account-id.r2.cloudflarestorage.com
# (Replace your-account-id with your actual Cloudflare account ID)
```

**Other S3-Compatible Services:**
- MinIO
- DigitalOcean Spaces
- Wasabi
- Any service implementing the S3 API

## Encryption

### AES256-GCM Encryption

The sync feature uses AES256-GCM encryption for secure storage:

- **Algorithm**: AES256-GCM (Galois/Counter Mode)
- **Key Derivation**: PBKDF2 with SHA-256
- **Salt**: Random 32-byte salt per file
- **Nonce**: Random 12-byte nonce per encryption
- **Authentication**: Built-in authentication tag

### Using Encryption

```bash
# Encrypt files before uploading
lc sync to s3 --encrypted

# Decrypt files after downloading
lc sync from s3 --encrypted
```

### Security Notes

- **Password**: You'll be prompted for a password during encryption/decryption
- **Key Storage**: Passwords are never stored; you must remember them
- **File Names**: Encrypted files get `.enc` extension in cloud storage
- **Metadata**: Original filenames and encryption status are stored as S3 metadata

## File Handling

### What Gets Synced

The sync feature automatically discovers and syncs all configuration files and databases in your lc configuration directory:

- **Configuration Files (.toml)**:
  - `config.toml` - Main lc configuration
  - `mcp.toml` - MCP server configurations
  - `sync.toml` - Sync provider configurations
  - Any other `.toml` files in the config directory

- **Provider Configuration Files (providers/*.toml)**:
  - `providers/openai.toml` - OpenAI provider configuration
  - `providers/bedrock.toml` - Amazon Bedrock provider configuration
  - `providers/gemini.toml` - Google Gemini provider configuration
  - All other provider-specific configuration files

- **Database Files**:
  - `logs.db` - Chat logs and session history database

### File Processing

1. **Upload Process**:
   - Reads all `.toml` configuration files and `logs.db` from config directory
   - Displays file types and sizes for transparency
   - Optionally encrypts content with AES256-GCM
   - Encodes as Base64 for safe S3 storage
   - Uploads with metadata (original name, file type, encryption status, file size, etc.)

2. **Download Process**:
   - Lists objects in S3 bucket under `llm_client_config/` prefix
   - Downloads and decodes from Base64
   - Displays file types and sizes during download
   - Optionally decrypts content
   - Restores files to local config directory

### Storage Structure

Files are stored in S3 with the following structure:
```
your-bucket/
└── llm_client_config/
    ├── config.toml (or config.toml.enc if encrypted)
    ├── mcp.toml (or mcp.toml.enc if encrypted)
    ├── sync.toml (or sync.toml.enc if encrypted)
    ├── providers/openai.toml (or providers/openai.toml.enc if encrypted)
    ├── providers/bedrock.toml (or providers/bedrock.toml.enc if encrypted)
    ├── providers/gemini.toml (or providers/gemini.toml.enc if encrypted)
    ├── [other provider files...]
    └── logs.db (or logs.db.enc if encrypted)
```

### Database Management

Since `logs.db` can grow large over time, the sync feature includes database management tools:

#### Purging Strategies

1. **Age-based Purging**: Remove entries older than N days
   ```bash
   lc logs purge --older-than-days 30
   ```

2. **Count-based Purging**: Keep only the most recent N entries
   ```bash
   lc logs purge --keep-recent 1000
   ```

3. **Size-based Purging**: Purge when database exceeds N MB
   ```bash
   lc logs purge --max-size-mb 50
   ```

4. **Combined Purging**: Use multiple strategies together
   ```bash
   lc logs purge --older-than-days 30 --keep-recent 1000 --max-size-mb 50
   ```

#### Recommended Purging Strategy

For optimal performance and reasonable sync times:
```bash
# Purge logs older than 30 days, keep max 1000 entries, limit to 50MB
lc logs purge --older-than-days 30 --keep-recent 1000 --max-size-mb 50
```

This strategy:
- Maintains recent context for chat continuity
- Prevents unlimited database growth
- Keeps sync operations fast
- Preserves important conversation history

## Examples

### Complete Workflow Example

```bash
# 1. Check available providers
lc sync providers

# 2. Set up S3 configuration
lc sync configure s3 setup
# Enter: bucket name, region, access key, secret key, endpoint (optional)

# 3. Verify configuration
lc sync configure s3 show

# 4. Sync to cloud with encryption
lc sync to s3 --encrypted
# Enter encryption password when prompted

# 5. Later, sync from cloud (e.g., on another machine)
lc sync from s3 --encrypted
# Enter the same decryption password
```

### Using Environment Variables

```bash
# Set environment variables
export LC_S3_BUCKET=my-lc-config-bucket
export LC_S3_REGION=us-west-2
export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY

# Sync without stored configuration
lc sync to s3 --encrypted
```

### Using Aliases for Speed

```bash
# Quick provider list
lc sy p

# Quick configuration setup
lc sy c s3 s

# Quick encrypted sync
lc sy to s3 -e
lc sy from s3 -e
```

## Troubleshooting

### Common Issues

1. **No Configuration Files Found**
   ```
   ⚠️ No configuration files found to sync
   ```
   - Ensure you have `.toml` files in your lc config directory
   - Check config directory path with `lc config path`

2. **S3 Access Denied**
   ```
   Cannot access S3 bucket 'bucket-name': Access Denied
   ```
   - Verify AWS credentials are correct
   - Ensure bucket exists and you have read/write permissions
   - Check bucket policy and IAM permissions

3. **Decryption Failed**
   ```
   Failed to decrypt file.toml.enc: Check your password
   ```
   - Ensure you're using the same password used for encryption
   - Verify the file was actually encrypted (check for `.enc` extension)

4. **Network/Endpoint Issues**
   ```
   Failed to connect to S3 endpoint
   ```
   - Check internet connectivity
   - Verify custom endpoint URL if using S3-compatible services
   - Ensure region is correct

### Debug Information

For detailed debugging, you can check:

1. **Configuration Directory**:
   ```bash
   lc config path
   ```

2. **Stored Sync Configuration**:
   ```bash
   lc sync configure s3 show
   ```

3. **Environment Variables**:
   ```bash
   env | grep -E "(LC_S3|AWS_)"
   ```

## Security Best Practices

1. **Use Stored Configuration**: Prefer `lc sync configure` over environment variables
2. **Enable Encryption**: Always use `--encrypted` for sensitive configurations
3. **Strong Passwords**: Use strong, unique passwords for encryption
4. **Bucket Security**: Configure proper S3 bucket policies and IAM permissions
5. **Network Security**: Use HTTPS endpoints (default for AWS S3)
6. **Access Control**: Limit S3 bucket access to specific IP ranges if possible

## Advanced Usage

### Custom S3 Endpoints

For S3-compatible services, you can specify custom endpoints during configuration:

```bash
lc sync configure s3 setup
# When prompted for endpoint, enter your custom URL:
# - Backblaze B2: https://s3.us-west-004.backblazeb2.com
# - Cloudflare R2: https://your-account-id.r2.cloudflarestorage.com
# - MinIO: https://your-minio-server.com:9000
```

### Multiple Configurations

You can manage multiple S3 configurations by:
1. Using different bucket names for different environments
2. Switching between stored configurations as needed
3. Using environment variables to override stored settings temporarily

### Automation

The sync feature can be automated in scripts:

```bash
#!/bin/bash
# Automated backup script

# Set credentials via environment
export LC_S3_BUCKET=my-backup-bucket
export AWS_ACCESS_KEY_ID=your-key
export AWS_SECRET_ACCESS_KEY=your-secret

# Sync with encryption (password from file or prompt)
echo "your-encryption-password" | lc sync to s3 --encrypted
```

## API and Integration

The sync functionality is built as a modular system that can be extended:

- **Provider Interface**: Easy to add new cloud providers
- **Encryption Interface**: Pluggable encryption algorithms
- **Configuration System**: TOML-based configuration storage
- **Cross-Platform**: Uses platform-appropriate config directories

## Future Enhancements

Planned improvements include:
- Additional cloud providers (Google Cloud Storage, Azure Blob Storage)
- Alternative encryption algorithms
- Compression support
- Incremental sync (only changed files)
- Sync scheduling and automation
- Configuration versioning and rollback
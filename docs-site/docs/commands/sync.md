---
id: sync
title: Sync Command
sidebar_position: 15
---

# Sync Command

Sync configuration files and chat logs database to/from cloud providers. This command enables backup and synchronization of your LLM Client configuration and chat history across multiple environments.

## Overview

The sync command provides seamless configuration and data management by allowing you to upload your local configuration files and chat logs database to cloud storage and download them from cloud providers. This ensures consistent setups and preserves chat history across different machines while enabling easy backup/restore operations.

### What Gets Synced

- **Configuration Files (.toml)**: All TOML configuration files in your lc config directory
  - `config.toml` - Main configuration
  - `keys.toml` - API keys (encrypted recommended)
  - `sync.toml` - Sync settings
  - `mcp.toml` - MCP server configurations
  - `providers/*.toml` - Provider-specific configurations
- **Database Files (.db)**: All database files for logs and embeddings
  - `logs.db` - Complete chat history and session data
  - `embeddings/*.db` - Vector database files
- **Encryption Support**: Optional AES256-GCM encryption for secure storage

## Usage

```bash
# List supported cloud providers
lc sync providers

# Configure a cloud provider
lc sync configure <provider>

# Upload configuration to cloud
lc sync to <provider>

# Download configuration from cloud
lc sync from <provider>

# Using aliases
lc sy to aws
lc sy from aws
```

## Subcommands

| Name        | Alias | Description                           |
|-------------|-------|---------------------------------------|
| `providers` | `p`   | List supported cloud providers        |
| `configure` | `c`   | Configure cloud provider settings     |
| `to`        | -     | Sync configuration to cloud provider  |
| `from`      | -     | Sync configuration from cloud provider|

## Options

| Short | Long          | Description                    | Default |
|-------|---------------|--------------------------------|---------|
| `-e`  | `--encrypted` | Enable encryption/decryption   | False   |
| `-h`  | `--help`      | Print help                     | False   |

## Examples

### Setup Cloud Provider

```bash
# List supported providers
lc sync providers
# Output: s3 (Amazon S3 and S3-compatible services)

# Configure S3
lc sync configure s3 setup
# Will prompt for S3 credentials and settings

# View current configuration
lc sync configure s3 show

# Remove configuration
lc sync configure s3 remove
```

### Sync Configuration and Database

```bash
# Upload to S3 (includes config files and logs.db)
lc sync to s3

# Download from S3
lc sync from s3

# With encryption (recommended for sensitive data)
lc sync to s3 --encrypted
lc sync from s3 --encrypted

# Using aliases
lc sy to s3
lc sy from s3 -e
```

### Multi-environment Workflow

```bash
# On machine A: clean up logs and upload
lc logs purge --older-than-days 30 --keep-recent 1000
lc sync to s3 --encrypted

# On machine B: download config and logs
lc sync from s3 --encrypted

# Now both machines have identical configurations and chat history
```

### Database Management Before Sync

```bash
# Check database size before syncing
lc logs stats

# Optimize database size for faster sync
lc logs purge --older-than-days 30 --keep-recent 1000 --max-size-mb 50

# Then sync with optimized database
lc sync to s3 --encrypted
```

## Troubleshooting

### Common Issues

#### "Provider not configured"

- **Error**: S3 provider settings missing
- **Solution**: Run `lc sync configure s3 setup` first

#### "Authentication failed"

- **Error**: Invalid S3 credentials
- **Solution**: Check and update your S3 credentials
- **Environment Variables**: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`
- **Bucket Variables**: `LC_S3_BUCKET`, `LC_S3_REGION`
- **Custom Endpoint**: `LC_S3_ENDPOINT` (for S3-compatible services)

#### "Network error"

- **Error**: Unable to connect to S3 endpoint
- **Solution**: Check internet connection and firewall settings
- **Custom Endpoints**: Verify endpoint URL for S3-compatible services

#### "Permission denied"

- **Error**: Insufficient permissions for S3 bucket
- **Solution**: Verify IAM roles and bucket permissions
- **Required Permissions**: `s3:GetObject`, `s3:PutObject`, `s3:ListBucket`

#### "Decryption failed"

- **Error**: Cannot decrypt downloaded files
- **Solution**: Ensure you're using the same password used for encryption
- **Check**: Verify files have `.enc` extension in S3

### Security Considerations

- Configuration files and chat logs may contain sensitive information
- **Always use encryption** for sensitive data: `lc sync to s3 --encrypted`
- Use strong, unique passwords for encryption
- Configure proper S3 bucket policies and IAM permissions
- Regularly rotate S3 access keys
- Consider purging old chat logs before syncing to reduce exposure

### Performance Tips

- **Purge logs before sync** to reduce upload/download time
- Use `lc logs purge --older-than-days 30 --keep-recent 1000 --max-size-mb 50`
- Monitor database size with `lc logs stats`
- Consider regional S3 buckets for faster sync

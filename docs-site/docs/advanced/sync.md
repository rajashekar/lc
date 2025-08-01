---
id: sync
title: Configuration Sync
sidebar_position: 4
---

# Configuration Sync

Synchronize your LLM Client configuration across multiple machines using cloud storage with optional encryption.

## Overview

The sync feature allows you to:
- Back up configuration to cloud storage
- Share settings across machines
- Encrypt sensitive data with AES256-GCM
- Support multiple cloud providers

## Quick Start

```bash
# 1. Configure cloud provider
lc sync configure s3 setup

# 2. Sync to cloud (with encryption)
lc sync to s3 --encrypted

# 3. Sync from cloud (on another machine)
lc sync from s3 --encrypted
```

## Command: `lc sync` (alias: `lc sy`)

### Subcommands

#### List Providers
```bash
lc sync providers
lc sy p
```

Shows supported cloud storage providers.

#### Configure Provider
```bash
# Interactive setup
lc sync configure <provider> setup
lc sy c <provider> s

# Show configuration
lc sync configure <provider> show
lc sy c <provider> sh

# Remove configuration
lc sync configure <provider> remove
lc sy c <provider> r
```

#### Sync Operations
```bash
# Sync to cloud
lc sync to <provider>
lc sy to <provider>

# Sync to cloud with encryption
lc sync to <provider> --encrypted
lc sy to <provider> -e

# Sync from cloud
lc sync from <provider>
lc sy from <provider>

# Sync from cloud with decryption
lc sync from <provider> --encrypted
lc sy from <provider> -e
```

## Supported Providers

### Amazon S3

Full S3 support with multiple configuration methods:

#### 1. Interactive Setup (Recommended)
```bash
lc sync configure s3 setup
```

You'll be prompted for:
- Bucket name
- AWS region
- Access key ID
- Secret access key
- Custom endpoint (optional)

#### 2. Environment Variables
```bash
export LC_S3_BUCKET=your-bucket-name
export LC_S3_REGION=us-east-1
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
export LC_S3_ENDPOINT=https://s3.amazonaws.com  # Optional
```

#### 3. Configuration File
Stored automatically after setup in `sync.toml`.

### S3-Compatible Services

Configure custom endpoints for S3-compatible services:

#### Backblaze B2
```bash
lc sync configure s3 setup
# Endpoint: https://s3.us-west-004.backblazeb2.com
```

#### Cloudflare R2
```bash
lc sync configure s3 setup
# Endpoint: https://your-account-id.r2.cloudflarestorage.com
```

#### MinIO
```bash
lc sync configure s3 setup
# Endpoint: https://your-minio-server.com:9000
```

#### DigitalOcean Spaces
```bash
lc sync configure s3 setup
# Endpoint: https://your-region.digitaloceanspaces.com
```

## What Gets Synced

All `.toml` configuration files in your config directory:

- `config.toml` - Providers, API keys, defaults
- `mcp.toml` - MCP server configurations
- `sync.toml` - Sync provider settings
- Any other `.toml` files

**Note**: Vector databases and logs are NOT synced.

## Encryption

### How It Works

When using `--encrypted`:
1. You're prompted for a password
2. Files are encrypted with AES256-GCM
3. Each file gets unique salt and nonce
4. Encrypted files get `.enc` extension
5. Original structure is preserved

### Encryption Details

- **Algorithm**: AES256-GCM (authenticated encryption)
- **Key Derivation**: PBKDF2-SHA256 with random salt
- **Security**: Military-grade encryption
- **Password**: Never stored, must remember

### Example Workflow

```bash
# First machine - backup with encryption
lc sync to s3 --encrypted
Enter encryption password: ********
Confirm password: ********
✓ Encrypted config.toml
✓ Encrypted mcp.toml
✓ Uploaded 2 files to s3://your-bucket/lc-config/

# Second machine - restore with decryption
lc sync from s3 --encrypted
Enter decryption password: ********
✓ Downloaded 2 files from s3://your-bucket/lc-config/
✓ Decrypted config.toml
✓ Decrypted mcp.toml
```

## Best Practices

### 1. Regular Backups

Set up a routine:
```bash
# Weekly backup
lc sync to s3 --encrypted
```

### 2. Password Management

- Use a strong, memorable password
- Store password in password manager
- Same password needed for decryption

### 3. Version Control

Consider versioning in S3:
- Enable bucket versioning
- Allows rollback if needed
- Track configuration history

### 4. Security

- Always use `--encrypted` for sensitive data
- Use IAM policies to restrict bucket access
- Enable MFA for AWS account
- Use dedicated sync credentials

## Complete Setup Example

### 1. Create S3 Bucket

```bash
# Using AWS CLI
aws s3 mb s3://my-lc-config-backup
aws s3api put-bucket-versioning \
  --bucket my-lc-config-backup \
  --versioning-configuration Status=Enabled
```

### 2. Create IAM User

Create user with minimal permissions:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket",
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject"
      ],
      "Resource": [
        "arn:aws:s3:::my-lc-config-backup",
        "arn:aws:s3:::my-lc-config-backup/*"
      ]
    }
  ]
}
```

### 3. Configure LLM Client

```bash
lc sync configure s3 setup
# Enter bucket: my-lc-config-backup
# Enter region: us-east-1
# Enter access key: AKIA...
# Enter secret key: ********
```

### 4. Test Sync

```bash
# Backup
lc sync to s3 --encrypted

# Verify
aws s3 ls s3://my-lc-config-backup/lc-config/

# Restore (on another machine)
lc sync from s3 --encrypted
```

## Troubleshooting

### "Access Denied"

1. Check AWS credentials:
   ```bash
   lc sync configure s3 show
   ```

2. Verify bucket permissions

3. Check bucket region matches configuration

### "Bucket not found"

1. Verify bucket name spelling
2. Ensure bucket exists in specified region
3. Check endpoint URL for S3-compatible services

### "Decryption failed"

1. Ensure using same password as encryption
2. Check if files were encrypted
3. Verify files weren't corrupted

### "Network error"

1. Check internet connection
2. Verify firewall allows S3 access
3. Try different region/endpoint

## Advanced Usage

### Selective Sync

Currently syncs all `.toml` files. For selective sync:
1. Move files to temporary location
2. Sync desired files
3. Restore other files

### Multiple Profiles

Use different buckets for different setups:
```bash
# Personal config
LC_S3_BUCKET=personal-lc lc sync to s3 -e

# Work config
LC_S3_BUCKET=work-lc lc sync to s3 -e
```

### Automation

Create backup script:
```bash
#!/bin/bash
# backup-lc.sh
echo "Backing up LLM Client config..."
lc sync to s3 --encrypted
echo "Backup complete!"
```

Add to crontab for automatic backups:
```bash
# Daily at 2 AM
0 2 * * * /path/to/backup-lc.sh
```

## Security Considerations

1. **API Keys**: Stored in config files - always use encryption
2. **Passwords**: Never stored - must remember or save securely
3. **Network**: Uses HTTPS for all transfers
4. **Storage**: Encrypted at rest if using `--encrypted`
5. **Access**: Use principle of least privilege for cloud credentials

## Next Steps

- Set up your first sync provider
- Enable encryption for sensitive data
- Create a backup routine
- Test restore process
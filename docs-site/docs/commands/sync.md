---
id: sync
title: Sync Command
sidebar_position: 15
---

# Sync Command

Sync configuration files to/from cloud providers. This command enables backup and synchronization of your LLM Client configuration across multiple environments.

## Overview

The sync command provides seamless configuration management by allowing you to upload your local configuration to cloud storage and download configurations from cloud providers. This ensures consistent setups across different machines and enables easy backup/restore operations.

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

| Short | Long     | Description | Default |
|-------|----------|-------------|---------|
| `-h`  | `--help` | Print help  | False   |

## Examples

### Setup Cloud Provider

```bash
# List supported providers
lc sync providers
# Output: aws, azure, gcp, dropbox

# Configure AWS
lc sync configure aws
# Will prompt for AWS credentials and settings
```

### Sync Configuration

```bash
# Upload to AWS
lc sync to aws

# Download from AWS
lc sync from aws

# Using aliases
lc sy to aws
lc sy from aws
```

### Multi-environment Workflow

```bash
# On machine A: upload config
lc sync to aws

# On machine B: download config
lc sync from aws

# Now both machines have identical configurations
```

## Troubleshooting

### Common Issues

#### "Provider not configured"

- **Error**: Cloud provider settings missing
- **Solution**: Run `lc sync configure <provider>` first

#### "Authentication failed"

- **Error**: Invalid cloud provider credentials
- **Solution**: Check and update your cloud credentials
- **AWS**: Verify `~/.aws/credentials` or environment variables
- **Azure**: Run `az login` to authenticate
- **GCP**: Check service account key or run `gcloud auth login`

#### "Network error"

- **Error**: Unable to connect to cloud provider
- **Solution**: Check internet connection and firewall settings

#### "Permission denied"

- **Error**: Insufficient permissions for cloud storage
- **Solution**: Verify IAM roles and bucket permissions

### Security Considerations

- Configuration files may contain sensitive information
- Use encrypted cloud storage when possible
- Consider using environment-specific configurations
- Regularly rotate cloud access keys

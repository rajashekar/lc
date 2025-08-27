---
id: troubleshooting
title: Troubleshooting
sidebar_position: 100
---

# Troubleshooting

Common issues and their solutions when using LLM Client.

## Installation Issues

### Rust Installation Fails

**Problem**: Can't install Rust or cargo commands not found

**Solutions**:

1. Ensure you have a C compiler installed:
   - Linux: `sudo apt install build-essential`
   - macOS: `xcode-select --install`
   - Windows: Install Visual Studio Build Tools

2. Try the official installer:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Add cargo to PATH:

   ```bash
   source $HOME/.cargo/env
   ```

### Build Errors

**Problem**: `cargo build --release` fails

**Solutions**:

1. Update Rust:

   ```bash
   rustup update
   ```

2. Clean and rebuild:

   ```bash
   cargo clean
   cargo build --release
   ```

3. Check for missing dependencies in error messages

### OpenSSL & pkg-config Build Errors

**Problem**: Build fails with OpenSSL-related errors

**Common Error Messages**:
- `Could not find directory of OpenSSL installation`
- `failed to run custom build command for 'openssl-sys'`
- `Could not find openssl via pkg-config`
- `The pkg-config command could not be found`

**Quick Solutions by Platform**:

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y pkg-config libssl-dev build-essential
```

#### Linux (RHEL/CentOS)
```bash
# RHEL/CentOS
sudo yum install -y pkgconfig openssl-devel gcc

# Fedora
sudo dnf install -y pkgconf-devel openssl-devel gcc
```

#### Linux (Alpine)
```bash
sudo apk add pkgconf openssl-dev build-base
```

#### macOS
```bash
# Install Xcode Command Line Tools first
xcode-select --install

# If still having issues, install via Homebrew
brew install pkg-config openssl@3
```

#### Windows
- Install Visual Studio Build Tools with C++ support
- Rust usually bundles OpenSSL statically, so system packages aren't needed
- If issues persist, try using `openssl-src` feature or static linking

**Verification Steps**:

1. Check if pkg-config can find OpenSSL:
   ```bash
   pkg-config --exists openssl && echo "OpenSSL found"
   pkg-config --libs --cflags openssl
   ```

2. Test a simple Rust build with OpenSSL dependency:
   ```bash
   cargo new test_openssl
   cd test_openssl
   # Add openssl to Cargo.toml dependencies
   echo 'openssl = "0.10"' >> Cargo.toml
   cargo build
   ```

**Advanced Solutions**:

1. **Manual OpenSSL path** (if installed in non-standard location):
   ```bash
   export OPENSSL_DIR=/usr/local/ssl
   export PKG_CONFIG_PATH=/usr/local/ssl/lib/pkgconfig:$PKG_CONFIG_PATH
   cargo build --release
   ```

2. **Use vendored OpenSSL** (if system OpenSSL is problematic):
   ```bash
   # Add to Cargo.toml
   [dependencies.openssl]
   version = "0.10"
   features = ["vendored"]
   ```

3. **Cross-compilation issues**:
   ```bash
   # For cross-compilation, ensure target has required libraries
   sudo apt install gcc-aarch64-linux-gnu
   export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
   cargo build --target aarch64-unknown-linux-gnu --release
   ```

**References**:
- [Rust OpenSSL crate documentation](https://docs.rs/openssl/latest/openssl/)
- [openssl-sys build instructions](https://docs.rs/openssl-sys/latest/openssl_sys/)

## Provider Issues

### "No providers configured"

**Problem**: Getting error when trying to use lc

**Solution**:

```bash
# Add a provider first
lc providers add openai https://api.openai.com/v1

# Verify it's added
lc providers list
```

### "Provider not found"

**Problem**: Provider name not recognized

**Solutions**:

1. Check exact spelling:

   ```bash
   lc providers list
   ```

2. Ensure provider is added:

   ```bash
   lc providers add <name> <url>
   ```

### "Invalid endpoint URL"

**Problem**: Provider endpoint rejected

**Solutions**:

1. Include protocol: `https://` or `http://`
2. Don't include trailing paths unless needed
3. For custom endpoints, use `-m` and `-c` flags

## API Key Issues

### "No API key found"

**Problem**: Provider requires API key

**Solution**:

```bash
lc keys add <provider>
# Enter key when prompted
```

### "Authentication failed"

**Problem**: API key rejected

**Solutions**:

1. Verify key is correct:

   ```bash
   lc keys remove <provider>
   lc keys add <provider>
   ```

2. Check if provider needs custom headers:

   ```bash
   # For Claude
   lc providers headers claude add x-api-key <key>
   lc providers headers claude add anthropic-version 2023-06-01
   ```

3. Ensure you have API credits/quota

## Model Issues

### "Model not found"

**Problem**: Specified model doesn't exist

**Solutions**:

1. List available models:

   ```bash
   lc providers models <provider>
   # or
   lc models
   ```

2. Use exact model name from the list

3. For HF router, use format: `model:provider`

### "Context length exceeded"

**Problem**: Input too long for model

**Solutions**:

1. Use a model with larger context:

   ```bash
   lc models --ctx 128k
   ```

2. Reduce input length

3. Split into multiple prompts

## Vector Database Issues

### "Database not found"

**Problem**: Vector database doesn't exist

**Solutions**:

1. Check available databases:

   ```bash
   lc vectors list
   ```

2. Create database by adding content:

   ```bash
   lc embed -m text-embedding-3-small -v <name> "content"
   ```

### "Dimension mismatch"

**Problem**: Embedding dimensions don't match

**Solutions**:

1. Check database model:

   ```bash
   lc vectors info <database>
   ```

2. Use the same model for all operations

3. Delete and recreate with consistent model:

   ```bash
   lc vectors delete <database>
   lc embed -m <model> -v <database> "content"
   ```

### "No similar content found"

**Problem**: Similarity search returns nothing

**Solutions**:

1. Verify database has content:

   ```bash
   lc vectors info <database>
   ```

2. Try different search terms

3. Check if content is relevant

## Chat Issues

### "Session not found"

**Problem**: Can't continue chat session

**Solutions**:

1. List recent sessions:

   ```bash
   lc logs recent
   ```

2. Use correct session ID:

   ```bash
   lc chat -m <model> --cid <session-id>
   ```

### "Chat history lost"

**Problem**: Previous messages not remembered

**Solutions**:

1. Ensure you're in chat mode:

   ```bash
   lc chat -m <model>
   ```

2. Don't use `/clear` unless you want to reset

3. Check logs database:

   ```bash
   lc logs stats
   ```

## Performance Issues

### Slow Response Times

**Solutions**:

1. Use a faster model:

   ```bash
   lc -m gpt-3.5-turbo "prompt"
   ```

2. Check network connection

3. Try a different provider

### High Token Usage

**Solutions**:

1. Use concise prompts
2. Set up system prompts for consistency
3. Use smaller models when appropriate

## Sync Issues

### "Sync failed"

**Problem**: Can't sync configuration

**Solutions**:

1. Check provider configuration:

   ```bash
   lc sync configure s3 show
   ```

2. Verify credentials:

   ```bash
   lc sync configure s3 setup
   ```

3. Check network/firewall settings

### "Decryption failed"

**Problem**: Can't decrypt synced files

**Solution**: Use the same password used for encryption

## Proxy Issues

### Proxy `-h` Conflict

**Problem**: Using `-h` flag with proxy command doesn't work as expected

**Cause**: The `-h` flag conflicts between `--host` and `--help` options

**Solutions**:

1. Use full flag names to avoid ambiguity:

   ```bash
   # Instead of: lc proxy -h 0.0.0.0
   lc proxy --host 0.0.0.0
   ```

2. Use `--help` instead of `-h` for help:

   ```bash
   lc proxy --help
   ```

**Reference**: See [Proxy Command Documentation](/commands/proxy#options) for all available flags

### Web Chat Proxy Port Issues

**Problem**: "Port already in use" or "Address already in use" errors

**Solutions**:

1. Check what's using the port:

   ```bash
   netstat -tlnp | grep :8080
   # or on macOS
   lsof -i :8080
   ```

2. Use a different port:

   ```bash
   lc web-chat-proxy start anthropic --port 3000
   ```

3. Stop existing proxy servers:

   ```bash
   lc web-chat-proxy list
   lc web-chat-proxy stop anthropic
   ```

4. Kill process using the port (if needed):

   ```bash
   # Replace PID with actual process ID from netstat/lsof
   kill -9 <PID>
   ```

**Reference**: See [Web Chat Proxy Documentation](/commands/web-chat-proxy#troubleshooting) for more details

### Sync Provider Authentication Errors

**Problem**: "Authentication failed" when syncing with cloud providers

**Solutions by Provider**:

**AWS**:

1. Check AWS credentials:

   ```bash
   cat ~/.aws/credentials
   ```

2. Set environment variables:

   ```bash
   export AWS_ACCESS_KEY_ID=your-key
   export AWS_SECRET_ACCESS_KEY=your-secret
   ```

3. Verify IAM permissions for S3 bucket access

**Azure**:

1. Login to Azure CLI:

   ```bash
   az login
   ```

2. Check current account:

   ```bash
   az account show
   ```

**GCP**:

1. Authenticate with service account:

   ```bash
   gcloud auth activate-service-account --key-file=key.json
   ```

2. Or login interactively:

   ```bash
   gcloud auth login
   ```

**General Steps**:

1. Reconfigure the provider:

   ```bash
   lc sync configure <provider>
   ```

2. Test connectivity:

   ```bash
   lc sync providers
   ```

3. Check network/firewall settings

**Reference**: See [Sync Command Documentation](/commands/sync#troubleshooting) for provider-specific setup

## Debug Mode

For detailed error information:

```bash
# Enable debug logging
export RUST_LOG=debug
lc -m gpt-4 "test prompt"
```

## Getting Help

If you're still having issues:

1. Check the [FAQ](/faq)
2. Search [GitHub Issues](https://github.com/rajashekar/lc/issues)
3. Create a new issue with:
   - Error message
   - Steps to reproduce
   - System information
   - Debug logs

## Common Error Messages

| Error | Meaning | Solution |
|-------|---------|----------|
| "No providers configured" | No providers added | Add a provider |
| "API request failed" | Network or API error | Check connection and API key |
| "Model not found" | Invalid model name | Use exact model name |
| "Rate limit exceeded" | Too many requests | Wait and retry |
| "Invalid API key" | Wrong or expired key | Update API key |
| "Context length exceeded" | Input too long | Use shorter input or larger model |
| "Could not find OpenSSL" | Missing system dependencies | Install pkg-config and libssl-dev |
| "pkg-config not found" | Missing build tool | Install pkg-config package |

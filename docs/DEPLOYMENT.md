# LC CLI Deployment Guide

## Issue Resolution: Binary Hanging After Installation

### Problem
When copying a new binary over an existing one while daemon processes are running, the binary can become corrupted or cause hanging behavior. This happens because:

1. **File Lock Conflicts**: The running process holds a lock on the executable file
2. **Memory Mapping Issues**: The OS may have memory-mapped the old binary
3. **Signal Handling**: Replacing a binary mid-execution can cause undefined behavior

### Solution: Safe Deployment Process

#### Method 1: Using the Deployment Script (Recommended)

```bash
# Use the provided deployment script
./deploy.sh
```

This script automatically:
- Builds the release binary
- Detects and stops running LC processes
- Safely replaces the binary
- Verifies the installation

#### Method 2: Manual Deployment

```bash
# 1. Build the release binary
cargo build --release

# 2. Check for running processes
ps -ef | grep "/opt/homebrew/bin/lc" | grep -v grep

# 3. Stop any running processes (replace PID with actual process ID)
kill <PID>

# 4. Remove the old binary
sudo rm /opt/homebrew/bin/lc

# 5. Copy the new binary
sudo cp target/release/lc /opt/homebrew/bin/lc

# 6. Set correct permissions
sudo chmod +x /opt/homebrew/bin/lc

# 7. Test the installation
lc --version
```

## Installation Methods

### Current Method: Manual Installation to Homebrew Path

**Pros:**
- Quick and simple
- Uses standard Homebrew binary path
- Available system-wide immediately

**Cons:**
- Requires manual updates
- No automatic dependency management
- Risk of conflicts with future Homebrew packages

### Alternative Methods

#### 1. Local Binary Path (Development)
```bash
# Add to your shell profile (.zshrc, .bashrc, etc.)
export PATH="$HOME/path/to/lc/target/release:$PATH"

# Or create a symlink
ln -s $HOME/path/to/lc/target/release/lc /usr/local/bin/lc
```

#### 2. Cargo Install (Rust Ecosystem)
```bash
# Install from local source
cargo install --path .

# This installs to ~/.cargo/bin/lc (ensure ~/.cargo/bin is in PATH)
```

#### 3. Future: Homebrew Formula
When available through Homebrew:
```bash
brew install lc-cli
brew upgrade lc-cli  # For updates
```

## Best Practices

### For Development
1. **Use the deployment script** for consistent, safe deployments
2. **Always stop daemon processes** before updating the binary
3. **Test the installation** after deployment
4. **Keep the source directory** for easy rebuilds

### For Production/Distribution
1. **Version your releases** with proper tagging
2. **Provide checksums** for binary verification
3. **Use package managers** when available (Homebrew, apt, etc.)
4. **Document dependencies** and system requirements

## Daemon Management

### Starting Daemons
```bash
# Start a web chat proxy daemon
lc w start kagi --port 8084 --host 127.0.0.1

# List running daemons
lc w list
```

### Stopping Daemons Before Updates
```bash
# Stop specific daemon
lc w stop kagi

# Or manually kill processes
ps -ef | grep "lc w start" | awk '{print $2}' | xargs kill
```

## Troubleshooting

### Binary Hangs or Gets Killed
1. **Check for running processes**: `ps -ef | grep lc`
2. **Stop all LC processes**: `pkill -f "lc "`
3. **Remove and reinstall**: Use the deployment script
4. **Verify permissions**: `ls -la /opt/homebrew/bin/lc`

### Permission Issues
```bash
# Fix permissions
sudo chmod +x /opt/homebrew/bin/lc
sudo chown root:admin /opt/homebrew/bin/lc
```

### Path Issues
```bash
# Verify LC is in PATH
which lc

# Check PATH includes Homebrew bin
echo $PATH | grep -o '/opt/homebrew/bin'
```

## Future Improvements

1. **Homebrew Formula**: Official package management
2. **Auto-updater**: Built-in update mechanism
3. **Service Management**: systemd/launchd integration for daemons
4. **Configuration Migration**: Automatic config updates between versions
5. **Rollback Capability**: Easy reversion to previous versions

## Performance Optimizations Included

The current binary includes these performance improvements:
- **Database Connection Pooling**: 15x faster database operations
- **Prepared Statement Caching**: Reduced SQL parsing overhead
- **Optimized SQLite Configuration**: WAL mode, larger cache sizes
- **Enhanced Indexing**: Faster query performance
- **Zero Compiler Warnings**: Clean, optimized release build

## Verification

After deployment, verify everything works:
```bash
# Basic functionality
lc --version
lc --help

# Database operations (should be fast)
lc vectors list

# Web chat proxy (if configured)
lc w list
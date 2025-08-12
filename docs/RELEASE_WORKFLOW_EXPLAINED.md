# GitHub Release Workflow Explained

## 🤖 **What `.github/workflows/release.yml` Does**

The `release.yml` file is a **GitHub Actions workflow** that automates the entire release process when you create a new version tag. Here's what it does:

## 🚀 **Trigger: Git Tags**

```yaml
on:
  push:
    tags:
      - 'v*'  # Triggers on tags like v1.0.0, v1.2.3, etc.
```

**When it runs**: Automatically when you push a version tag like `v1.0.0`

```bash
# This command will trigger the workflow:
git tag v1.0.0
git push origin v1.0.0
```

## 🏗️ **What It Does (4 Main Jobs)**

### **Job 1: `create-release` 📝**
- **Purpose**: Creates a GitHub Release with release notes
- **Output**: Release page with installation instructions
- **Content**: Includes `cargo install lc-cli` instructions

### **Job 2: `build-and-upload` 🔧**  
- **Purpose**: Builds binaries for all platforms
- **Platforms**: Linux (x64/ARM), macOS (Intel/M1), Windows
- **Output**: Pre-built binaries attached to the release

| Platform | Binary File |
|----------|-------------|
| Linux x64 | `lc-linux-amd64.tar.gz` |
| Linux ARM64 | `lc-linux-arm64.tar.gz` |
| macOS Intel | `lc-macos-amd64.tar.gz` |
| macOS M1/M2 | `lc-macos-arm64.tar.gz` |
| Windows | `lc-windows-amd64.zip` |

### **Job 3: `publish-crates` 📦**
- **Purpose**: Publishes to crates.io automatically
- **Requirements**: Needs `CRATES_TOKEN` secret
- **Result**: Updates https://crates.io/crates/lc-cli

### **Job 4: `update-homebrew` 🍺**
- **Purpose**: Updates Homebrew formula automatically  
- **Requirements**: Needs `HOMEBREW_TOKEN` and homebrew-tap repo
- **Result**: Users can `brew install lc`

## ⚠️ **Current Status Issues**

### **❌ Crates.io Conflict**
The workflow tries to publish to crates.io, but you **already published manually**:
- ✅ **You**: Published `lc-cli v0.1.0` to https://crates.io/crates/lc-cli
- ❌ **Workflow**: Will fail trying to publish the same version

### **🔧 Missing Secrets**
The workflow needs GitHub repository secrets:

```bash
# Required secrets (you need to add these):
CRATES_TOKEN        # Your crates.io API token  
HOMEBREW_TOKEN      # GitHub token for homebrew tap
```

### **🍺 Homebrew Setup Required**
The workflow expects:
- A `homebrew-tap` repository (e.g., `your-username/homebrew-tap`)
- Proper homebrew formula setup

## 🛠️ **How to Fix the Workflow**

### **Option 1: Disable Crates Publishing**
Since you already published manually, comment out the crates job:

```yaml
# Comment out or remove this job since package is already published
# publish-crates:
#   name: Publish to crates.io
#   needs: build-and-upload  
#   runs-on: ubuntu-latest
#   steps:
#     - uses: actions/checkout@v4
#     - uses: dtolnay/rust-toolchain@stable
#     - run: cargo publish --token ${{ secrets.CRATES_TOKEN }}
```

### **Option 2: Setup Secrets for Future Releases**
Add these to your GitHub repository settings → Secrets:

1. **CRATES_TOKEN**: 
   ```bash
   # Get from https://crates.io/me
   # Settings → API Tokens → New Token
   ```

2. **HOMEBREW_TOKEN**:
   ```bash  
   # GitHub Personal Access Token
   # Settings → Developer Settings → Personal Access Tokens
   ```

### **Option 3: Simplified Workflow**
Keep only binary building for now:

```yaml
# Keep: create-release, build-and-upload
# Remove: publish-crates, update-homebrew  
```

## 🎯 **What Happens When You Tag v1.0.1**

Assuming the workflow runs successfully:

1. **✅ GitHub Release Created**
   - Release page: `https://github.com/your-repo/releases/tag/v1.0.1`
   - Installation instructions included
   - Binaries attached

2. **✅ Binaries Built**
   - 5 platform-specific binaries
   - Users can download directly

3. **❌ Crates.io Publish** (will fail - version already exists)
   
4. **❌ Homebrew Update** (will fail - no secrets/tap setup)

## 💡 **Recommendations**

### **For Next Release (v1.0.1)**:

1. **Update version in Cargo.toml** to `1.0.1`
2. **Disable crates publishing** (already published manually)  
3. **Create git tag**: `git tag v1.0.1 && git push origin v1.0.1`
4. **Workflow creates**: GitHub release + binaries

### **For Future (v1.1.0+)**:

1. **Setup secrets** for automated crates.io publishing
2. **Setup homebrew tap** for automated brew formula
3. **Full automation**: Tag → Release → Crates.io → Homebrew

## 📊 **Workflow Success Matrix**

| Job | Current Status | Fix Required |
|-----|----------------|-------------|
| **create-release** | ✅ Works | None |
| **build-and-upload** | ✅ Works | None |  
| **publish-crates** | ❌ Fails | Remove or add secrets |
| **update-homebrew** | ❌ Fails | Setup tap + secrets |

## 🔄 **Alternative: Manual Process**

If the workflow is complex, you can also release manually:

```bash
# 1. Update version
vim Cargo.toml

# 2. Build and publish to crates.io  
cargo publish

# 3. Create GitHub release manually
git tag v1.0.1
git push origin v1.0.1
# Go to GitHub → Releases → Create Release

# 4. Attach binaries manually
cargo build --release --target x86_64-unknown-linux-gnu
# ... repeat for other targets
```

## 🎉 **Summary**

The `release.yml` workflow is a **powerful automation tool** that:

- ✅ **Creates GitHub releases** with proper formatting
- ✅ **Builds cross-platform binaries** automatically  
- ✅ **Can publish to crates.io** (when secrets are setup)
- ✅ **Can update Homebrew** (when tap is setup)

But requires **proper setup** to work fully. For now, it will create releases and binaries, but the publishing steps may fail without the required secrets and configuration.

The good news is that **the most important part works**: creating releases with pre-built binaries for users who don't want to install via Cargo! 🚀

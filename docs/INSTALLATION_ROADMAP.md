# Installation Roadmap for LC (LLM Client)

## Current State & Platform Analysis

### ✅ **Currently Working**
- **From Source**: `git clone` + `cargo build --release` (all platforms)
- **WSL2**: Full compatibility with all Linux installation methods

### 🚧 **In Development** 
- Cargo package publication
- GitHub releases with pre-built binaries
- Package manager submissions

## Platform-Specific Installation Methods

### 🍎 **macOS**

| Method | Command | Status | Timeline | Priority |
|--------|---------|---------|----------|----------|
| **Homebrew** | `brew install lc` | 🚧 Planned | 2-4 weeks | **High** |
| **Cargo** | `cargo install llm-client` | 🚧 Ready to publish | 1 week | **High** |
| **Direct Binary** | Download from releases | 🚧 Planned | 2 weeks | **High** |
| **From Source** | `cargo build --release` | ✅ Working | - | **Current** |

**Homebrew Considerations:**
- ✅ No conflicts found with `brew search lc`
- ✅ Several related tools (`lc0`, `lci`, etc.) but no exact `lc` match
- 📝 Need to create Homebrew tap or submit to main repository

### 🐧 **Linux** 

| Distribution | Method | Command | Status | Priority |
|--------------|--------|---------|---------|----------|
| **Ubuntu/Debian** | APT | `apt install lc` | 🚧 Complex | Medium |
| **Fedora/RHEL** | DNF/YUM | `dnf install lc` | 🚧 Complex | Medium |
| **Arch Linux** | AUR | `yay -S lc` | 🚧 Planned | Medium |
| **Any Linux** | Snap | `snap install lc` | 🚧 Possible | Low |
| **Universal** | Cargo | `cargo install llm-client` | 🚧 Ready | **High** |
| **Direct Binary** | Download | Manual install | 🚧 Planned | **High** |

**Linux Package Manager Challenges:**
- 🚧 **APT/DNF**: Requires sponsorship and complex approval process
- ✅ **AUR**: Community-driven, easier to publish
- ✅ **Cargo**: Universal solution for all Linux distributions

### 🪟 **Windows**

| Method | Command | Status | WSL2 | Priority |
|--------|---------|---------|------|----------|
| **Scoop** | `scoop install lc` | 🚧 Planned | ❌ | **High** |
| **Chocolatey** | `choco install lc` | 🚧 Planned | ❌ | Medium |
| **WinGet** | `winget install lc` | 🚧 Planned | ❌ | Medium |
| **Cargo** | `cargo install llm-client` | 🚧 Ready | ✅ | **High** |
| **Direct Binary** | Download `.exe` | 🚧 Planned | ❌ | **High** |
| **From Source** | `cargo build --release` | ✅ Working | ✅ | **Current** |

**WSL2 Special Case:**
- ✅ **Full Linux Compatibility**: All Linux methods work perfectly
- ✅ **MCP Daemon Support**: Unix sockets work in WSL2
- 🎯 **Recommended**: Use Linux package managers within WSL2

## Name Conflict Analysis

### 🔍 **Package Manager Conflicts**

| Platform | Existing `lc` | Conflict Level | Recommendation |
|----------|---------------|----------------|----------------|
| **crates.io** | `lc = "0.3.0"` (Link checker) | ⚠️ Minor | Use `llm-client` package name |
| **Homebrew** | None found | ✅ Safe | Use `lc` directly |
| **Scoop** | Unknown | 🔍 Need to check | TBD |
| **Chocolatey** | Unknown | 🔍 Need to check | TBD |
| **APT** | Unknown | 🔍 Need to check | TBD |
| **AUR** | Unknown | 🔍 Need to check | TBD |

### 💡 **Resolution Strategy**

**For Cargo (crates.io):**
```toml
# Option A: Avoid conflict with descriptive name
[package]
name = "llm-client"
# Users install: cargo install llm-client
# Binary name remains: lc

# Option B: Direct name (risky but simple)
[package] 
name = "lc"
# Users install: cargo install lc
```

**For Package Managers:**
- Most should accept `lc` as package name
- Fallback to `llm-client` or `lc-cli` if conflicts arise

## Implementation Timeline

### 📅 **Phase 1: Core Distribution (Week 1-2)**

1. **Cargo Package Publication**
   ```bash
   # Update Cargo.toml for publication
   cargo publish --dry-run
   cargo publish
   ```

2. **GitHub Releases Setup**
   ```yaml
   # Implement .github/workflows/release.yml
   # Auto-build binaries for all platforms
   # Upload to GitHub releases
   ```

3. **Documentation Update**
   - Update installation instructions
   - Add platform-specific guides
   - Document WSL2 compatibility

### 📅 **Phase 2: Package Managers (Week 3-6)**

4. **Homebrew (macOS)**
   ```ruby
   # Create homebrew-lc tap
   # Submit formula
   # Test installation process
   ```

5. **Scoop (Windows)**
   ```json
   # Create scoop bucket
   # Submit manifest
   # Test on Windows systems
   ```

6. **AUR (Arch Linux)**
   ```bash
   # Create PKGBUILD
   # Submit to AUR
   # Test with yay/paru
   ```

### 📅 **Phase 3: Official Repositories (Month 2-3)**

7. **Advanced Package Managers**
   - Debian/Ubuntu APT (requires sponsorship)
   - Chocolatey (requires moderation)
   - WinGet (Microsoft community repo)

## Installation Testing Matrix

### 🧪 **Test Platforms**

| OS | Version | Architecture | Installation Methods to Test |
|----|---------|-------------|------------------------------|
| **macOS** | 12+ | x86_64, arm64 | Homebrew, Cargo, Binary, Source |
| **Ubuntu** | 20.04, 22.04 | x86_64, arm64 | AUR, Cargo, Binary, Source |
| **Windows** | 10, 11 | x86_64 | Scoop, Cargo, Binary, Source |
| **WSL2** | Ubuntu | x86_64 | All Linux methods |
| **Arch** | Rolling | x86_64 | AUR, Cargo, Binary, Source |

### ✅ **Success Criteria**

- **Installation Success Rate**: > 99%
- **Cross-Platform Functionality**: All features work as expected
- **Package Manager Availability**: Within 24h of release
- **User Experience**: Simple one-command installation
- **Documentation**: Clear platform-specific instructions

## Alternative Names (If Conflicts Arise)

### 🏷️ **Backup Package Names**

| Platform | Primary | Alternative 1 | Alternative 2 |
|----------|---------|---------------|---------------|
| **Cargo** | `llm-client` | `lc-cli` | `llm-cli` |
| **Homebrew** | `lc` | `llm-client` | `lc-cli` |
| **Others** | `lc` | `llm-client` | `lc-tool` |

### 📝 **Binary Name Strategy**

```bash
# Binary always remains 'lc' regardless of package name
# Users always type: lc --help
# Package managers can have different names
```

## Special Platform Considerations

### 🪟 **Windows-Specific**

1. **Command Prompt vs PowerShell**: Both should work
2. **Windows Terminal**: Enhanced experience
3. **WSL2 Integration**: Full Unix feature parity
4. **Path Management**: Automatic PATH updates via installers

### 🐧 **Linux-Specific**

1. **Distribution Differences**: Package manager variations
2. **Permission Models**: User vs system installation
3. **Dependency Management**: Minimal external dependencies
4. **Shell Integration**: Works with bash, zsh, fish

### 🍎 **macOS-Specific**

1. **Homebrew Integration**: Automatic updates
2. **Apple Silicon**: Native ARM64 builds
3. **macOS Versions**: Support 10.15+ (to be determined)
4. **Code Signing**: Future consideration for distribution

## Monitoring and Metrics

### 📊 **Installation Analytics**

- Track installation methods by platform
- Monitor success/failure rates
- Collect user feedback on installation experience
- Measure time-to-first-use after installation

### 🔍 **Quality Assurance**

- Automated installation testing in CI/CD
- User acceptance testing on different platforms  
- Package manager compliance verification
- Security scanning of published packages

## Conclusion

The installation strategy prioritizes **universal availability** while maintaining **platform-native experience**. The phased approach ensures stable releases and proper testing, with Cargo serving as the universal fallback and platform-specific package managers providing the best user experience.

**Key Success Factors:**
1. ✅ **WSL2 Full Compatibility** - Windows users get full Unix features
2. 🚀 **Cargo as Universal Method** - Works everywhere Rust is available  
3. 🍎 **Native Package Managers** - Optimal experience per platform
4. 📖 **Clear Documentation** - Platform-specific installation guides
5. 🔄 **Automated Testing** - Ensure reliability across all methods

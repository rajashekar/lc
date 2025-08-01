# Package Distribution Plan for LLM Client

This document outlines the strategy for distributing LLM Client through popular package managers across different platforms.

## Overview

### Target Package Managers

| Platform | Package Manager | Priority | Installation Command |
|----------|----------------|----------|---------------------|
| macOS | Homebrew | High | `brew install lc` |
| Windows | Scoop | High | `scoop install lc` |
| Windows | Chocolatey | Medium | `choco install lc` |
| Windows | WinGet | Medium | `winget install lc` |
| Debian/Ubuntu | APT | High | `apt install lc` |
| Fedora/RHEL | DNF/YUM | Medium | `dnf install lc` |
| Arch Linux | AUR | Medium | `yay -S lc` |
| Any Linux | Snap | Low | `snap install lc` |
| Any | Cargo | High | `cargo install lc` |
| Any | NPM (wrapper) | Low | `npm install -g @lc/cli` |

## Release Strategy

### 1. Version Management

Use semantic versioning (SemVer):
- MAJOR.MINOR.PATCH (e.g., 1.2.3)
- Tag releases in Git: `v1.2.3`

### 2. Release Process

1. **Development** → `main` branch
2. **Testing** → `release/v1.2.3` branch
3. **Release** → Tag `v1.2.3`
4. **Distribution** → Automated via GitHub Actions

### 3. Platform Testing Matrix

| OS | Architecture | Rust Version | Test Suite |
|----|--------------|--------------|------------|
| Ubuntu 20.04 | x86_64 | 1.70+ | Full |
| Ubuntu 22.04 | x86_64 | 1.70+ | Full |
| macOS 12 | x86_64 | 1.70+ | Full |
| macOS 13 | aarch64 | 1.70+ | Full |
| Windows Server 2019 | x86_64 | 1.70+ | Full |
| Windows Server 2022 | x86_64 | 1.70+ | Full |

## GitHub Workflows

### Main Release Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          draft: false
          prerelease: false

  build-and-upload:
    name: Build and Upload
    needs: create-release
    strategy:
      matrix:
        include:
          # Linux
          - os: ubuntu-20.04
            target: x86_64-unknown-linux-gnu
            artifact_name: lc
            asset_name: lc-linux-amd64
          - os: ubuntu-20.04
            target: aarch64-unknown-linux-gnu
            artifact_name: lc
            asset_name: lc-linux-arm64
          
          # macOS
          - os: macos-12
            target: x86_64-apple-darwin
            artifact_name: lc
            asset_name: lc-macos-amd64
          - os: macos-12
            target: aarch64-apple-darwin
            artifact_name: lc
            asset_name: lc-macos-arm64
          
          # Windows
          - os: windows-2019
            target: x86_64-pc-windows-msvc
            artifact_name: lc.exe
            asset_name: lc-windows-amd64.exe

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
          asset_name: ${{ matrix.asset_name }}
          asset_content_type: application/octet-stream

  publish-crates:
    name: Publish to crates.io
    needs: build-and-upload
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo publish --token ${{ secrets.CRATES_TOKEN }}
```

### Package Manager Workflows

#### Homebrew (macOS)

Create `.github/workflows/homebrew.yml`:

```yaml
name: Update Homebrew Formula

on:
  release:
    types: [published]

jobs:
  update-homebrew:
    runs-on: ubuntu-latest
    steps:
      - name: Update Homebrew Formula
        uses: mislav/bump-homebrew-formula-action@v2
        with:
          formula-name: lc
          homebrew-tap: your-username/homebrew-tap
        env:
          COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TOKEN }}
```

#### Scoop (Windows)

Create `.github/workflows/scoop.yml`:

```yaml
name: Update Scoop Manifest

on:
  release:
    types: [published]

jobs:
  update-scoop:
    runs-on: windows-latest
    steps:
      - name: Update Scoop Manifest
        run: |
          # Update scoop manifest in your bucket
          # This would typically be a separate repository
```

## Package Configurations

### Homebrew Formula

Create `homebrew/lc.rb`:

```ruby
class Lc < Formula
  desc "Fast, Rust-based CLI for interacting with Large Language Models"
  homepage "https://lc.viwq.dev"
  version "1.0.0"
  
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/your-username/lc/releases/download/v1.0.0/lc-macos-arm64"
      sha256 "YOUR_SHA256_HERE"
    else
      url "https://github.com/your-username/lc/releases/download/v1.0.0/lc-macos-amd64"
      sha256 "YOUR_SHA256_HERE"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/your-username/lc/releases/download/v1.0.0/lc-linux-arm64"
      sha256 "YOUR_SHA256_HERE"
    else
      url "https://github.com/your-username/lc/releases/download/v1.0.0/lc-linux-amd64"
      sha256 "YOUR_SHA256_HERE"
    end
  end

  def install
    bin.install "lc"
  end

  test do
    system "#{bin}/lc", "--version"
  end
end
```

### Scoop Manifest

Create `scoop/lc.json`:

```json
{
    "version": "1.0.0",
    "description": "Fast, Rust-based CLI for interacting with Large Language Models",
    "homepage": "https://lc.viwq.dev",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/your-username/lc/releases/download/v1.0.0/lc-windows-amd64.exe",
            "hash": "YOUR_SHA256_HERE"
        }
    },
    "bin": "lc.exe",
    "checkver": {
        "github": "https://github.com/your-username/lc"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/your-username/lc/releases/download/v$version/lc-windows-amd64.exe"
            }
        }
    }
}
```

### Debian Package

Create `debian/control`:

```
Package: lc
Version: 1.0.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Your Name <your.email@example.com>
Description: Fast, Rust-based CLI for interacting with Large Language Models
 LLM Client (lc) is a command-line tool for interacting with
 Large Language Models through OpenAI-compatible APIs.
```

### AUR PKGBUILD

Create `aur/PKGBUILD`:

```bash
# Maintainer: Your Name <your.email@example.com>
pkgname=lc
pkgver=1.0.0
pkgrel=1
pkgdesc="Fast, Rust-based CLI for interacting with Large Language Models"
arch=('x86_64' 'aarch64')
url="https://lc.viwq.dev"
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/your-username/lc/archive/v$pkgver.tar.gz")
sha256sums=('YOUR_SHA256_HERE')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/lc" "$pkgdir/usr/bin/lc"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
```

## Testing Strategy

### Pre-release Testing

Create `.github/workflows/test-release.yml`:

```yaml
name: Test Release Builds

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  test-builds:
    strategy:
      matrix:
        os: [ubuntu-20.04, ubuntu-22.04, macos-12, macos-13, windows-2019, windows-2022]
        rust: [stable, 1.70.0]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo build --release
      - run: cargo test --release
      - run: ./target/release/lc --version
```

### Integration Tests

Create `tests/integration_test.sh`:

```bash
#!/bin/bash
set -e

echo "Testing lc installation and basic functionality..."

# Test version
lc --version

# Test help
lc --help

# Test provider commands
lc providers list

# Test configuration
lc config path

echo "All tests passed!"
```

## Release Checklist

### Before Release

- [ ] Update version in `Cargo.toml`
- [ ] Update CHANGELOG.md
- [ ] Run full test suite on all platforms
- [ ] Update documentation
- [ ] Create release branch
- [ ] Test package installations locally

### Release Steps

1. **Tag Release**:
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```

2. **Monitor GitHub Actions**:
   - Check release workflow
   - Verify all platform builds succeed
   - Ensure assets are uploaded

3. **Update Package Managers**:
   - Homebrew formula PR
   - Scoop manifest update
   - AUR package update
   - Debian package submission

4. **Announce Release**:
   - GitHub release notes
   - Documentation site update
   - Social media (if applicable)

### Post-Release

- [ ] Verify package manager installations work
- [ ] Monitor issue tracker for problems
- [ ] Update roadmap for next release

## Automation Tools

### Version Bumping

Create `scripts/bump-version.sh`:

```bash
#!/bin/bash
VERSION=$1

# Update Cargo.toml
sed -i "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update package files
sed -i "s/version\": \".*\"/version\": \"$VERSION\"/" scoop/lc.json
sed -i "s/version \".*\"/version \"$VERSION\"/" homebrew/lc.rb

# Commit changes
git add -A
git commit -m "Bump version to $VERSION"
```

### Release Script

Create `scripts/release.sh`:

```bash
#!/bin/bash
VERSION=$1

# Run tests
cargo test --release

# Bump version
./scripts/bump-version.sh $VERSION

# Create tag
git tag -a "v$VERSION" -m "Release version $VERSION"

# Push
git push origin main
git push origin "v$VERSION"

echo "Release $VERSION initiated!"
```

## Maintenance

### Regular Tasks

- **Weekly**: Check for dependency updates
- **Monthly**: Review and merge package manager PRs
- **Quarterly**: Major version planning

### Security Updates

- Monitor security advisories
- Automated dependency updates via Dependabot
- Quick patch releases for critical issues

## Success Metrics

- Installation success rate > 99%
- Package manager availability within 24h of release
- Cross-platform test coverage > 95%
- User-reported installation issues < 1%
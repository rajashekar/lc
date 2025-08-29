#!/usr/bin/env bash
set -eu

##############################################################################
# LLM Client (lc) Install Script
# 
# This script downloads the latest stable 'lc' CLI binary from GitHub releases
# and installs it to your system.
#
# Supported OS: macOS (darwin), Linux, Windows (MSYS2/Git Bash/WSL)
# Supported Architectures: x86_64, arm64
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/rajashekar/lc/main/install.sh | bash
#
# Environment variables:
#   LC_BIN_DIR - Directory to install lc (default: $HOME/.local/bin)
#   LC_VERSION - Specific version to install (e.g., "v0.1.2", defaults to latest)
#   LC_FORCE   - Set to "true" to overwrite existing installation
##############################################################################

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
GITHUB_REPO="rajashekar/lc"
DEFAULT_BIN_DIR="$HOME/.local/bin"
BIN_DIR="${LC_BIN_DIR:-$DEFAULT_BIN_DIR}"
VERSION="${LC_VERSION:-latest}"
FORCE="${LC_FORCE:-false}"

# Helper functions
info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}" >&2
}

# Check dependencies
check_deps() {
    local missing_deps=""
    
    if ! command -v curl >/dev/null 2>&1; then
        missing_deps="${missing_deps}curl "
    fi
    
    if ! command -v tar >/dev/null 2>&1 && ! command -v unzip >/dev/null 2>&1; then
        missing_deps="${missing_deps}tar/unzip "
    fi
    
    if [ -n "$missing_deps" ]; then
        error "Missing required dependencies: $missing_deps"
        error "Please install them and try again."
        exit 1
    fi
}

# Detect OS and architecture
detect_platform() {
    local os arch
    
    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)          
            error "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="arm64" ;;
        *)              
            error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    # Map to release naming convention
    case "$os-$arch" in
        "linux-x86_64")    PLATFORM="lc-linux-x86_64" ;;
        "linux-arm64")     PLATFORM="lc-linux-arm64" ;;
        "macos-x86_64")    PLATFORM="lc-macos-x86_64" ;;
        "macos-arm64")     PLATFORM="lc-macos-arm64" ;;
        "windows-x86_64")  PLATFORM="lc-windows-amd64" ;;
        *)                 
            error "No binary available for $os-$arch"
            exit 1
            ;;
    esac
    
    # Set file extension
    if [ "$os" = "windows" ]; then
        FILE_EXT=".zip"
        BINARY_NAME="lc.exe"
    else
        FILE_EXT=".tar.gz"
        BINARY_NAME="lc"
    fi
    
    info "Detected platform: $os-$arch"
}

# Get latest version from GitHub API
get_latest_version() {
    if [ "$VERSION" = "latest" ]; then
        info "Fetching latest version..."
        VERSION=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | \
                  grep -o '"tag_name": "[^"]*' | \
                  grep -o '[^"]*$')
        
        if [ -z "$VERSION" ]; then
            error "Failed to fetch latest version"
            exit 1
        fi
    fi
    
    info "Installing version: $VERSION"
}

# Create installation directory
setup_install_dir() {
    if [ ! -d "$BIN_DIR" ]; then
        info "Creating installation directory: $BIN_DIR"
        mkdir -p "$BIN_DIR"
    fi
}

# Check if lc is already installed
check_existing() {
    local existing_path
    existing_path=$(command -v lc 2>/dev/null || echo "")
    
    if [ -n "$existing_path" ] && [ "$FORCE" != "true" ]; then
        warn "lc is already installed at: $existing_path"
        warn "Use LC_FORCE=true to overwrite, or remove the existing installation first"
        exit 1
    fi
}

# Download and install lc
install_lc() {
    local download_url temp_dir archive_file
    
    download_url="https://github.com/$GITHUB_REPO/releases/download/$VERSION/$PLATFORM$FILE_EXT"
    temp_dir=$(mktemp -d)
    archive_file="$temp_dir/$PLATFORM$FILE_EXT"
    
    info "Downloading from: $download_url"
    
    # Download the archive
    if ! curl -fsSL "$download_url" -o "$archive_file"; then
        error "Failed to download lc binary"
        error "URL: $download_url"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # Extract the archive
    info "Extracting archive..."
    cd "$temp_dir"
    
    if [ "$FILE_EXT" = ".zip" ]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$archive_file"
        else
            error "unzip is required for Windows installation"
            rm -rf "$temp_dir"
            exit 1
        fi
    else
        tar -xzf "$archive_file"
    fi
    
    # Install the binary
    info "Installing lc to $BIN_DIR/$BINARY_NAME"
    
    if [ -f "$BINARY_NAME" ]; then
        cp "$BINARY_NAME" "$BIN_DIR/$BINARY_NAME"
        chmod +x "$BIN_DIR/$BINARY_NAME"
    else
        error "Binary not found in archive"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
}

# Verify installation
verify_installation() {
    local installed_path
    
    # Check if the binary exists and is executable
    if [ -x "$BIN_DIR/$BINARY_NAME" ]; then
        success "lc installed successfully at $BIN_DIR/$BINARY_NAME"
    else
        error "Installation verification failed"
        exit 1
    fi
    
    # Check if it's in PATH
    if installed_path=$(command -v lc 2>/dev/null); then
        success "lc is available in PATH: $installed_path"
        
        # Test basic functionality
        info "Testing installation..."
        if "$installed_path" --version >/dev/null 2>&1; then
            success "Installation test passed!"
        else
            warn "lc installed but --version test failed"
        fi
    else
        warn "lc installed but not found in PATH"
        warn "Add $BIN_DIR to your PATH or create a symlink:"
        info "  export PATH=\"$BIN_DIR:\$PATH\""
        info "  # Or add the above line to your shell profile (.bashrc, .zshrc, etc.)"
    fi
}

# Main installation flow
main() {
    echo "🚀 LLM Client (lc) Installer"
    echo "=============================="
    
    check_deps
    detect_platform
    get_latest_version
    setup_install_dir
    check_existing
    install_lc
    verify_installation
    
    echo
    success "Installation complete! 🎉"
    echo
    info "Get started with:"
    info "  lc --help"
    info "  lc providers add openai https://api.openai.com/v1"
    info "  lc keys add openai"
    echo
    info "Documentation: https://lc.viwq.dev"
}

# Run main function
main "$@"

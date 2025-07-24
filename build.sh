#!/bin/bash

# Build script for lc (LLM Client)

echo "Building lc..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed."
    echo "Please install Rust from https://rustup.rs/"
    echo "Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build in release mode
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "Binary location: target/release/lc"
    echo ""
    echo "To install globally, run:"
    echo "  sudo cp target/release/lc /usr/local/bin/"
    echo "  # or"
    echo "  cp target/release/lc ~/.local/bin/"
    echo ""
    echo "To test the build:"
    echo "  ./target/release/lc --help"
else
    echo "❌ Build failed!"
    exit 1
fi
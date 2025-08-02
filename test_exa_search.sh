#!/bin/bash

# Test script for Exa search integration

echo "Testing Exa search provider integration..."
echo

# Build the project first
echo "Building project..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# Add Exa provider
echo "1. Adding Exa search provider..."
./target/release/lc search provider add exa https://api.exa.ai -t exa
echo

# Set API key (you'll need to replace this with your actual API key)
echo "2. Setting Exa API key..."
echo "Please enter your Exa API key:"
read -s EXA_API_KEY
./target/release/lc search provider set exa x-api-key "$EXA_API_KEY"
echo

# List providers to verify
echo "3. Listing search providers..."
./target/release/lc search provider list
echo

# Test search query
echo "4. Testing search query..."
./target/release/lc search query exa "rust programming best practices" -f json -n 3
echo

# Test search with markdown format
echo "5. Testing search with markdown format..."
./target/release/lc search query exa "rust programming best practices" -f md -n 3
echo

# Test integration with prompt
echo "6. Testing search integration with prompt..."
./target/release/lc "What are the best practices for Rust programming?" --use-search exa
echo

echo "Exa search provider testing complete!"
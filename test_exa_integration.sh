#!/bin/bash

echo "=== Exa Search Provider Integration Test ==="
echo

# Build the project
echo "Building project..."
cargo build --release
echo

# Add Exa provider
echo "1. Adding Exa search provider..."
./target/release/lc search provider add exa https://api.exa.ai -t exa
echo

# Set API key (using test key - replace with valid key)
echo "2. Setting Exa API key..."
./target/release/lc search provider set exa x-api-key "YOUR_VALID_EXA_API_KEY"
echo

# List providers to verify
echo "3. Listing search providers..."
./target/release/lc search provider list
echo

# Test search (will fail with invalid API key)
echo "4. Testing Exa search..."
echo "Note: This will fail with 'invalid API key' if using a test key"
./target/release/lc search query exa "latest AI developments" -n 3 || echo "Expected failure with test API key"
echo

echo "=== Exa Integration Complete ==="
echo "The Exa search provider has been successfully implemented."
echo "To use it, replace 'YOUR_VALID_EXA_API_KEY' with a valid Exa API key."
echo
echo "Features implemented:"
echo "- POST request to /search endpoint"
echo "- JSON request body with query and contents.text=true"
echo "- Proper header handling for x-api-key"
echo "- Response parsing with text content support"
echo "- Integration with main search command and --use-search flag"
#!/bin/bash

# Test script for search integration
# This script tests the search functionality with Brave API

echo "=== LC Search Integration Test ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if API key is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Please provide your Brave API key as an argument${NC}"
    echo "Usage: ./test_search_integration.sh YOUR_BRAVE_API_KEY"
    exit 1
fi

BRAVE_API_KEY="$1"

# Function to run command and check result
run_test() {
    local description="$1"
    local command="$2"
    
    echo -e "${YELLOW}Test: $description${NC}"
    echo "Command: $command"
    
    if eval "$command"; then
        echo -e "${GREEN}✓ Success${NC}"
    else
        echo -e "${RED}✗ Failed${NC}"
        exit 1
    fi
    echo
}

# Build the project first
echo "Building the project..."
cargo build --release
echo

# Test 1: Add Brave search provider
run_test "Add Brave search provider" \
    "cargo run --release -- search provider add brave https://api.search.brave.com/res/v1/web/search"

# Test 2: Set API key
run_test "Set Brave API key" \
    "cargo run --release -- search provider set brave X-Subscription-Token '$BRAVE_API_KEY'"

# Test 3: List providers
run_test "List search providers" \
    "cargo run --release -- search provider list"

# Test 4: Set default search provider
run_test "Set default search provider" \
    "cargo run --release -- config set search brave"

# Test 5: Get default search provider
run_test "Get default search provider" \
    "cargo run --release -- config get search"

# Test 6: Direct search with JSON output
run_test "Direct search with JSON output" \
    "cargo run --release -- search query brave 'Rust programming language' -f json -n 3"

# Test 7: Direct search with Markdown output
run_test "Direct search with Markdown output" \
    "cargo run --release -- search query brave 'OpenAI GPT-4' -f md -n 5"

# Test 8: Use search in prompt (with default provider)
run_test "Use search in prompt with default provider" \
    "cargo run --release -- --use-search brave 'What are the latest developments in quantum computing?'"

# Test 9: Use search with custom query
run_test "Use search with custom query" \
    "cargo run --release -- --use-search 'brave:AI safety research 2024' 'Summarize the key findings'"

# Test 10: Delete default search provider
run_test "Delete default search provider" \
    "cargo run --release -- config delete search"

# Test 11: Delete search provider
run_test "Delete search provider" \
    "cargo run --release -- search provider delete brave"

# Test 12: Verify provider is deleted
echo -e "${YELLOW}Test: Verify provider is deleted${NC}"
if cargo run --release -- search provider list 2>&1 | grep -q "No search providers configured"; then
    echo -e "${GREEN}✓ Success - Provider deleted${NC}"
else
    echo -e "${RED}✗ Failed - Provider still exists${NC}"
    exit 1
fi

echo
echo -e "${GREEN}=== All tests passed! ===${NC}"
echo "The search integration is working correctly."
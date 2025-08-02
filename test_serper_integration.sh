#!/bin/bash

# Test script for Serper search provider integration

echo "=== Testing Serper Search Provider Integration ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if API key is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Please provide Serper API key as argument${NC}"
    echo "Usage: $0 <serper-api-key>"
    exit 1
fi

SERPER_API_KEY=$1

echo "1. Adding Serper search provider..."
./target/debug/lc search provider add serper https://google.serper.dev -t serper
echo

echo "2. Setting API key for Serper..."
./target/debug/lc search provider set serper X-API-KEY "$SERPER_API_KEY"
echo

echo "3. Listing search providers..."
./target/debug/lc search provider list
echo

echo "4. Testing search with JSON output..."
echo -e "${YELLOW}Command: lc search query serper \"rust programming\" -f json${NC}"
./target/debug/lc search query serper "rust programming" -f json
echo

echo "5. Testing search with Markdown output..."
echo -e "${YELLOW}Command: lc search query serper \"rust programming\" -f md${NC}"
./target/debug/lc search query serper "rust programming" -f md
echo

echo "6. Testing search with custom result count..."
echo -e "${YELLOW}Command: lc search query serper \"OpenAI GPT-4\" -n 3${NC}"
./target/debug/lc search query serper "OpenAI GPT-4" -n 3
echo

echo "7. Setting Serper as default search provider..."
./target/debug/lc config set search serper
echo

echo "8. Testing search integration with prompt..."
echo -e "${YELLOW}Command: lc \"What is Rust programming language?\" --use-search serper${NC}"
./target/debug/lc "What is Rust programming language?" --use-search serper
echo

echo "9. Testing search with custom query..."
echo -e "${YELLOW}Command: lc \"Tell me about AI\" --use-search \"serper:artificial intelligence latest news\"${NC}"
./target/debug/lc "Tell me about AI" --use-search "serper:artificial intelligence latest news"
echo

echo -e "${GREEN}=== Serper Integration Test Complete ===${NC}"
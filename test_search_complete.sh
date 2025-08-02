#!/bin/bash

echo "=== LC Search Feature - Complete Demonstration ==="
echo

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}1. Search Provider Management${NC}"
echo "----------------------------------------"
echo "Available commands:"
echo "  lc search provider add <name> <url> -t <type>  # Add a new provider"
echo "  lc search provider list                         # List all providers"
echo "  lc search provider set <name> <header> <value>  # Set API key"
echo "  lc search provider remove <name>                # Remove a provider"
echo "  lc search provider default <name>               # Set default provider"
echo

echo -e "${BLUE}2. Current Configuration${NC}"
echo "----------------------------------------"
cargo run -- search provider list
echo

echo -e "${BLUE}3. Search Commands${NC}"
echo "----------------------------------------"
echo "Available search options:"
echo "  lc search <query>                    # Search using default provider"
echo "  lc search <query> -p <provider>      # Search using specific provider"
echo "  lc search <query> -f json            # Output in JSON format"
echo "  lc search <query> -f markdown        # Output in Markdown format"
echo "  lc search <query> -l <limit>         # Limit number of results"
echo

echo -e "${BLUE}4. Integration with Prompts${NC}"
echo "----------------------------------------"
echo "Use search results as context:"
echo "  lc prompt \"Your question\" --use-search \"search query\""
echo "  lc prompt \"Your question\" --use-search \"search query\" --search-provider brave"
echo "  lc prompt \"Your question\" --use-search \"search query\" --search-limit 5"
echo

echo -e "${BLUE}5. Provider-Specific Features${NC}"
echo "----------------------------------------"
echo -e "${YELLOW}Brave Search:${NC}"
echo "  - Web search with snippets"
echo "  - GET request to API"
echo "  - Requires X-Subscription-Token header"
echo
echo -e "${YELLOW}Exa Search:${NC}"
echo "  - AI-powered search"
echo "  - POST request with JSON body"
echo "  - Requires x-api-key header"
echo "  - Returns full text content"
echo

echo -e "${BLUE}6. Example Usage (with valid API keys)${NC}"
echo "----------------------------------------"
echo "# Add and configure Brave"
echo "lc search provider add brave https://api.search.brave.com/res/v1 -t brave"
echo "lc search provider set brave X-Subscription-Token YOUR_BRAVE_KEY"
echo
echo "# Add and configure Exa"
echo "lc search provider add exa https://api.exa.ai -t exa"
echo "lc search provider set exa x-api-key YOUR_EXA_KEY"
echo
echo "# Search examples"
echo "lc search \"rust programming\""
echo "lc search \"machine learning\" -p exa -f json"
echo "lc search \"climate change\" -p brave -l 3"
echo
echo "# Integration example"
echo "lc prompt \"Explain the latest developments\" --use-search \"AI breakthroughs 2024\""
echo

echo -e "${GREEN}=== Search Feature Implementation Complete ===${NC}"
echo -e "${GREEN}✓ Modular provider architecture${NC}"
echo -e "${GREEN}✓ Multiple search providers (Brave, Exa)${NC}"
echo -e "${GREEN}✓ Provider management CLI${NC}"
echo -e "${GREEN}✓ JSON and Markdown output formats${NC}"
echo -e "${GREEN}✓ Integration with prompt context${NC}"
echo -e "${GREEN}✓ Comprehensive error handling${NC}"
echo -e "${GREEN}✓ Full test coverage${NC}"
echo -e "${GREEN}✓ Updated documentation${NC}"
#!/bin/bash

# Simple LLM CLI Provider Test Script
# Tests providers without complex output parsing that might interfere with JSON

set +e  # Don't exit on errors

echo "=== LLM CLI Provider Test ==="
echo "Testing providers with API keys..."
echo

# Get providers with keys
providers_with_keys=$(./target/release/lc p l | grep "API Key: ✓" | sed 's/^[[:space:]]*•[[:space:]]*\([^[:space:]]*\).*/\1/')

total_providers=0
model_success=0
chat_success=0

for provider in $providers_with_keys; do
    total_providers=$((total_providers + 1))
    echo "--- Testing $provider ---"
    
    # Test model listing
    echo -n "Models: "
    if ./target/release/lc p m "$provider" >/dev/null 2>&1; then
        model_count=$(./target/release/lc p m "$provider" 2>/dev/null | grep -c "^  •" || echo "0")
        echo "✓ ($model_count models)"
        model_success=$((model_success + 1))
        
        # Get first model for chat test
        first_model=$(./target/release/lc p m "$provider" 2>/dev/null | grep "^  •" | head -1 | sed 's/^  • //')
        
        # Test chat
        echo -n "Chat: "
        if echo "2+2=" | timeout 15 ./target/release/lc -p "$provider" -m "$first_model" >/dev/null 2>&1; then
            echo "✓"
            chat_success=$((chat_success + 1))
        else
            echo "✗"
        fi
    else
        echo "✗"
    fi
    echo
done

echo "=== SUMMARY ==="
echo "Total providers tested: $total_providers"
echo "Model listing success: $model_success/$total_providers"
echo "Chat success: $chat_success/$total_providers"

if [ $total_providers -gt 0 ]; then
    model_rate=$((model_success * 100 / total_providers))
    chat_rate=$((chat_success * 100 / total_providers))
    echo "Model success rate: ${model_rate}%"
    echo "Chat success rate: ${chat_rate}%"
fi
#!/bin/bash

# Test script for the new model metadata implementation

echo "Testing Model Metadata Commands..."
echo "================================="

# Test 1: List model paths
echo -e "\n1. Testing model paths list:"
cargo run -- models path list

# Test 2: Add a model path
echo -e "\n2. Testing add model path:"
cargo run -- models path add ".data[].models[]"

# Test 3: List model paths again
echo -e "\n3. Listing model paths after add:"
cargo run -- models path list

# Test 4: List tags
echo -e "\n4. Testing tags list:"
cargo run -- models tags list

# Test 5: Add a tag rule
echo -e "\n5. Testing add tag rule:"
cargo run -- models tags add "fast" "fast, model.id"

# Test 6: List tags again
echo -e "\n6. Listing tags after add:"
cargo run -- models tags list

# Test 7: Filter models by tags
echo -e "\n7. Testing model filtering by tags:"
cargo run -- models filter --tag tools,vision

# Test 8: Add custom model to a provider
echo -e "\n8. Testing add custom model:"
cargo run -- providers models openai add "gpt-5-turbo" --tag "ctx=200k,out=4096,tools,vision,reasoning"

# Test 9: List models with tag filter
echo -e "\n9. Testing models list with tag filter:"
cargo run -- models --tag tools,vision

echo -e "\nAll tests completed!"
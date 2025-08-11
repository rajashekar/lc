#!/bin/bash

echo "=== Shell Completion Setup and Test Script ==="
echo ""

# Detect shell
SHELL_NAME=$(basename "$SHELL")
echo "Detected shell: $SHELL_NAME"
echo ""

# Generate completion for the current shell
echo "Generating completion script for $SHELL_NAME..."
case "$SHELL_NAME" in
    "zsh")
        echo "Generating Zsh completion..."
        cargo run --quiet -- completions zsh > /tmp/lc_completion_test.zsh 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "✓ Zsh completion generated successfully"
            echo ""
            echo "To test the completion:"
            echo "1. Source the completion script:"
            echo "   source /tmp/lc_completion_test.zsh"
            echo ""
            echo "2. Test basic completion:"
            echo "   lc <TAB><TAB>"
            echo ""
            echo "3. Test providers completion:"
            echo "   lc providers <TAB><TAB>"
            echo ""
            echo "4. Test alias completion:"
            echo "   lc p <TAB><TAB>"
            echo ""
            echo "5. Test providers models completion:"
            echo "   lc p m <TAB><TAB>"
            echo ""
            echo "To make it permanent, add this to your ~/.zshrc:"
            echo "eval \"\$(lc completions zsh)\""
        else
            echo "✗ Failed to generate Zsh completion"
        fi
        ;;
    "bash")
        echo "Generating Bash completion..."
        cargo run --quiet -- completions bash > /tmp/lc_completion_test.bash 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "✓ Bash completion generated successfully"
            echo ""
            echo "To test the completion:"
            echo "1. Source the completion script:"
            echo "   source /tmp/lc_completion_test.bash"
            echo ""
            echo "2. Test basic completion:"
            echo "   lc <TAB><TAB>"
            echo ""
            echo "3. Test providers completion:"
            echo "   lc providers <TAB><TAB>"
            echo ""
            echo "4. Test alias completion:"
            echo "   lc p <TAB><TAB>"
            echo ""
            echo "To make it permanent, add this to your ~/.bashrc:"
            echo "eval \"\$(lc completions bash)\""
        else
            echo "✗ Failed to generate Bash completion"
        fi
        ;;
    *)
        echo "Unsupported shell: $SHELL_NAME"
        echo "Supported shells: bash, zsh, fish, powershell, elvish"
        ;;
esac

echo ""
echo "=== Debugging Information ==="
echo "Current working directory: $(pwd)"
echo "lc binary location: $(which lc 2>/dev/null || echo 'lc not found in PATH')"
echo ""

# Check if lc is working
echo "Testing lc command..."
if command -v lc >/dev/null 2>&1; then
    echo "✓ lc command is available"
    echo "lc version: $(lc --version 2>/dev/null || echo 'version not available')"
else
    echo "✗ lc command not found in PATH"
    echo "You may need to install lc first with: cargo install --path ."
fi

echo ""
echo "=== Next Steps ==="
echo "1. Make sure lc is installed and in your PATH"
echo "2. Source the generated completion script"
echo "3. Test the completion as shown above"
echo "4. If it works, add the eval line to your shell's rc file"
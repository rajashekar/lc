#!/bin/bash

# LC CLI Deployment Script
# This script safely deploys the LC CLI binary to the system

set -e

echo "🔧 Building LC CLI in release mode..."
cargo build --release

echo "🔍 Checking for running LC processes..."
RUNNING_PROCESSES=$(ps -ef | grep "/opt/homebrew/bin/lc" | grep -v grep | wc -l)

if [ "$RUNNING_PROCESSES" -gt 0 ]; then
    echo "⚠️  Found running LC processes. Stopping them first..."
    ps -ef | grep "/opt/homebrew/bin/lc" | grep -v grep | awk '{print $2}' | xargs kill
    echo "✅ Stopped running processes"
    sleep 1
fi

echo "📦 Installing LC CLI to /opt/homebrew/bin..."
sudo cp target/release/lc /opt/homebrew/bin/lc

echo "🔐 Setting correct permissions..."
sudo chmod +x /opt/homebrew/bin/lc

echo "✅ Testing installation..."
if lc --version > /dev/null 2>&1; then
    echo "🎉 LC CLI successfully deployed!"
    lc --version
else
    echo "❌ Installation test failed"
    exit 1
fi

echo ""
echo "📝 Deployment complete! You can now use 'lc' from anywhere."
echo "💡 To restart any web chat proxy daemons, use: lc w start <provider>"
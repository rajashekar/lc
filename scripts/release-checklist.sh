#!/bin/bash
set -e

# Release checklist script
# This script guides through the release process

echo "🚀 LLM Client Release Checklist"
echo "==============================="
echo ""

# Function to ask yes/no questions
confirm() {
    read -p "$1 (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Aborted. Please complete this step before continuing."
        exit 1
    fi
}

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $CURRENT_VERSION"
echo ""

# Ask for new version
read -p "Enter new version: " NEW_VERSION
echo ""

echo "📋 Pre-release Checklist"
echo "------------------------"

confirm "1. Have you updated CHANGELOG.md with all changes?"
confirm "2. Have you run 'cargo test' and all tests pass?"
confirm "3. Have you run 'cargo clippy' with no warnings?"
confirm "4. Have you run 'cargo fmt' to format code?"
confirm "5. Have you tested on macOS?"
confirm "6. Have you tested on Linux?"
confirm "7. Have you tested on Windows?"
confirm "8. Have you updated documentation if needed?"
confirm "9. Are all CI checks passing on main branch?"

echo ""
echo "✅ Pre-release checks complete!"
echo ""

echo "📦 Preparing Release"
echo "-------------------"

# Bump version
echo "Bumping version to $NEW_VERSION..."
./scripts/bump-version.sh $NEW_VERSION

# Show changes
echo ""
echo "📝 Changes to be committed:"
git diff --stat

echo ""
confirm "Do the changes look correct?"

# Commit changes
echo "Committing version bump..."
git add -A
git commit -m "Bump version to $NEW_VERSION"

# Create tag
echo "Creating tag v$NEW_VERSION..."
git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"

echo ""
echo "🎯 Release Prepared!"
echo "-------------------"
echo ""
echo "Next steps:"
echo "1. Push changes: git push origin main"
echo "2. Push tag: git push origin v$NEW_VERSION"
echo "3. GitHub Actions will automatically:"
echo "   - Build binaries for all platforms"
echo "   - Create GitHub release"
echo "   - Publish to crates.io"
echo "   - Update Homebrew formula"
echo ""
echo "4. After release, update package managers:"
echo "   - Scoop: Update manifest in scoop bucket"
echo "   - AUR: Update PKGBUILD"
echo "   - Debian/Ubuntu: Submit to package maintainers"
echo ""
echo "5. Announce release:"
echo "   - Update documentation site"
echo "   - Post release notes"
echo ""

confirm "Ready to push? (This will trigger the release)"

# Push
echo "Pushing to GitHub..."
git push origin main
git push origin "v$NEW_VERSION"

echo ""
echo "🎉 Release v$NEW_VERSION initiated!"
echo ""
echo "Monitor the release at:"
echo "https://github.com/your-username/lc/actions"
echo ""
echo "Once complete, the release will be available at:"
echo "https://github.com/your-username/lc/releases/tag/v$NEW_VERSION"
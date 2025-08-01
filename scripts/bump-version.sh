#!/bin/bash
set -e

# Script to bump version across all files
# Usage: ./scripts/bump-version.sh 1.2.3

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.2.3"
    exit 1
fi

echo "Bumping version to $VERSION..."

# Update Cargo.toml
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
fi

# Update Cargo.lock
cargo update --workspace

# Update documentation if needed
if [ -f "docs-site/package.json" ]; then
    cd docs-site
    npm version $VERSION --no-git-tag-version
    cd ..
fi

echo "Version bumped to $VERSION"
echo ""
echo "Next steps:"
echo "1. Review changes: git diff"
echo "2. Commit: git commit -am \"Bump version to $VERSION\""
echo "3. Tag: git tag -a v$VERSION -m \"Release version $VERSION\""
echo "4. Push: git push && git push --tags"
#!/bin/bash
set -e

# Release script for dupfind
# Usage: ./scripts/release.sh v0.1.0

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.1.0"
    exit 1
fi

if ! [[ "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format v0.0.0"
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean"
    exit 1
fi

# Update version in Cargo.toml
SEMVER=${VERSION#v}
sed -i.bak "s/^version = \".*\"/version = \"$SEMVER\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
cargo check --quiet

# Generate changelog
if command -v git-cliff &> /dev/null; then
    git-cliff --tag "$VERSION" -o CHANGELOG.md
    echo "Generated CHANGELOG.md"
fi

# Commit and tag
git add Cargo.toml Cargo.lock CHANGELOG.md 2>/dev/null || git add Cargo.toml Cargo.lock
git commit -m "chore(release): $VERSION"
git tag -a "$VERSION" -m "Release $VERSION"

echo ""
echo "Release $VERSION prepared!"
echo "Run 'git push && git push --tags' to trigger the release workflow"


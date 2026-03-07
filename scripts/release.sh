#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

echo "=== AgenticForge Release $VERSION ==="

# Ensure clean working tree
if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working tree is not clean. Commit or stash changes first."
    exit 1
fi

# Run tests
echo "Running tests..."
cargo test --workspace

# Run guardrails
echo "Running guardrails..."
bash scripts/check-canonical-consistency.sh
bash scripts/check-command-surface.sh
bash scripts/check-mcp-consolidation.sh

# Build release
echo "Building release..."
cargo build --workspace --release

# Update version in Cargo.toml files
echo "Updating version to $VERSION..."
for toml in Cargo.toml crates/*/Cargo.toml; do
    if [ -f "$toml" ]; then
        sed -i '' "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" "$toml" 2>/dev/null || true
    fi
done

echo "=== Release $VERSION prepared ==="
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git commit -am 'chore: release v$VERSION'"
echo "  3. Tag: git tag v$VERSION"
echo "  4. Push: git push && git push --tags"

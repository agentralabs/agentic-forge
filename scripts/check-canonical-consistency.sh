#!/usr/bin/env bash
set -euo pipefail

FAIL=0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Canonical Consistency Check ==="

# Check sister.manifest.json exists
if [ ! -f "$ROOT/sister.manifest.json" ]; then
    echo "FAIL: sister.manifest.json not found"
    FAIL=1
else
    echo "PASS: sister.manifest.json exists"
fi

# Check CLAUDE.md exists
if [ ! -f "$ROOT/CLAUDE.md" ]; then
    echo "FAIL: CLAUDE.md not found"
    FAIL=1
else
    echo "PASS: CLAUDE.md exists"
fi

# Check 4-crate workspace structure
for crate in agentic-forge-core agentic-forge-mcp agentic-forge-cli agentic-forge-ffi; do
    if [ ! -d "$ROOT/crates/$crate" ]; then
        echo "FAIL: crates/$crate not found"
        FAIL=1
    else
        echo "PASS: crates/$crate exists"
    fi
done

# Check version consistency
MANIFEST_VERSION=$(grep '"version"' "$ROOT/sister.manifest.json" | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')
CARGO_VERSION=$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
if [ "$MANIFEST_VERSION" != "$CARGO_VERSION" ]; then
    echo "FAIL: Version mismatch: manifest=$MANIFEST_VERSION cargo=$CARGO_VERSION"
    FAIL=1
else
    echo "PASS: Version consistent ($CARGO_VERSION)"
fi

# Check invention count consistency
MANIFEST_INVENTIONS=$(grep '"invention_count"' "$ROOT/sister.manifest.json" | sed 's/[^0-9]//g')
CODE_INVENTIONS=$(grep -r 'INVENTION_COUNT.*=.*[0-9]' "$ROOT/crates/agentic-forge-core/src/" | head -1 | sed 's/.*= *\([0-9]*\).*/\1/')
if [ "$MANIFEST_INVENTIONS" != "$CODE_INVENTIONS" ]; then
    echo "FAIL: Invention count mismatch: manifest=$MANIFEST_INVENTIONS code=$CODE_INVENTIONS"
    FAIL=1
else
    echo "PASS: Invention count consistent ($CODE_INVENTIONS)"
fi

# Section 47: Standard reference doc pages (mandatory)
REQUIRED_DOCS="architecture.md cli-reference.md configuration.md ffi-reference.md mcp-tools.md mcp-resources.md mcp-prompts.md troubleshooting.md"
for doc in $REQUIRED_DOCS; do
    if [ ! -f "$ROOT/docs/public/$doc" ]; then
        echo "FAIL: docs/public/$doc not found (Section 47 mandatory)"
        FAIL=1
    else
        echo "PASS: docs/public/$doc exists"
    fi
done

# Check baseline docs
BASELINE_DOCS="quickstart.md concepts.md integration-guide.md faq.md benchmarks.md api-reference.md"
for doc in $BASELINE_DOCS; do
    if [ ! -f "$ROOT/docs/public/$doc" ]; then
        echo "FAIL: docs/public/$doc not found (baseline)"
        FAIL=1
    else
        echo "PASS: docs/public/$doc exists"
    fi
done

# Check 4 SVG design assets
SVG_COUNT=$(find "$ROOT/docs/public" -name "*.svg" 2>/dev/null | wc -l | tr -d ' ')
if [ "$SVG_COUNT" -lt 4 ]; then
    echo "FAIL: Only $SVG_COUNT SVGs found in docs/public/ (need 4)"
    FAIL=1
else
    echo "PASS: $SVG_COUNT SVG design assets found"
fi

# Check CI workflows
for workflow in ci.yml canonical.yml install-test.yml; do
    if [ ! -f "$ROOT/.github/workflows/$workflow" ]; then
        echo "FAIL: .github/workflows/$workflow not found"
        FAIL=1
    else
        echo "PASS: .github/workflows/$workflow exists"
    fi
done

if [ $FAIL -ne 0 ]; then
    echo "=== FAILED ==="
    exit 1
fi
echo "=== ALL PASSED ==="

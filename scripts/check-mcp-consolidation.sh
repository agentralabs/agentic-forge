#!/usr/bin/env bash
set -eu

FAIL=0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MCP_DIR="$ROOT/crates/agentic-forge-mcp/src"

echo "=== MCP Consolidation Check ==="

# Check no .unwrap() in MCP code (excluding tests)
UNWRAP_COUNT=0
while IFS= read -r line; do
    UNWRAP_COUNT=$((UNWRAP_COUNT + 1))
done < <(grep -rn '\.unwrap()' "$MCP_DIR" --include='*.rs' 2>/dev/null | grep -v 'mod tests' | grep -v '#\[test\]' | grep -v '#\[cfg(test)\]' || true)

if [ "$UNWRAP_COUNT" -gt 0 ]; then
    echo "FAIL: Found $UNWRAP_COUNT .unwrap() calls in MCP production code"
    FAIL=1
else
    echo "PASS: No .unwrap() in MCP production code"
fi

# Check tool descriptions are verb-first (no trailing periods)
REGISTRY="$MCP_DIR/tools/registry.rs"
if [ -f "$REGISTRY" ]; then
    TRAILING=$(grep -c 'description.*\."' "$REGISTRY" 2>/dev/null || true)
    TRAILING=${TRAILING:-0}
    if [ "$TRAILING" -gt 0 ]; then
        echo "FAIL: Found $TRAILING tool descriptions with trailing periods"
        FAIL=1
    else
        echo "PASS: No trailing periods in tool descriptions"
    fi
fi

# Check error code -32803 for unknown tools
if grep -q '32803' "$MCP_DIR/types/error.rs" 2>/dev/null; then
    echo "PASS: TOOL_NOT_FOUND error code -32803 present"
else
    echo "FAIL: No -32803 TOOL_NOT_FOUND error code found"
    FAIL=1
fi

# Check all tools are registered
REGISTERED=$(grep -c '"forge_' "$REGISTRY" 2>/dev/null || echo "0")
echo "INFO: $REGISTERED tool name references in registry"

if [ $FAIL -ne 0 ]; then
    echo "=== FAILED ==="
    exit 1
fi
echo "=== ALL PASSED ==="

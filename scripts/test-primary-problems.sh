#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0

echo "=== Primary Problems Test ==="

# Problem 1: Core types compile
echo ""
echo "--- Problem 1: Core types ---"
if cargo check -p agentic-forge-core --manifest-path "$ROOT/Cargo.toml" 2>/dev/null; then
    echo "PASS: Core crate compiles"
else
    echo "FAIL: Core crate does not compile"
    FAIL=1
fi

# Problem 2: MCP tools are defined
echo ""
echo "--- Problem 2: MCP tool definitions ---"
TOOL_COUNT=$(grep -c '"forge_' "$ROOT/crates/agentic-forge-mcp/src/tools/registry.rs" 2>/dev/null || echo 0)
if [ "$TOOL_COUNT" -ge 15 ]; then
    echo "PASS: $TOOL_COUNT tool references found (>= 15)"
else
    echo "FAIL: Only $TOOL_COUNT tool references found (need 15)"
    FAIL=1
fi

# Problem 3: CLI subcommands are defined
echo ""
echo "--- Problem 3: CLI subcommands ---"
CMD_COUNT=$(grep -c 'Commands::' "$ROOT/crates/agentic-forge-cli/src/main.rs" 2>/dev/null || echo 0)
if [ "$CMD_COUNT" -ge 10 ]; then
    echo "PASS: $CMD_COUNT command matches found"
else
    echo "FAIL: Only $CMD_COUNT command matches found"
    FAIL=1
fi

# Problem 4: Inventions are registered
echo ""
echo "--- Problem 4: Invention registry ---"
INV_FILE="$ROOT/crates/agentic-forge-core/src/inventions/mod.rs"
if grep -q 'INVENTION_COUNT.*32' "$INV_FILE" 2>/dev/null; then
    echo "PASS: 32 inventions registered"
else
    echo "FAIL: Invention count not 32"
    FAIL=1
fi

# Problem 5: FFI exports
echo ""
echo "--- Problem 5: FFI exports ---"
FFI_EXPORTS=$(grep -c '#\[no_mangle\]' "$ROOT/crates/agentic-forge-ffi/src/lib.rs" 2>/dev/null || echo 0)
if [ "$FFI_EXPORTS" -ge 3 ]; then
    echo "PASS: $FFI_EXPORTS FFI exports found"
else
    echo "FAIL: Only $FFI_EXPORTS FFI exports found (need >= 3)"
    FAIL=1
fi

# Problem 6: Tests exist
echo ""
echo "--- Problem 6: Test files ---"
TEST_FILES=$(find "$ROOT/crates" -name '*.rs' -path '*/tests/*' 2>/dev/null | wc -l | tr -d ' ')
if [ "$TEST_FILES" -ge 1 ]; then
    echo "PASS: $TEST_FILES test files found"
else
    echo "FAIL: No test files found"
    FAIL=1
fi

echo ""
if [ $FAIL -ne 0 ]; then
    echo "=== FAILED ==="
    exit 1
fi
echo "=== ALL PASSED ==="

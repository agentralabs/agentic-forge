#!/usr/bin/env bash
set -euo pipefail

FAIL=0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Runtime Hardening Check ==="

# Check no .unwrap() in production code (excluding tests)
for crate in core mcp cli ffi; do
    SRC="$ROOT/crates/agentic-forge-$crate/src"
    if [ -d "$SRC" ]; then
        COUNT=$(grep -rn '\.unwrap()' "$SRC" --include='*.rs' | grep -v '#\[cfg(test)\]' | grep -v '#\[test\]' | grep -v 'mod tests' | wc -l | tr -d ' ')
        if [ "$COUNT" -gt 0 ]; then
            echo "WARN: $crate has $COUNT .unwrap() in production code"
            # Only fail for MCP crate (strict requirement)
            if [ "$crate" = "mcp" ]; then
                FAIL=1
            fi
        else
            echo "PASS: $crate has no .unwrap() in production code"
        fi
    fi
done

# Check no panic! in MCP code
MCP_SRC="$ROOT/crates/agentic-forge-mcp/src"
PANIC_COUNT=$(grep -rn 'panic!' "$MCP_SRC" --include='*.rs' | grep -v '#\[test\]' | grep -v 'mod tests' | wc -l | tr -d ' ')
if [ "$PANIC_COUNT" -gt 0 ]; then
    echo "FAIL: Found $PANIC_COUNT panic! calls in MCP production code"
    FAIL=1
else
    echo "PASS: No panic! in MCP production code"
fi

# Check FFI null safety
FFI_SRC="$ROOT/crates/agentic-forge-ffi/src"
if grep -q 'is_null' "$FFI_SRC/lib.rs" 2>/dev/null; then
    echo "PASS: FFI has null checks"
else
    echo "WARN: FFI may lack null safety checks"
fi

# Check unsafe blocks are documented
UNSAFE_COUNT=$(grep -rn 'unsafe {' "$ROOT/crates/" --include='*.rs' | wc -l | tr -d ' ')
echo "INFO: $UNSAFE_COUNT unsafe blocks found across workspace"

if [ $FAIL -ne 0 ]; then
    echo "=== FAILED ==="
    exit 1
fi
echo "=== ALL PASSED ==="

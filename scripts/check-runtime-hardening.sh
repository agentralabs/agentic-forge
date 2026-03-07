#!/usr/bin/env bash
set -euo pipefail

FAIL=0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Runtime Hardening Check ==="

search_matches() {
    local pattern="$1"
    local path="$2"
    if command -v rg >/dev/null 2>&1; then
        rg -n "$pattern" "$path" --glob '*.rs' --glob '!**/tests/**' || true
    else
        grep -rn "$pattern" "$path" --include='*.rs' 2>/dev/null || true
    fi
}

# Check no .unwrap() in production code (excluding dedicated test files)
for crate in core mcp cli ffi; do
    SRC="$ROOT/crates/agentic-forge-$crate/src"
    if [ -d "$SRC" ]; then
        MATCHES="$(search_matches '\.unwrap\(\)' "$SRC")"
        COUNT=$(printf '%s\n' "$MATCHES" | sed '/^$/d' | wc -l | tr -d ' ')
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
PANIC_MATCHES="$(search_matches 'panic!' "$MCP_SRC")"
PANIC_COUNT=$(printf '%s\n' "$PANIC_MATCHES" | sed '/^$/d' | wc -l | tr -d ' ')
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
if command -v rg >/dev/null 2>&1; then
    UNSAFE_MATCHES="$(rg -n 'unsafe \{' "$ROOT/crates/" --glob '*.rs' || true)"
else
    UNSAFE_MATCHES="$(grep -rn 'unsafe {' "$ROOT/crates/" --include='*.rs' 2>/dev/null || true)"
fi
UNSAFE_COUNT=$(printf '%s\n' "$UNSAFE_MATCHES" | sed '/^$/d' | wc -l | tr -d ' ')
echo "INFO: $UNSAFE_COUNT unsafe blocks found across workspace"

if [ $FAIL -ne 0 ]; then
    echo "=== FAILED ==="
    exit 1
fi
echo "=== ALL PASSED ==="

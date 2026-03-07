#!/usr/bin/env bash
set -euo pipefail

FAIL=0

echo "=== Install Commands Verification ==="

# Check cargo is available
if command -v cargo >/dev/null 2>&1; then
    echo "PASS: cargo found ($(cargo --version))"
else
    echo "FAIL: cargo not found"
    FAIL=1
fi

# Check rustc version
if command -v rustc >/dev/null 2>&1; then
    RUST_VER=$(rustc --version)
    echo "PASS: rustc found ($RUST_VER)"
else
    echo "FAIL: rustc not found"
    FAIL=1
fi

# Check workspace builds
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if cargo check --workspace --manifest-path "$ROOT/Cargo.toml" 2>/dev/null; then
    echo "PASS: Workspace compiles"
else
    echo "FAIL: Workspace does not compile"
    FAIL=1
fi

# Check aforge binary exists after build
if [ -f "$ROOT/target/debug/aforge" ] || [ -f "$ROOT/target/release/aforge" ]; then
    echo "PASS: aforge binary found"
elif command -v aforge >/dev/null 2>&1; then
    echo "PASS: aforge installed in PATH"
else
    echo "INFO: aforge binary not yet built (run cargo build first)"
fi

if [ $FAIL -ne 0 ]; then
    echo "=== FAILED ==="
    exit 1
fi
echo "=== ALL PASSED ==="

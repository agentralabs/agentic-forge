#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0

echo "=== Full Canonical Sister Validation ==="

# Run all sub-checks
for check in check-canonical-consistency.sh check-command-surface.sh check-mcp-consolidation.sh; do
    echo ""
    echo "--- Running $check ---"
    if bash "$ROOT/scripts/$check"; then
        echo "--- $check: PASSED ---"
    else
        echo "--- $check: FAILED ---"
        FAIL=1
    fi
done

# Additional checks

echo ""
echo "--- Additional Checks ---"

# Check Cargo.toml workspace members
MEMBERS=$(grep -c 'agentic-forge-' "$ROOT/Cargo.toml" || echo 0)
if [ "$MEMBERS" -ge 4 ]; then
    echo "PASS: Workspace has $MEMBERS forge crate references"
else
    echo "FAIL: Expected at least 4 workspace members, found $MEMBERS"
    FAIL=1
fi

# Check license
LICENSE=$(grep 'license' "$ROOT/Cargo.toml" | head -1)
if echo "$LICENSE" | grep -q 'MIT'; then
    echo "PASS: MIT license declared"
else
    echo "FAIL: MIT license not found in Cargo.toml"
    FAIL=1
fi

# Check SDK dependency
if grep -q 'agentic-sdk' "$ROOT/Cargo.toml"; then
    echo "PASS: agentic-sdk dependency present"
else
    echo "FAIL: agentic-sdk dependency missing"
    FAIL=1
fi

echo ""
if [ $FAIL -ne 0 ]; then
    echo "=== FULL VALIDATION FAILED ==="
    exit 1
fi
echo "=== FULL VALIDATION PASSED ==="

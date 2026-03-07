#!/usr/bin/env bash
set -euo pipefail

FAIL=0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Command Surface Check ==="

# Expected MCP tool names (15 tools)
TOOLS=(
    forge_blueprint_create
    forge_blueprint_get
    forge_blueprint_update
    forge_blueprint_validate
    forge_blueprint_list
    forge_entity_add
    forge_entity_infer
    forge_dependency_resolve
    forge_dependency_add
    forge_structure_generate
    forge_skeleton_create
    forge_test_generate
    forge_import_graph
    forge_wiring_create
    forge_export
)

echo "Checking 15 MCP tools in registry..."
REGISTRY="$ROOT/crates/agentic-forge-mcp/src/tools/registry.rs"
for tool in "${TOOLS[@]}"; do
    if ! grep -q "\"$tool\"" "$REGISTRY" 2>/dev/null; then
        echo "FAIL: MCP tool '$tool' not found in registry"
        FAIL=1
    else
        echo "PASS: $tool"
    fi
done

# Check CLI subcommands exist in main.rs
CLI_MAIN="$ROOT/crates/agentic-forge-cli/src/main.rs"
CLI_COMMANDS=(Blueprint Entity Dependency Structure Skeleton Test Import Wiring Export Serve Info Version)
echo ""
echo "Checking CLI subcommands..."
for cmd in "${CLI_COMMANDS[@]}"; do
    if ! grep -q "$cmd" "$CLI_MAIN" 2>/dev/null; then
        echo "FAIL: CLI command '$cmd' not found"
        FAIL=1
    else
        echo "PASS: $cmd"
    fi
done

# Check tool count constant
TOOL_COUNT=$(grep -r 'MCP_TOOL_COUNT.*=.*[0-9]' "$ROOT/crates/agentic-forge-mcp/src/" | head -1 | sed 's/.*= *\([0-9]*\).*/\1/')
if [ "$TOOL_COUNT" != "15" ]; then
    echo "FAIL: MCP_TOOL_COUNT is $TOOL_COUNT, expected 15"
    FAIL=1
else
    echo "PASS: MCP_TOOL_COUNT = 15"
fi

if [ $FAIL -ne 0 ]; then
    echo "=== FAILED ==="
    exit 1
fi
echo "=== ALL PASSED ==="

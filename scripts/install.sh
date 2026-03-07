#!/usr/bin/env bash
set -euo pipefail

PROFILE="${1:-default}"
REPO_URL="https://github.com/agentralabs/agentic-forge"
VERSION="0.1.0"

echo "=== AgenticForge Installation (profile: $PROFILE) ==="
echo ""

# Check prerequisites
command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found. Install Rust: https://rustup.rs"; exit 1; }
command -v git >/dev/null 2>&1 || { echo "ERROR: git not found."; exit 1; }

RUST_VERSION=$(rustc --version | awk '{print $2}')
echo "Rust version: $RUST_VERSION"

# Try release artifact first, fall back to source build
INSTALLED=false

# Source build (always available)
if [ "$INSTALLED" = false ]; then
    echo "Building from source..."
    if [ -f "Cargo.toml" ] && grep -q "agentic-forge" Cargo.toml 2>/dev/null; then
        echo "Building in current directory..."
    else
        TMPDIR=$(mktemp -d)
        echo "Cloning $REPO_URL..."
        git clone --depth 1 "$REPO_URL" "$TMPDIR/agentic-forge" 2>/dev/null || {
            echo "Clone failed, building from local source..."
        }
        if [ -d "$TMPDIR/agentic-forge" ]; then
            cd "$TMPDIR/agentic-forge"
        fi
    fi
    cargo build --workspace --release
    cargo install --path crates/agentic-forge-cli --force
    INSTALLED=true
fi

# MCP config merge (merge-only, never destructive overwrite)
merge_mcp_config() {
    local CONFIG_DIR="$1"
    local CONFIG_FILE="$CONFIG_DIR/mcp.json"
    local MCP_KEY="agentic-forge-mcp"
    local MCP_CMD
    MCP_CMD=$(command -v agentic-forge-mcp 2>/dev/null || echo "$(dirname "$(command -v aforge 2>/dev/null || echo "$HOME/.cargo/bin/aforge")")/../bin/agentic-forge-mcp")

    if [ ! -d "$CONFIG_DIR" ]; then
        mkdir -p "$CONFIG_DIR"
    fi

    if [ ! -f "$CONFIG_FILE" ]; then
        echo "{\"mcpServers\":{}}" > "$CONFIG_FILE"
    fi

    # Check if key already exists (merge-only: never overwrite existing)
    if command -v python3 >/dev/null 2>&1; then
        python3 -c "
import json, sys
with open('$CONFIG_FILE') as f:
    cfg = json.load(f)
servers = cfg.setdefault('mcpServers', {})
if '$MCP_KEY' not in servers:
    servers['$MCP_KEY'] = {'command': '$MCP_CMD', 'args': []}
    with open('$CONFIG_FILE', 'w') as f:
        json.dump(cfg, f, indent=2)
    print('  Added $MCP_KEY to $CONFIG_FILE')
else:
    print('  $MCP_KEY already in $CONFIG_FILE (preserved)')
" 2>/dev/null || true
    fi
}

# Profile-specific MCP config
case "$PROFILE" in
    desktop|default)
        echo ""
        echo "Configuring MCP for desktop clients..."
        # Claude Desktop
        if [ -d "$HOME/Library/Application Support/Claude" ]; then
            merge_mcp_config "$HOME/Library/Application Support/Claude"
        elif [ -d "$HOME/.config/claude" ]; then
            merge_mcp_config "$HOME/.config/claude"
        fi
        ;;
    terminal)
        echo ""
        echo "Terminal profile: CLI-only installation."
        ;;
    server)
        echo ""
        echo "Server profile: set AGENTIC_TOKEN for auth gate."
        if [ -z "${AGENTIC_TOKEN:-}" ]; then
            echo "  WARNING: AGENTIC_TOKEN not set. Server will run without auth."
        fi
        ;;
esac

# Verify installation
echo ""
if command -v aforge >/dev/null 2>&1; then
    echo "aforge installed: $(aforge version 2>/dev/null || echo 'ok')"
else
    echo "WARNING: aforge not found in PATH. Add ~/.cargo/bin to your PATH."
fi

# Completion block
SERVER_KEY="agentic-forge"
SERVER_CMD="$(command -v agentic-forge-mcp 2>/dev/null || echo "$HOME/.cargo/bin/agentic-forge-mcp")"
SERVER_ARGS_TEXT='["serve","--mode","stdio"]'

echo ""
echo "=== Installation Complete ==="
echo ""
echo "MCP client summary:"
echo "  - Compatible with Claude Desktop, Codex, Cursor, Windsurf,"
echo "    VS Code, Cline, and any MCP client."
echo ""
echo "Universal MCP entry (works in any MCP client):"
echo "  command: ${SERVER_CMD}"
echo "  args: ${SERVER_ARGS_TEXT}"
echo ""
echo "Quick terminal check:"
echo "  aforge --help"
echo "  aforge info"
echo ""
echo "What happens after installation:"
echo "  1. ${SERVER_KEY} CLI is installed as: $(command -v aforge 2>/dev/null || echo "$HOME/.cargo/bin/aforge")"
echo "  2. Restart your MCP client so it reloads MCP configuration."
echo "  3. After restart, confirm '${SERVER_KEY}' appears in your MCP server list."
echo "  4. Optional feedback: open https://github.com/agentralabs/agentic-forge/issues"
echo ""
echo "Optional feedback:"
echo "  - https://github.com/agentralabs/agentic-forge/issues"
echo ""

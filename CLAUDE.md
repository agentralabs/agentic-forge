# AgenticForge — Claude Code Instructions

Sister #11 "The Forge" — Blueprint Engine for complete project architecture.

## Quick Reference
- **Binary**: `aforge`
- **File format**: `.forge`
- **MCP Tools**: 15
- **Inventions**: 32 (8 tiers x 4)
- **Version**: 0.1.0

## Workspace Layout
```
crates/
  agentic-forge-core/    # Core library (types, engine, inventions, bridges)
  agentic-forge-mcp/     # MCP server (15 tools, stdio transport)
  agentic-forge-cli/     # CLI binary (aforge, 40+ commands)
  agentic-forge-ffi/     # C FFI bindings
```

## Build & Test
```bash
cargo build --workspace
cargo test --workspace      # 300+ tests
```

## MCP Quality Standard
- Tool descriptions: verb-first imperative, no trailing periods
- Error handling: tool execution errors use isError: true; protocol errors use JSON-RPC error
- Unknown tool: error code -32803 (TOOL_NOT_FOUND)
- Zero .unwrap() in MCP code

## Commit Style
- Conventional prefixes: feat:, fix:, chore:, docs:
- Never add Co-Authored-By: Claude

## Guardrails
```bash
bash scripts/check-canonical-consistency.sh
bash scripts/check-command-surface.sh
bash scripts/check-mcp-consolidation.sh
```

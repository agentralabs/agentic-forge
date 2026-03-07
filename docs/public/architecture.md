# System Architecture

## Workspace Structure

AgenticForge uses a 4-crate Rust workspace:

```
crates/
  agentic-forge-core/   Core library (types, engine, inventions, bridges)
  agentic-forge-mcp/    MCP server (15 tools, stdio transport)
  agentic-forge-cli/    CLI binary (aforge, 40+ commands)
  agentic-forge-ffi/    C FFI bindings
```

## Dependency Flow

```
cli ---> core <--- mcp
          ^
          |
         ffi
```

Both CLI and MCP depend on core. FFI wraps core for C consumers.

## Invention Tiers (8 tiers, 32 inventions)

| Tier | Name           | Purpose                                    |
|------|----------------|--------------------------------------------|
| 1    | Decomposition  | Break domains into layers and concerns      |
| 2    | Entity         | Infer domain objects, fields, relationships |
| 3    | Operation      | Design operations, signatures, error flows  |
| 4    | Structure      | Generate file layouts and import graphs     |
| 5    | Dependency     | Resolve external dependencies               |
| 6    | Blueprint      | Create code skeletons and contracts         |
| 7    | Integration    | Wire components, data flows, init sequences |
| 8    | Test           | Generate test cases, fixtures, mocks        |

## Engine Architecture

- **ForgeEngine** -- orchestrates all operations
- **BlueprintStore** -- in-memory storage with `.forge` file persistence
- **WriteEngine** -- mutation operations (add/remove entities, deps, files)
- **QueryEngine** -- read-only queries and summaries
- **BlueprintValidator** -- structural validation before export

## MCP Server

- JSON-RPC 2.0 over stdio transport
- SessionManager holds engine state per connection
- 15 tools with verb-first imperative descriptions
- Error code -32803 for unknown tools

## Security Model

- No `.unwrap()` in MCP code paths
- All FFI functions are null-safe
- Blueprint IDs use UUID v4

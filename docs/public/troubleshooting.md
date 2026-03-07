# Troubleshooting

## Build Issues

### Compilation fails with missing agentic-sdk

Ensure `agentic-sdk v0.2.0` is available. If developing locally, add a path
override in `.cargo/config.toml`:

```toml
[patch.crates-io]
agentic-sdk = { path = "../agentic-sdk" }
```

### Linker errors with FFI crate

The FFI crate produces both static and dynamic libraries. Ensure you link
against the correct one for your platform.

## CLI Issues

### "Blueprint not found" error

Blueprint IDs are UUIDs. Verify the ID with `aforge blueprint list`.
Blueprints are stored in memory and lost when the process exits unless
exported with `aforge export forge <id> <path>`.

### Entity inference returns no results

The EntityInferrer expects descriptive natural language. Provide context about
the domain objects, their relationships, and actions.

## MCP Issues

### Server does not respond

Ensure `aforge serve --mode stdio` is running and your client connects to
stdin/stdout. Check stderr for error messages.

### Unknown tool error (-32803)

This means the tool name is not recognized. Verify against the 15 supported
tools listed in [MCP Tools](mcp-tools.md).

### Invalid parameters error

Check the required fields for each tool. All blueprint operations require
a `blueprint_id` parameter (string UUID).

## Performance Issues

### Slow blueprint operations

Run in release mode: `cargo build --release`. Debug builds are significantly
slower due to lack of optimizations.

## Getting Help

- File issues at https://github.com/agentralabs/agentic-forge
- Check `aforge health` to verify the installation
- Use `aforge --verbose` for debug-level logging

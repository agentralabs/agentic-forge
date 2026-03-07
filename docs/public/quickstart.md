# Quickstart

Get started with AgenticForge in under five minutes.

## Prerequisites

- Rust 1.75+ with Cargo
- Git

## Install

```bash
cargo install --path crates/agentic-forge-cli
```

## Create Your First Blueprint

```bash
# Initialize a blueprint
aforge blueprint create my-api --domain api --description "REST API for task management"

# Infer entities from a description
aforge entity infer <blueprint_id> "Users create tasks with deadlines and assign them to teams"

# Resolve dependencies automatically
aforge dependency resolve <blueprint_id>

# Generate file structure
aforge structure generate <blueprint_id>

# Generate code skeletons
aforge skeleton create <blueprint_id>

# Generate test architecture
aforge test generate <blueprint_id>

# Validate the blueprint
aforge blueprint validate <blueprint_id>

# Export to JSON
aforge export json <blueprint_id>
```

## Use via MCP

```bash
aforge serve --mode stdio
```

Connect your MCP client to the stdio transport. The server exposes 15 tools
prefixed with `forge_` for blueprint management, entity inference, dependency
resolution, structure generation, and more.

## Next Steps

- Read [Concepts](concepts.md) for core terminology
- See [CLI Reference](cli-reference.md) for all commands
- See [MCP Tools](mcp-tools.md) for the full tool list

# Changelog

All notable changes to AgenticForge are documented here.

## [0.1.0] - 2026-03-06

### Added
- Core engine with BlueprintStore, WriteEngine, and QueryEngine
- 32 inventions across 8 tiers (Decomposition, Entity, Operation, Structure, Dependency, Blueprint, Integration, Test)
- 15 MCP tools with stdio JSON-RPC transport
- CLI binary `aforge` with 40+ commands
- C FFI bindings (version, create_blueprint, free_string, invention_count, tool_count)
- Blueprint lifecycle: draft -> in_progress -> complete -> validated -> exported
- Entity inference from natural language descriptions
- Automatic dependency resolution by domain
- File structure generation
- Code skeleton generation
- Test case generation
- Import graph generation
- Component wiring diagrams
- Blueprint validation
- 10 sister bridges with NoOp defaults
- `.forge` file format for persistence
- 313+ tests across all crates

# Rust API Reference

## Core Crate (`agentic-forge-core`)

### ForgeEngine

The central engine for blueprint management.

```rust
use agentic_forge_core::engine::ForgeEngine;

let mut engine = ForgeEngine::new();
let id = engine.create_blueprint("my-app", "A web app", Domain::Web)?;
```

**Key methods:**
- `create_blueprint(name, description, domain) -> Result<BlueprintId>`
- `reader() -> QueryEngine` -- read-only access
- `writer() -> WriteEngine` -- mutable operations
- `store` -- direct access to the BlueprintStore

### WriteEngine

- `add_entity(bp_id, entity) -> Result<EntityId>`
- `remove_entity(bp_id, entity_id) -> Result<()>`
- `add_dependency(bp_id, dep) -> Result<DependencyId>`
- `add_file(bp_id, file) -> Result<FileId>`
- `add_test_case(bp_id, tc) -> Result<TestCaseId>`
- `add_wiring(bp_id, wiring) -> Result<WiringId>`
- `rename_blueprint(bp_id, name) -> Result<()>`
- `set_description(bp_id, desc) -> Result<()>`
- `set_status(bp_id, status) -> Result<()>`
- `delete_blueprint(bp_id) -> Result<()>`

### QueryEngine

- `list_blueprints() -> Vec<&Blueprint>`
- `list_by_status(status) -> Vec<&Blueprint>`
- `blueprint_summary(bp_id) -> Result<Value>`

### Types

- `Blueprint`, `Entity`, `Dependency`, `Domain`, `BlueprintStatus`
- `BlueprintId`, `EntityId`, `DependencyId` (newtype wrappers over UUID)
- `IntentSpec`, `Constraint`

### Inventions (32)

Each invention is a struct with a primary static method (e.g., `::infer`,
`::generate`, `::analyze`, `::build`). See [Concepts](concepts.md) for the
full tier list.

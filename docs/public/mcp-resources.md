# MCP Resources

AgenticForge exposes blueprint data through the MCP resources protocol.

## URI Scheme

| URI Pattern | Description |
|-------------|-------------|
| `forge://blueprints` | List all blueprint summaries |
| `forge://blueprint/{id}` | Full blueprint data by ID |
| `forge://blueprint/{id}/entities` | Entity list for a blueprint |
| `forge://blueprint/{id}/files` | File structure for a blueprint |
| `forge://blueprint/{id}/dependencies` | Dependency list |
| `forge://blueprint/{id}/tests` | Test case list |
| `forge://blueprint/{id}/wiring` | Component wiring diagram |

## Formats

All resources return JSON. Blueprint resources include full serialized
Blueprint structs matching the Rust types in `agentic-forge-core`.

## Cross-Sister References

| Sister | Resource Integration |
|--------|---------------------|
| AgenticMemory | `memory://` URIs can store blueprint snapshots |
| AgenticReality | `reality://` URIs ground blueprints in deployment context |
| AgenticCognition | `cognition://` URIs link architecture reasoning |
| AgenticIdentity | `identity://` URIs bind blueprint ownership |

## Subscription

Resource subscription is not currently supported. Clients should
poll `forge://blueprints` for updates or use tool calls for mutations.

## Example

```json
{
  "method": "resources/read",
  "params": {
    "uri": "forge://blueprint/bp-550e8400-e29b-41d4-a716-446655440000"
  }
}
```

Response contains the full blueprint JSON with entities, files,
dependencies, tests, wiring, and data flows.

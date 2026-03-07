# MCP Tools Reference

AgenticForge exposes 15 MCP tools via JSON-RPC 2.0 over stdio transport.

## Starting the Server

```bash
aforge serve --mode stdio
```

## Tool List

### Blueprint Tools

| Tool                       | Description                                          |
|----------------------------|------------------------------------------------------|
| `forge_blueprint_create`   | Create a new project blueprint from intent description |
| `forge_blueprint_get`      | Get blueprint by ID                                  |
| `forge_blueprint_update`   | Update an existing blueprint                         |
| `forge_blueprint_validate` | Validate that a blueprint is buildable               |
| `forge_blueprint_list`     | List all blueprints                                  |

### Entity Tools

| Tool                       | Description                                          |
|----------------------------|------------------------------------------------------|
| `forge_entity_add`         | Add an entity to a blueprint                         |
| `forge_entity_infer`       | Infer entities from a natural language description   |

### Dependency Tools

| Tool                       | Description                                          |
|----------------------------|------------------------------------------------------|
| `forge_dependency_resolve` | Resolve all dependencies for a blueprint             |
| `forge_dependency_add`     | Add a dependency to a blueprint manually             |

### Generation Tools

| Tool                       | Description                                          |
|----------------------------|------------------------------------------------------|
| `forge_structure_generate` | Generate file structure for a blueprint              |
| `forge_skeleton_create`    | Create code skeletons for blueprint entities         |
| `forge_test_generate`      | Generate test architecture for a blueprint           |
| `forge_import_graph`       | Generate import graph for blueprint files            |
| `forge_wiring_create`      | Create component wiring diagram for a blueprint      |

### Export Tools

| Tool                       | Description                                          |
|----------------------------|------------------------------------------------------|
| `forge_export`             | Export blueprint to files on disk                    |

## Error Handling

- Tool execution errors: returned with `isError: true` in the result
- Unknown tool: JSON-RPC error code `-32803` (TOOL_NOT_FOUND)
- Invalid parameters: JSON-RPC error with descriptive message
- All descriptions are verb-first imperative with no trailing periods

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "forge_blueprint_create",
    "arguments": {
      "name": "my-api",
      "description": "REST API for task management",
      "domain": "api"
    }
  }
}
```

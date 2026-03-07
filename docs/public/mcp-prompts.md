# MCP Prompts

AgenticForge provides structured prompts to guide blueprint creation workflows.

## Available Prompts

### `forge_new_project`

Generate a complete blueprint from a natural language project description.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `description` | string | yes | Natural language project description |
| `domain` | string | no | Target domain (web, api, cli, library, service) |
| `language` | string | no | Target language (rust, python, typescript) |

**Behavior**: Runs the full invention pipeline (entity inference, field
derivation, operation inference, dependency resolution, file structure
generation, skeleton creation, test generation, wiring) and returns a
summary of the created blueprint.

### `forge_review_blueprint`

Review an existing blueprint for completeness and quality.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `blueprint_id` | string | yes | Blueprint ID to review |

**Behavior**: Runs validation, checks for missing entities, orphaned files,
unresolved dependencies, and missing test coverage. Returns a structured
review with actionable recommendations.

### `forge_evolve_blueprint`

Suggest architectural improvements to an existing blueprint.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `blueprint_id` | string | yes | Blueprint ID to evolve |
| `focus` | string | no | Focus area (performance, security, scalability) |

**Behavior**: Analyzes the blueprint through decomposition, concern analysis,
and wiring review. Returns suggested improvements with rationale.

## Example

```json
{
  "method": "prompts/get",
  "params": {
    "name": "forge_new_project",
    "arguments": {
      "description": "A REST API for managing tasks with user auth",
      "domain": "api"
    }
  }
}
```

Returns a structured message sequence that guides the LLM through
blueprint creation steps.

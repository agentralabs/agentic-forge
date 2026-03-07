# CLI Reference

Binary: `aforge`

## Global Flags

- `--format <text|json>` -- Output format (default: text)
- `--verbose` -- Enable debug logging

## Commands

### Blueprint Management

| Command                        | Description                     |
|--------------------------------|---------------------------------|
| `blueprint create <name> --domain <d>` | Create a new blueprint  |
| `blueprint get <id>`          | Get blueprint details            |
| `blueprint list [--status <s>]` | List blueprints                |
| `blueprint validate <id>`     | Validate a blueprint             |
| `blueprint export <id> [--format <f>]` | Export blueprint        |
| `blueprint delete <id>`       | Delete a blueprint               |
| `blueprint update <id> [--name] [--description]` | Update fields |

### Entity Management

| Command                        | Description                     |
|--------------------------------|---------------------------------|
| `entity add <bp_id> <name>`   | Add an entity                    |
| `entity infer <bp_id> <desc>` | Infer entities from description  |
| `entity list <bp_id>`         | List entities                    |
| `entity remove <bp_id> <eid>` | Remove an entity                 |
| `entity fields <bp_id> <name>` | Show entity fields              |

### Dependency Management

| Command                        | Description                     |
|--------------------------------|---------------------------------|
| `dependency resolve <bp_id>`  | Auto-resolve dependencies        |
| `dependency add <bp_id> <name> <ver>` | Add manually             |
| `dependency list <bp_id>`     | List dependencies                |
| `dependency remove <bp_id> <did>` | Remove a dependency          |

### Generation

| Command                        | Description                     |
|--------------------------------|---------------------------------|
| `structure generate <bp_id>`  | Generate file structure          |
| `skeleton create <bp_id>`     | Generate code skeletons          |
| `test generate <bp_id>`       | Generate test cases              |
| `test list <bp_id>`           | List test cases                  |
| `import graph <bp_id>`        | Generate import graph            |
| `wiring create <bp_id>`       | Create wiring diagram            |
| `wiring list <bp_id>`         | List wirings                     |
| `export json <bp_id>`         | Export as JSON                   |
| `export forge <bp_id> <path>` | Export as .forge file            |

### Utility

| Command                        | Description                     |
|--------------------------------|---------------------------------|
| `serve --mode stdio`          | Start MCP server                 |
| `info`                        | Show system info                 |
| `version`                     | Show version                     |
| `health`                      | Health check                     |
| `clean`                       | Clean state                      |
| `init --name <n> --domain <d>` | Quick-create a blueprint        |
| `validate <bp_id>`            | Validate (shorthand)             |
| `decompose --domain <d>`      | Show domain layers               |
| `infer --description <d>`     | Infer entities standalone        |
| `resolve <bp_id>`             | Resolve deps (shorthand)         |
| `status <bp_id>`              | Show blueprint status            |
| `summary <bp_id>`             | Show blueprint summary           |
| `concerns --domain <d>`       | Analyze cross-cutting concerns   |
| `layers --domain <d>`         | Show domain layers               |
| `config show`                 | Show configuration               |
| `config set <key> <value>`    | Set configuration value          |

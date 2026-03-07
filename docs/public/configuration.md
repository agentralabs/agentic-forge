# Configuration

## Environment Variables

| Variable              | Default | Description                  |
|-----------------------|---------|------------------------------|
| `AFORGE_LOG`          | `warn`  | Log level (trace/debug/info/warn/error) |
| `AFORGE_DATA_DIR`     | `~/.agentic-forge` | Data directory      |
| `AFORGE_FORMAT`       | `text`  | Default output format        |

## CLI Configuration

View current configuration:

```bash
aforge config show
```

Set a configuration value:

```bash
aforge config set key value
```

## MCP Server Configuration

The MCP server reads configuration from:

1. Environment variables (highest priority)
2. `~/.agentic-forge/config.json`
3. Built-in defaults

### Example config.json

```json
{
  "log_level": "info",
  "data_dir": "/path/to/data",
  "max_blueprints": 1000,
  "auto_validate": true
}
```

## Blueprint Defaults

When creating blueprints, these defaults apply:

- Status: `draft`
- Format: in-memory (use `export` to persist)
- Validation: manual (run `blueprint validate` or `forge_blueprint_validate`)

## Verbose Mode

Enable verbose logging for any command:

```bash
aforge --verbose blueprint create my-app --domain api
```

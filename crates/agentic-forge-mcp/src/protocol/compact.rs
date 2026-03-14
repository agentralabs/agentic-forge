//! Compact MCP tool facade — collapses 15 tools into 3 grouped facades

use serde_json::{json, Value};

/// Blueprint operations: create, get, update, validate, list
const BLUEPRINT_OPS: &[&str] = &[
    "blueprint_create",
    "blueprint_get",
    "blueprint_update",
    "blueprint_validate",
    "blueprint_list",
];

/// Entity operations: entity_add, entity_infer, dependency_resolve, dependency_add
const ENTITY_OPS: &[&str] = &[
    "entity_add",
    "entity_infer",
    "dependency_resolve",
    "dependency_add",
];

/// Generation operations: structure, skeleton, tests, import graph, wiring, export
const GENERATION_OPS: &[&str] = &[
    "structure_generate",
    "skeleton_create",
    "test_generate",
    "import_graph",
    "wiring_create",
    "export",
];

/// Check whether the compact tool surface is active
pub fn mcp_tool_surface_is_compact() -> bool {
    std::env::var("AFORGE_MCP_TOOL_SURFACE")
        .or_else(|_| std::env::var("MCP_TOOL_SURFACE"))
        .map(|v| v.eq_ignore_ascii_case("compact"))
        .unwrap_or(false)
}

/// Build an input schema for a compact facade tool with an `operation` enum
fn compact_op_schema(ops: &[&str], description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": {
            "operation": {
                "type": "string",
                "enum": ops,
                "description": "Operation to perform"
            },
            "args": {
                "type": "object",
                "description": "Arguments forwarded to the underlying tool",
                "additionalProperties": true
            }
        },
        "required": ["operation"]
    })
}

/// Return the three compact tool definitions
pub fn compact_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "forge_blueprint",
            "description": "Manage project blueprints",
            "inputSchema": compact_op_schema(
                BLUEPRINT_OPS,
                "Blueprint lifecycle: create, get, update, validate, list"
            )
        }),
        json!({
            "name": "forge_entity",
            "description": "Manage entities and dependencies",
            "inputSchema": compact_op_schema(
                ENTITY_OPS,
                "Entity management and dependency resolution"
            )
        }),
        json!({
            "name": "forge_generation",
            "description": "Generate code structure, skeletons, tests, and exports",
            "inputSchema": compact_op_schema(
                GENERATION_OPS,
                "Code generation, import graphs, wiring, and export"
            )
        }),
    ]
}

/// Decode a compact tool call into (operation, args)
pub fn decode_compact_operation(args: Value) -> Result<(String, Value), String> {
    let operation = args
        .get("operation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'operation' field".to_string())?
        .to_string();

    let inner_args = args
        .get("args")
        .cloned()
        .unwrap_or_else(|| json!({}));

    Ok((operation, inner_args))
}

/// Resolve a compact group + operation to the canonical tool name
pub fn resolve_compact_tool(group: &str, operation: &str) -> Option<String> {
    let ops: &[&str] = match group {
        "forge_blueprint" => BLUEPRINT_OPS,
        "forge_entity" => ENTITY_OPS,
        "forge_generation" => GENERATION_OPS,
        _ => return None,
    };

    if ops.contains(&operation) {
        Some(format!("forge_{}", operation))
    } else {
        None
    }
}

/// Normalize a tool call: if compact, resolve to canonical name + inner args.
/// If already a canonical tool name, pass through unchanged.
pub fn normalize_compact_tool_call(
    tool_name: &str,
    args: Value,
) -> Result<(String, Value), String> {
    match tool_name {
        "forge_blueprint" | "forge_entity" | "forge_generation" => {
            let (operation, inner_args) = decode_compact_operation(args)?;
            let canonical = resolve_compact_tool(tool_name, &operation)
                .ok_or_else(|| {
                    format!(
                        "Unknown operation '{}' for group '{}'",
                        operation, tool_name
                    )
                })?;
            Ok((canonical, inner_args))
        }
        // Not a compact name — pass through as-is
        _ => Ok((tool_name.to_string(), args)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_definitions_count() {
        assert_eq!(compact_tool_definitions().len(), 3);
    }

    #[test]
    fn resolve_blueprint_ops() {
        assert_eq!(
            resolve_compact_tool("forge_blueprint", "blueprint_create"),
            Some("forge_blueprint_create".into())
        );
        assert_eq!(
            resolve_compact_tool("forge_blueprint", "blueprint_list"),
            Some("forge_blueprint_list".into())
        );
        assert_eq!(
            resolve_compact_tool("forge_blueprint", "bogus"),
            None
        );
    }

    #[test]
    fn resolve_entity_ops() {
        assert_eq!(
            resolve_compact_tool("forge_entity", "entity_add"),
            Some("forge_entity_add".into())
        );
        assert_eq!(
            resolve_compact_tool("forge_entity", "dependency_resolve"),
            Some("forge_dependency_resolve".into())
        );
    }

    #[test]
    fn resolve_generation_ops() {
        assert_eq!(
            resolve_compact_tool("forge_generation", "structure_generate"),
            Some("forge_structure_generate".into())
        );
        assert_eq!(
            resolve_compact_tool("forge_generation", "export"),
            Some("forge_export".into())
        );
    }

    #[test]
    fn normalize_compact_call() {
        let args = json!({
            "operation": "blueprint_create",
            "args": { "name": "test", "description": "d", "domain": "web" }
        });
        let (name, inner) =
            normalize_compact_tool_call("forge_blueprint", args).unwrap();
        assert_eq!(name, "forge_blueprint_create");
        assert_eq!(inner["name"], "test");
    }

    #[test]
    fn normalize_passthrough() {
        let args = json!({ "blueprint_id": "abc" });
        let (name, inner) =
            normalize_compact_tool_call("forge_blueprint_get", args.clone())
                .unwrap();
        assert_eq!(name, "forge_blueprint_get");
        assert_eq!(inner, args);
    }

    #[test]
    fn decode_missing_operation() {
        let args = json!({ "args": {} });
        assert!(decode_compact_operation(args).is_err());
    }

    #[test]
    fn decode_missing_args_defaults_empty() {
        let args = json!({ "operation": "export" });
        let (op, inner) = decode_compact_operation(args).unwrap();
        assert_eq!(op, "export");
        assert_eq!(inner, json!({}));
    }

    #[test]
    fn compact_mode_off_by_default() {
        assert!(!mcp_tool_surface_is_compact());
    }
}

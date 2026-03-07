//! Tool registry — 15 MCP tools for AgenticForge.

use crate::session::SessionManager;
use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};
use agentic_forge_core::types::blueprint::*;
use agentic_forge_core::types::ids::*;
use agentic_forge_core::types::intent::*;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

pub const MCP_TOOL_COUNT: usize = 15;

pub struct ToolRegistry;

/// Token conservation params added to all query/get/list tools.
fn conservation_properties() -> Vec<(&'static str, Value)> {
    vec![
        (
            "include_content",
            json!({ "type": "boolean", "default": false, "description": "Include full content in response" }),
        ),
        (
            "intent",
            json!({ "type": "string", "enum": ["exists", "ids", "summary", "full"], "description": "Extraction scope" }),
        ),
        (
            "since",
            json!({ "type": "integer", "description": "Delta: only changes after this timestamp" }),
        ),
        (
            "token_budget",
            json!({ "type": "integer", "description": "Maximum token budget for response" }),
        ),
        (
            "max_results",
            json!({ "type": "integer", "default": 10, "description": "Maximum number of results" }),
        ),
        (
            "cursor",
            json!({ "type": "string", "description": "Pagination cursor" }),
        ),
    ]
}

/// Merge conservation properties into a schema's properties object.
fn with_conservation(mut schema: Value) -> Value {
    if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
        for (key, val) in conservation_properties() {
            props.insert(key.to_string(), val);
        }
    }
    schema
}

impl ToolRegistry {
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "forge_blueprint_create".into(),
                description: Some("Create a new project blueprint from intent description".into()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Blueprint name" },
                        "description": { "type": "string", "description": "Project description" },
                        "domain": { "type": "string", "enum": ["web", "api", "cli", "library", "service", "database", "embedded", "mobile", "desktop", "plugin"], "description": "Project domain" }
                    },
                    "required": ["name", "description", "domain"]
                }),
            },
            ToolDefinition {
                name: "forge_blueprint_get".into(),
                description: Some("Get blueprint by ID".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string", "description": "Blueprint ID" }
                    },
                    "required": ["blueprint_id"]
                })),
            },
            ToolDefinition {
                name: "forge_blueprint_update".into(),
                description: Some("Update an existing blueprint".into()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string", "description": "Blueprint ID" },
                        "name": { "type": "string" },
                        "description": { "type": "string" },
                        "status": { "type": "string", "enum": ["draft", "in_progress", "complete", "validated"] }
                    },
                    "required": ["blueprint_id"]
                }),
            },
            ToolDefinition {
                name: "forge_blueprint_validate".into(),
                description: Some("Validate that a blueprint is buildable".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string", "description": "Blueprint ID" }
                    },
                    "required": ["blueprint_id"]
                })),
            },
            ToolDefinition {
                name: "forge_blueprint_list".into(),
                description: Some("List all blueprints".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "status": { "type": "string", "description": "Filter by status" }
                    }
                })),
            },
            ToolDefinition {
                name: "forge_entity_add".into(),
                description: Some("Add an entity to a blueprint".into()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" },
                        "name": { "type": "string", "description": "Entity name" },
                        "description": { "type": "string", "description": "Entity description" },
                        "is_aggregate_root": { "type": "boolean", "default": false }
                    },
                    "required": ["blueprint_id", "name", "description"]
                }),
            },
            ToolDefinition {
                name: "forge_entity_infer".into(),
                description: Some("Infer entities from a natural language description".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" },
                        "description": { "type": "string", "description": "Description to infer entities from" }
                    },
                    "required": ["blueprint_id", "description"]
                })),
            },
            ToolDefinition {
                name: "forge_dependency_resolve".into(),
                description: Some("Resolve all dependencies for a blueprint".into()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" }
                    },
                    "required": ["blueprint_id"]
                }),
            },
            ToolDefinition {
                name: "forge_dependency_add".into(),
                description: Some("Add a dependency to a blueprint manually".into()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" },
                        "name": { "type": "string", "description": "Dependency name" },
                        "version": { "type": "string", "description": "Version string" }
                    },
                    "required": ["blueprint_id", "name", "version"]
                }),
            },
            ToolDefinition {
                name: "forge_structure_generate".into(),
                description: Some("Generate file structure for a blueprint".into()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" }
                    },
                    "required": ["blueprint_id"]
                }),
            },
            ToolDefinition {
                name: "forge_skeleton_create".into(),
                description: Some("Create code skeletons for blueprint entities".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" }
                    },
                    "required": ["blueprint_id"]
                })),
            },
            ToolDefinition {
                name: "forge_test_generate".into(),
                description: Some("Generate test architecture for a blueprint".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" }
                    },
                    "required": ["blueprint_id"]
                })),
            },
            ToolDefinition {
                name: "forge_import_graph".into(),
                description: Some("Generate import graph for blueprint files".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" }
                    },
                    "required": ["blueprint_id"]
                })),
            },
            ToolDefinition {
                name: "forge_wiring_create".into(),
                description: Some("Create component wiring diagram for a blueprint".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" }
                    },
                    "required": ["blueprint_id"]
                })),
            },
            ToolDefinition {
                name: "forge_export".into(),
                description: Some("Export blueprint to files on disk".into()),
                input_schema: with_conservation(json!({
                    "type": "object",
                    "properties": {
                        "blueprint_id": { "type": "string" },
                        "output_path": { "type": "string", "description": "Output directory path" },
                        "format": { "type": "string", "enum": ["json", "forge", "files"], "default": "json" }
                    },
                    "required": ["blueprint_id"]
                })),
            },
        ]
    }

    pub async fn call(
        name: &str,
        arguments: Option<Value>,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let args = arguments.unwrap_or(Value::Object(serde_json::Map::new()));

        match name {
            "forge_blueprint_create" => Self::blueprint_create(args, session).await,
            "forge_blueprint_get" => Self::blueprint_get(args, session).await,
            "forge_blueprint_update" => Self::blueprint_update(args, session).await,
            "forge_blueprint_validate" => Self::blueprint_validate(args, session).await,
            "forge_blueprint_list" => Self::blueprint_list(args, session).await,
            "forge_entity_add" => Self::entity_add(args, session).await,
            "forge_entity_infer" => Self::entity_infer(args, session).await,
            "forge_dependency_resolve" => Self::dependency_resolve(args, session).await,
            "forge_dependency_add" => Self::dependency_add(args, session).await,
            "forge_structure_generate" => Self::structure_generate(args, session).await,
            "forge_skeleton_create" => Self::skeleton_create(args, session).await,
            "forge_test_generate" => Self::test_generate(args, session).await,
            "forge_import_graph" => Self::import_graph(args, session).await,
            "forge_wiring_create" => Self::wiring_create(args, session).await,
            "forge_export" => Self::export(args, session).await,
            _ => Err(McpError::ToolNotFound(name.to_string())),
        }
    }

    async fn blueprint_create(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("name is required".into()))?
            .to_string();
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("description is required".into()))?
            .to_string();
        let domain_str = args
            .get("domain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("domain is required".into()))?;
        let domain = Domain::from_name(domain_str)
            .ok_or_else(|| McpError::InvalidParams(format!("Unknown domain: {}", domain_str)))?;

        let mut session = session.lock().await;
        let id = session
            .engine
            .create_blueprint(&name, &description, domain)
            .map_err(|e| McpError::Forge(e.to_string()))?;

        Ok(ToolCallResult::json(&json!({
            "blueprint_id": id.to_string(),
            "name": name,
            "domain": domain_str,
            "status": "draft"
        })))
    }

    async fn blueprint_get(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let session = session.lock().await;
        let bp = session
            .engine
            .store
            .load(&bp_id)
            .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;

        Ok(ToolCallResult::json(&json!({
            "blueprint_id": bp.id.to_string(),
            "name": bp.name,
            "description": bp.description,
            "domain": format!("{:?}", bp.domain),
            "status": bp.status.name(),
            "entities": bp.entity_count(),
            "files": bp.file_count(),
            "dependencies": bp.dependency_count(),
            "tests": bp.test_count()
        })))
    }

    async fn blueprint_update(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let mut session = session.lock().await;

        if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
            session
                .engine
                .writer()
                .rename_blueprint(&bp_id, name)
                .map_err(|e| McpError::Forge(e.to_string()))?;
        }
        if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
            session
                .engine
                .writer()
                .set_description(&bp_id, desc)
                .map_err(|e| McpError::Forge(e.to_string()))?;
        }
        if let Some(status_str) = args.get("status").and_then(|v| v.as_str()) {
            let status = match status_str {
                "draft" => BlueprintStatus::Draft,
                "in_progress" => BlueprintStatus::InProgress,
                "complete" => BlueprintStatus::Complete,
                "validated" => BlueprintStatus::Validated,
                _ => {
                    return Err(McpError::InvalidParams(format!(
                        "Unknown status: {}",
                        status_str
                    )))
                }
            };
            session
                .engine
                .writer()
                .set_status(&bp_id, status)
                .map_err(|e| McpError::Forge(e.to_string()))?;
        }

        Ok(ToolCallResult::text(format!("Blueprint {} updated", bp_id)))
    }

    async fn blueprint_validate(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let session = session.lock().await;
        let bp = session
            .engine
            .store
            .load(&bp_id)
            .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;

        let report = agentic_forge_core::engine::validator::BlueprintValidator::validate(bp)
            .map_err(|e| McpError::Forge(e.to_string()))?;

        Ok(ToolCallResult::json(&json!({
            "is_valid": report.is_valid,
            "errors": report.errors,
            "warnings": report.warnings,
            "error_count": report.error_count(),
            "warning_count": report.warning_count()
        })))
    }

    async fn blueprint_list(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let session = session.lock().await;
        let reader = session.engine.reader();

        let blueprints = if let Some(status_str) = args.get("status").and_then(|v| v.as_str()) {
            let status = match status_str {
                "draft" => BlueprintStatus::Draft,
                "in_progress" => BlueprintStatus::InProgress,
                "complete" => BlueprintStatus::Complete,
                "validated" => BlueprintStatus::Validated,
                "exported" => BlueprintStatus::Exported,
                _ => {
                    return Err(McpError::InvalidParams(format!(
                        "Unknown status: {}",
                        status_str
                    )))
                }
            };
            reader.list_by_status(status)
        } else {
            reader.list_blueprints()
        };

        let list: Vec<Value> = blueprints
            .iter()
            .map(|bp| {
                json!({
                    "id": bp.id.to_string(),
                    "name": bp.name,
                    "status": bp.status.name(),
                    "entities": bp.entity_count(),
                    "files": bp.file_count()
                })
            })
            .collect();

        Ok(ToolCallResult::json(
            &json!({ "blueprints": list, "count": list.len() }),
        ))
    }

    async fn entity_add(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("name is required".into()))?
            .to_string();
        let desc = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let is_root = args
            .get("is_aggregate_root")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut entity = Entity::new(&name, &desc);
        entity.is_aggregate_root = is_root;

        let mut session = session.lock().await;
        let eid = session
            .engine
            .writer()
            .add_entity(&bp_id, entity)
            .map_err(|e| McpError::Forge(e.to_string()))?;

        Ok(ToolCallResult::json(
            &json!({ "entity_id": eid.to_string(), "name": name }),
        ))
    }

    async fn entity_infer(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("description is required".into()))?;

        let inferred = agentic_forge_core::inventions::EntityInferrer::infer(description);
        let mut added = Vec::new();

        let mut session = session.lock().await;
        for spec in &inferred {
            let entity = Entity::new(&spec.name, &spec.description);
            if let Ok(eid) = session.engine.writer().add_entity(&bp_id, entity) {
                added.push(json!({ "entity_id": eid.to_string(), "name": spec.name }));
            }
        }

        Ok(ToolCallResult::json(
            &json!({ "inferred": added, "count": added.len() }),
        ))
    }

    async fn dependency_resolve(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let mut session = session.lock().await;
        let bp = session
            .engine
            .store
            .load(&bp_id)
            .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;
        let domain = bp.domain;
        let entities: Vec<Entity> = bp.entities.clone();
        let constraints: Vec<agentic_forge_core::types::intent::Constraint> = Vec::new();

        let inferred = agentic_forge_core::inventions::DependencyInferrer::infer(
            domain,
            &entities,
            &constraints,
        );
        let mut added = 0usize;
        for dep in inferred {
            if session.engine.writer().add_dependency(&bp_id, dep).is_ok() {
                added += 1;
            }
        }

        Ok(ToolCallResult::json(
            &json!({ "dependencies_added": added }),
        ))
    }

    async fn dependency_add(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("name is required".into()))?
            .to_string();
        let version = args
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("version is required".into()))?
            .to_string();

        let dep = Dependency::new(&name, &version);
        let mut session = session.lock().await;
        let did = session
            .engine
            .writer()
            .add_dependency(&bp_id, dep)
            .map_err(|e| McpError::Forge(e.to_string()))?;

        Ok(ToolCallResult::json(
            &json!({ "dependency_id": did.to_string(), "name": name, "version": version }),
        ))
    }

    async fn structure_generate(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let mut session = session.lock().await;
        let bp = session
            .engine
            .store
            .load(&bp_id)
            .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?
            .clone();
        let files = agentic_forge_core::inventions::FileStructureGenerator::generate(&bp);
        let mut added = 0usize;
        for file in files {
            if session.engine.writer().add_file(&bp_id, file).is_ok() {
                added += 1;
            }
        }

        Ok(ToolCallResult::json(&json!({ "files_generated": added })))
    }

    async fn skeleton_create(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let session = session.lock().await;
        let bp = session
            .engine
            .store
            .load(&bp_id)
            .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;
        let mut skeletons = Vec::new();
        for entity in &bp.entities {
            skeletons.push(json!({
                "entity": entity.name,
                "skeleton": agentic_forge_core::inventions::SkeletonGenerator::generate(entity)
            }));
        }

        Ok(ToolCallResult::json(
            &json!({ "skeletons": skeletons, "count": skeletons.len() }),
        ))
    }

    async fn test_generate(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let mut session = session.lock().await;
        let entities: Vec<Entity> = {
            let bp = session
                .engine
                .store
                .load(&bp_id)
                .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;
            bp.entities.clone()
        };
        let mut added = 0usize;
        for entity in &entities {
            let tests = agentic_forge_core::inventions::TestCaseGenerator::generate(entity);
            for tc in tests {
                if session.engine.writer().add_test_case(&bp_id, tc).is_ok() {
                    added += 1;
                }
            }
        }

        Ok(ToolCallResult::json(&json!({ "tests_generated": added })))
    }

    async fn import_graph(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let session = session.lock().await;
        let bp = session
            .engine
            .store
            .load(&bp_id)
            .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;
        let edges = agentic_forge_core::inventions::ImportGraphGenerator::generate(&bp.files);

        let graph: Vec<Value> = edges
            .iter()
            .map(|e| {
                json!({
                    "from": e.from_file,
                    "to": e.to_file,
                    "symbols": e.imported_symbols
                })
            })
            .collect();

        Ok(ToolCallResult::json(
            &json!({ "import_graph": graph, "edges": graph.len() }),
        ))
    }

    async fn wiring_create(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let mut session = session.lock().await;
        let (entities, layers) = {
            let bp = session
                .engine
                .store
                .load(&bp_id)
                .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;
            (bp.entities.clone(), bp.layers.clone())
        };
        let wirings =
            agentic_forge_core::inventions::WiringDiagramBuilder::build(&entities, &layers);
        let mut added = 0usize;
        for w in wirings {
            if session.engine.writer().add_wiring(&bp_id, w).is_ok() {
                added += 1;
            }
        }

        Ok(ToolCallResult::json(&json!({ "wirings_created": added })))
    }

    async fn export(
        args: Value,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ToolCallResult> {
        let bp_id_str = args
            .get("blueprint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("blueprint_id is required".into()))?;
        let bp_id: BlueprintId = bp_id_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;
        let format = args
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let session = session.lock().await;
        let bp = session
            .engine
            .store
            .load(&bp_id)
            .map_err(|e| McpError::BlueprintNotFound(e.to_string()))?;

        match format {
            "json" => Ok(ToolCallResult::json(bp)),
            _ => Ok(ToolCallResult::json(bp)),
        }
    }
}

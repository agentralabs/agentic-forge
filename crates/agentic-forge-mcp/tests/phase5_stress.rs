//! Phase 5: MCP stress tests — rapid tool calls, large payloads, pipeline sequences.

use agentic_forge_mcp::session::SessionManager;
use agentic_forge_mcp::tools::registry::ToolRegistry;
use agentic_forge_mcp::types::*;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

fn session() -> Arc<Mutex<SessionManager>> {
    Arc::new(Mutex::new(SessionManager::new()))
}

fn extract_bp_id(result: &ToolCallResult) -> String {
    let text = match &result.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    data["blueprint_id"].as_str().unwrap().to_string()
}

// ── Rapid creation stress ────────────────────────────────────────────

#[tokio::test]
async fn test_stress_rapid_blueprint_creation() {
    let s = session();
    let start = Instant::now();

    for i in 0..100 {
        let result = ToolRegistry::call(
            "forge_blueprint_create",
            Some(json!({
                "name": format!("Rapid_{}", i),
                "description": format!("Rapid test {}", i),
                "domain": "api"
            })),
            &s,
        )
        .await
        .unwrap();
        assert!(result.is_error.is_none());
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 5000,
        "100 creates took {}ms",
        elapsed.as_millis()
    );

    // Verify count
    let list_result = ToolRegistry::call("forge_blueprint_list", Some(json!({})), &s)
        .await
        .unwrap();
    let text = match &list_result.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["count"].as_u64().unwrap(), 100);
}

// ── Full pipeline stress ─────────────────────────────────────────────

#[tokio::test]
async fn test_stress_full_pipeline() {
    let s = session();
    let start = Instant::now();

    // Create blueprint
    let create = ToolRegistry::call(
        "forge_blueprint_create",
        Some(json!({
            "name": "PipelineStress",
            "description": "Full pipeline stress test",
            "domain": "api"
        })),
        &s,
    )
    .await
    .unwrap();
    let bp_id = extract_bp_id(&create);

    // Entity inference
    let infer = ToolRegistry::call("forge_entity_infer", Some(json!({
        "blueprint_id": bp_id,
        "description": "A system with users, posts, comments, tags, categories, products, orders, and payments"
    })), &s).await.unwrap();
    assert!(infer.is_error.is_none());

    // Dependency resolution
    let resolve = ToolRegistry::call(
        "forge_dependency_resolve",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(resolve.is_error.is_none());

    // Add manual dependency
    let add_dep = ToolRegistry::call(
        "forge_dependency_add",
        Some(json!({
            "blueprint_id": bp_id,
            "name": "custom-crate",
            "version": "0.5.0"
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(add_dep.is_error.is_none());

    // Structure generation
    let structure = ToolRegistry::call(
        "forge_structure_generate",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(structure.is_error.is_none());

    // Skeleton creation
    let skeleton = ToolRegistry::call(
        "forge_skeleton_create",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(skeleton.is_error.is_none());

    // Test generation
    let test_gen = ToolRegistry::call(
        "forge_test_generate",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(test_gen.is_error.is_none());

    // Import graph
    let graph = ToolRegistry::call(
        "forge_import_graph",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(graph.is_error.is_none());

    // Wiring
    let wiring = ToolRegistry::call(
        "forge_wiring_create",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(wiring.is_error.is_none());

    // Validate
    let validate = ToolRegistry::call(
        "forge_blueprint_validate",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(validate.is_error.is_none());

    // Export
    let export = ToolRegistry::call(
        "forge_export",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    assert!(export.is_error.is_none());

    // Get final state
    let get = ToolRegistry::call(
        "forge_blueprint_get",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    let text = match &get.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 5000,
        "Full pipeline took {}ms",
        elapsed.as_millis()
    );

    // Verify populated
    assert!(
        data["entities"].as_u64().unwrap() >= 4,
        "Should have inferred entities"
    );
    assert!(
        data["dependencies"].as_u64().unwrap() >= 3,
        "Should have dependencies"
    );
    assert!(
        data["files"].as_u64().unwrap() >= 3,
        "Should have generated files"
    );
    assert!(
        data["tests"].as_u64().unwrap() >= 5,
        "Should have generated tests"
    );
}

// ── Many entities in one blueprint ───────────────────────────────────

#[tokio::test]
async fn test_stress_many_entities() {
    let s = session();
    let create = ToolRegistry::call(
        "forge_blueprint_create",
        Some(json!({
            "name": "ManyEntities", "description": "stress", "domain": "api"
        })),
        &s,
    )
    .await
    .unwrap();
    let bp_id = extract_bp_id(&create);

    for i in 0..200 {
        let result = ToolRegistry::call(
            "forge_entity_add",
            Some(json!({
                "blueprint_id": bp_id,
                "name": format!("Entity_{}", i),
                "description": format!("Entity number {}", i)
            })),
            &s,
        )
        .await
        .unwrap();
        assert!(result.is_error.is_none());
    }

    let get = ToolRegistry::call(
        "forge_blueprint_get",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    let text = match &get.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["entities"].as_u64().unwrap(), 200);
}

// ── Rapid list calls ─────────────────────────────────────────────────

#[tokio::test]
async fn test_stress_rapid_list_calls() {
    let s = session();

    // Create some blueprints
    for i in 0..20 {
        ToolRegistry::call(
            "forge_blueprint_create",
            Some(json!({
                "name": format!("List_{}", i), "description": "list test", "domain": "cli"
            })),
            &s,
        )
        .await
        .unwrap();
    }

    let start = Instant::now();
    for _ in 0..500 {
        let result = ToolRegistry::call("forge_blueprint_list", Some(json!({})), &s)
            .await
            .unwrap();
        assert!(result.is_error.is_none());
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 5000,
        "500 list calls took {}ms",
        elapsed.as_millis()
    );
}

// ── Sequential pipeline repetition ───────────────────────────────────

#[tokio::test]
async fn test_stress_repeated_pipelines() {
    let s = session();

    for i in 0..20 {
        let create = ToolRegistry::call(
            "forge_blueprint_create",
            Some(json!({
                "name": format!("Pipeline_{}", i), "description": "repeat", "domain": "api"
            })),
            &s,
        )
        .await
        .unwrap();
        let bp_id = extract_bp_id(&create);

        ToolRegistry::call(
            "forge_entity_infer",
            Some(json!({
                "blueprint_id": bp_id, "description": "users and posts"
            })),
            &s,
        )
        .await
        .unwrap();

        ToolRegistry::call(
            "forge_dependency_resolve",
            Some(json!({
                "blueprint_id": bp_id
            })),
            &s,
        )
        .await
        .unwrap();

        ToolRegistry::call(
            "forge_structure_generate",
            Some(json!({
                "blueprint_id": bp_id
            })),
            &s,
        )
        .await
        .unwrap();

        ToolRegistry::call(
            "forge_test_generate",
            Some(json!({
                "blueprint_id": bp_id
            })),
            &s,
        )
        .await
        .unwrap();
    }

    let list = ToolRegistry::call("forge_blueprint_list", Some(json!({})), &s)
        .await
        .unwrap();
    let text = match &list.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["count"].as_u64().unwrap(), 20);
}

// ── Update stress ────────────────────────────────────────────────────

#[tokio::test]
async fn test_stress_rapid_updates() {
    let s = session();
    let create = ToolRegistry::call(
        "forge_blueprint_create",
        Some(json!({
            "name": "Updatable", "description": "update stress", "domain": "api"
        })),
        &s,
    )
    .await
    .unwrap();
    let bp_id = extract_bp_id(&create);

    let start = Instant::now();
    for i in 0..100 {
        ToolRegistry::call(
            "forge_blueprint_update",
            Some(json!({
                "blueprint_id": bp_id,
                "name": format!("Updated_{}", i),
                "description": format!("Round {}", i)
            })),
            &s,
        )
        .await
        .unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 2000,
        "100 updates took {}ms",
        elapsed.as_millis()
    );

    let get = ToolRegistry::call(
        "forge_blueprint_get",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    let text = match &get.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["name"], "Updated_99");
}

// ── Dependency stress ────────────────────────────────────────────────

#[tokio::test]
async fn test_stress_many_dependencies() {
    let s = session();
    let create = ToolRegistry::call(
        "forge_blueprint_create",
        Some(json!({
            "name": "DepStress", "description": "dep stress", "domain": "api"
        })),
        &s,
    )
    .await
    .unwrap();
    let bp_id = extract_bp_id(&create);

    for i in 0..100 {
        ToolRegistry::call(
            "forge_dependency_add",
            Some(json!({
                "blueprint_id": bp_id,
                "name": format!("dep-{}", i),
                "version": format!("{}.0", i)
            })),
            &s,
        )
        .await
        .unwrap();
    }

    let get = ToolRegistry::call(
        "forge_blueprint_get",
        Some(json!({
            "blueprint_id": bp_id
        })),
        &s,
    )
    .await
    .unwrap();
    let text = match &get.content[0] {
        ToolContent::Text { text } => text.clone(),
    };
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["dependencies"].as_u64().unwrap(), 100);
}

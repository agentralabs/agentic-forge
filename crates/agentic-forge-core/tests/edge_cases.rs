//! Edge case tests — boundary conditions, malformed input, error paths.

use agentic_forge_core::engine::ForgeEngine;
use agentic_forge_core::engine::validator::BlueprintValidator;
use agentic_forge_core::format::{ForgeReader, ForgeWriter};
use agentic_forge_core::inventions::*;
use agentic_forge_core::storage::format::{ForgeFooter, ForgeHeader, SectionType};
use agentic_forge_core::storage::BlueprintStore;
use agentic_forge_core::types::blueprint::*;
use agentic_forge_core::types::ids::*;
use agentic_forge_core::types::intent::*;
use agentic_forge_core::types::*;
use std::io::Cursor;

// ── ID edge cases ────────────────────────────────────────────────────

#[test]
fn test_id_determinism_same_context() {
    let a = BlueprintId::from_context("same");
    let b = BlueprintId::from_context("same");
    assert_eq!(a, b, "Same context must produce identical IDs");
}

#[test]
fn test_id_different_contexts() {
    let a = BlueprintId::from_context("one");
    let b = BlueprintId::from_context("two");
    assert_ne!(a, b);
}

#[test]
fn test_id_empty_context() {
    let a = ForgeId::from_context("");
    let b = ForgeId::from_context("");
    assert_eq!(a, b, "Empty context should still be deterministic");
}

#[test]
fn test_id_unicode_context() {
    let id = EntityId::from_context("日本語テスト🔥");
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_id_very_long_context() {
    let long = "x".repeat(100_000);
    let id = ForgeId::from_context(&long);
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_id_parse_invalid() {
    let result: Result<ForgeId, _> = "not-a-uuid".parse();
    assert!(result.is_err());
}

#[test]
fn test_id_parse_raw_uuid() {
    let id = BlueprintId::new();
    let uuid_str = id.as_uuid().to_string();
    let parsed: BlueprintId = uuid_str.parse().unwrap();
    assert_eq!(id, parsed);
}

// ── Blueprint edge cases ─────────────────────────────────────────────

#[test]
fn test_blueprint_empty_name() {
    let bp = Blueprint::new("", "desc", Domain::Api);
    assert!(!bp.is_valid());
}

#[test]
fn test_blueprint_very_long_name() {
    let long_name = "A".repeat(10_000);
    let bp = Blueprint::new(&long_name, "desc", Domain::Api);
    assert_eq!(bp.name.len(), 10_000);
    let report = BlueprintValidator::validate(&bp).unwrap();
    assert!(!report.is_valid, "Name > 256 chars should fail validation");
}

#[test]
fn test_blueprint_unicode_name() {
    let bp = Blueprint::new("プロジェクト名🚀", "説明", Domain::Web);
    assert!(bp.is_valid());
    assert_eq!(bp.name, "プロジェクト名🚀");
}

#[test]
fn test_blueprint_special_chars_name() {
    let bp = Blueprint::new("test<>&\"'\\n\\t", "desc", Domain::Api);
    assert!(bp.is_valid());
}

#[test]
fn test_blueprint_serialization_roundtrip_with_all_fields() {
    let mut bp = Blueprint::new("Full", "All fields", Domain::Api);
    bp.entities.push(Entity::new("User", "A user"));
    bp.files.push(FileBlueprint::new("src/main.rs", FileType::Source));
    bp.dependencies.push(Dependency::new("serde", "1.0"));
    bp.test_cases.push(TestCase::new("test_a", TestType::Unit, "A"));
    bp.type_definitions.push(TypeDefinition::new("UserType", TypeKind::Struct));
    bp.function_blueprints.push(FunctionBlueprint::new("create_user"));
    bp.layers.push(ArchitectureLayer { name: "domain".into(), description: "".into(), modules: vec![], allowed_dependencies: vec![] });
    bp.concerns.push(CrossCuttingConcern { name: "logging".into(), concern_type: ConcernType::Logging, affected_layers: vec![], implementation_strategy: "".into() });
    bp.wiring.push(ComponentWiring { source: "A".into(), target: "B".into(), wiring_type: WiringType::DirectCall, description: "".into() });
    bp.data_flows.push(DataFlow { source: "A".into(), target: "B".into(), data_type: "X".into(), direction: FlowDirection::Unidirectional, is_async: false });
    bp.import_graph.push(ImportEdge { from_file: "a.rs".into(), to_file: "b.rs".into(), imported_symbols: vec!["Foo".into()] });
    bp.generation_order = vec!["types".into(), "main".into()];
    bp.metadata.insert("key".into(), "value".into());

    let json = serde_json::to_string(&bp).unwrap();
    let parsed: Blueprint = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "Full");
    assert_eq!(parsed.entities.len(), 1);
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(parsed.dependencies.len(), 1);
    assert_eq!(parsed.test_cases.len(), 1);
    assert_eq!(parsed.type_definitions.len(), 1);
    assert_eq!(parsed.function_blueprints.len(), 1);
    assert_eq!(parsed.layers.len(), 1);
    assert_eq!(parsed.concerns.len(), 1);
    assert_eq!(parsed.wiring.len(), 1);
    assert_eq!(parsed.data_flows.len(), 1);
    assert_eq!(parsed.import_graph.len(), 1);
    assert_eq!(parsed.generation_order.len(), 2);
    assert_eq!(parsed.metadata.get("key").unwrap(), "value");
}

// ── Engine edge cases ────────────────────────────────────────────────

#[test]
fn test_engine_operate_on_nonexistent_blueprint() {
    let mut engine = ForgeEngine::new();
    let fake_id = BlueprintId::new();
    assert!(engine.writer().rename_blueprint(&fake_id, "x").is_err());
    assert!(engine.writer().add_entity(&fake_id, Entity::new("A", "B")).is_err());
    assert!(engine.writer().add_file(&fake_id, FileBlueprint::new("a", FileType::Source)).is_err());
    assert!(engine.writer().add_dependency(&fake_id, Dependency::new("x", "1")).is_err());
    assert!(engine.writer().add_test_case(&fake_id, TestCase::new("t", TestType::Unit, "t")).is_err());
    assert!(engine.writer().delete_blueprint(&fake_id).is_err());
}

#[test]
fn test_engine_add_entity_with_empty_name() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    // Empty entity name is technically allowed at engine level; validator catches it
    let eid = engine.writer().add_entity(&id, Entity::new("", "empty")).unwrap();
    let bp = engine.store.load(&id).unwrap();
    assert_eq!(bp.find_entity_by_id(&eid).unwrap().name, "");
}

#[test]
fn test_engine_duplicate_entity_name() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    engine.writer().add_entity(&id, Entity::new("Dup", "first")).unwrap();
    let result = engine.writer().add_entity(&id, Entity::new("Dup", "second"));
    assert!(result.is_err(), "Duplicate entity name must error");
}

#[test]
fn test_engine_duplicate_dependency_name() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    engine.writer().add_dependency(&id, Dependency::new("serde", "1.0")).unwrap();
    let result = engine.writer().add_dependency(&id, Dependency::new("serde", "2.0"));
    assert!(result.is_err(), "Duplicate dependency name must error");
}

#[test]
fn test_engine_remove_nonexistent_entity() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    let fake_eid = EntityId::new();
    assert!(engine.writer().remove_entity(&id, &fake_eid).is_err());
}

#[test]
fn test_engine_remove_nonexistent_file() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    let fake_fid = FileId::new();
    assert!(engine.writer().remove_file(&id, &fake_fid).is_err());
}

#[test]
fn test_engine_remove_nonexistent_dependency() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    let fake_did = DependencyId::new();
    assert!(engine.writer().remove_dependency(&id, &fake_did).is_err());
}

#[test]
fn test_engine_remove_nonexistent_test_case() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    let fake_tid = TestCaseId::new();
    assert!(engine.writer().remove_test_case(&id, &fake_tid).is_err());
}

#[test]
fn test_engine_add_field_to_nonexistent_entity() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    let fake_eid = EntityId::new();
    let field = EntityField::new("x", FieldType::String);
    assert!(engine.writer().add_field_to_entity(&id, &fake_eid, field).is_err());
}

#[test]
fn test_engine_remove_field_nonexistent() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    let eid = engine.writer().add_entity(&id, Entity::new("X", "X")).unwrap();
    assert!(engine.writer().remove_field_from_entity(&id, &eid, "ghost").is_err());
}

#[test]
fn test_engine_update_dep_version_nonexistent() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("T", "T", Domain::Api).unwrap();
    let fake_did = DependencyId::new();
    assert!(engine.writer().update_dependency_version(&id, &fake_did, "9.9").is_err());
}

// ── Store edge cases ─────────────────────────────────────────────────

#[test]
fn test_store_persist_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.json");
    let mut store = BlueprintStore::with_path(&path);
    store.persist().unwrap();
    let loaded = BlueprintStore::load_from_disk(&path).unwrap();
    assert_eq!(loaded.count(), 0);
}

#[test]
fn test_store_load_corrupt_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    std::fs::write(&path, "not json at all!!!").unwrap();
    let result = BlueprintStore::load_from_disk(&path);
    assert!(result.is_err());
}

#[test]
fn test_store_load_wrong_json_shape() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wrong.json");
    std::fs::write(&path, r#"[1, 2, 3]"#).unwrap();
    let result = BlueprintStore::load_from_disk(&path);
    assert!(result.is_err());
}

// ── Format edge cases ────────────────────────────────────────────────

#[test]
fn test_format_header_wrong_version() {
    let mut header = ForgeHeader::new();
    header.version = 999;
    let mut buf = Vec::new();
    header.write_to(&mut buf).unwrap();
    // Manually patch magic to be correct
    buf[0..4].copy_from_slice(&FORGE_MAGIC);
    // But version is 999
    let result = ForgeHeader::read_from(&mut Cursor::new(&buf));
    assert!(result.is_err());
}

#[test]
fn test_format_footer_wrong_magic() {
    let mut buf = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00, // bad magic
                       0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                       0x00, 0x00, 0x00, 0x00];
    buf.extend_from_slice(&[0x00; 44]); // pad to 64
    let result = ForgeFooter::read_from(&mut Cursor::new(&buf));
    assert!(result.is_err());
}

#[test]
fn test_format_truncated_header() {
    let buf = vec![0x46, 0x52, 0x47, 0x45]; // just magic, nothing else
    let result = ForgeHeader::read_from(&mut Cursor::new(&buf));
    assert!(result.is_err());
}

#[test]
fn test_format_is_forge_file_too_short() {
    assert!(!ForgeReader::is_forge_file(&[]));
    assert!(!ForgeReader::is_forge_file(&[0x46]));
    assert!(!ForgeReader::is_forge_file(&[0x46, 0x52, 0x47]));
}

#[test]
fn test_format_write_read_roundtrip_large() {
    let mut blueprints = Vec::new();
    for i in 0..50 {
        let mut bp = Blueprint::new(&format!("Project_{}", i), "test", Domain::Api);
        for j in 0..10 {
            bp.entities.push(Entity::new(&format!("Entity_{}_{}", i, j), "entity"));
        }
        blueprints.push(bp);
    }
    let bytes = ForgeWriter::write_to_bytes(&blueprints).unwrap();
    let loaded = ForgeReader::read_from_bytes(&bytes).unwrap();
    assert_eq!(loaded.len(), 50);
    for (orig, load) in blueprints.iter().zip(loaded.iter()) {
        assert_eq!(orig.name, load.name);
        assert_eq!(orig.entities.len(), load.entities.len());
    }
}

#[test]
fn test_format_section_type_out_of_range() {
    assert!(SectionType::from_u8(12).is_none());
    assert!(SectionType::from_u8(255).is_none());
}

// ── Domain edge cases ────────────────────────────────────────────────

#[test]
fn test_domain_from_u8_all_valid() {
    for v in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255u8] {
        assert!(Domain::from_u8(v).is_some(), "Domain::from_u8({}) should be Some", v);
    }
}

#[test]
fn test_domain_from_u8_invalid() {
    for v in [10, 11, 50, 100, 200, 254u8] {
        assert!(Domain::from_u8(v).is_none(), "Domain::from_u8({}) should be None", v);
    }
}

#[test]
fn test_domain_from_name_case_insensitive() {
    assert_eq!(Domain::from_name("API"), Some(Domain::Api));
    assert_eq!(Domain::from_name("Web"), Some(Domain::Web));
    assert_eq!(Domain::from_name("CLI"), Some(Domain::Cli));
    assert_eq!(Domain::from_name("LIBRARY"), Some(Domain::Library));
}

#[test]
fn test_domain_from_name_unknown() {
    assert!(Domain::from_name("nonexistent").is_none());
    assert!(Domain::from_name("").is_none());
}

// ── Invention edge cases ─────────────────────────────────────────────

#[test]
fn test_entity_inferrer_empty_string() {
    let entities = EntityInferrer::infer("");
    assert!(!entities.is_empty(), "Should return default entity for empty input");
}

#[test]
fn test_entity_inferrer_all_noise() {
    let entities = EntityInferrer::infer("the quick brown fox jumps over the lazy dog");
    assert!(!entities.is_empty());
}

#[test]
fn test_entity_inferrer_repeated_entities() {
    let entities = EntityInferrer::infer("user user user user user");
    // Should only produce one User, not five
    let user_count = entities.iter().filter(|e| e.name == "User").count();
    assert_eq!(user_count, 1);
}

#[test]
fn test_field_deriver_unknown_entity() {
    let fields = FieldDeriver::derive_fields("UnknownXYZ", Domain::Api);
    // Should still produce at minimum: id, created_at, updated_at, name
    assert!(fields.len() >= 4);
}

#[test]
fn test_layer_decomposer_all_domains() {
    for v in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255u8] {
        let domain = Domain::from_u8(v).unwrap();
        let layers = LayerDecomposer::decompose(domain);
        assert!(!layers.is_empty(), "Domain {:?} must produce at least one layer", domain);
    }
}

#[test]
fn test_skeleton_generator_entity_with_no_fields() {
    let entity = Entity::new("Empty", "No fields");
    let skeleton = SkeletonGenerator::generate(&entity);
    assert!(skeleton.contains("pub struct Empty"));
}

#[test]
fn test_skeleton_generator_entity_with_optional_field() {
    let mut entity = Entity::new("Test", "test");
    entity.fields.push(EntityField {
        name: "optional_field".into(),
        field_type: FieldType::String,
        required: false,
        default_value: None,
        description: String::new(),
    });
    let skeleton = SkeletonGenerator::generate(&entity);
    assert!(skeleton.contains("Option<"));
}

#[test]
fn test_dependency_inferrer_all_domains() {
    for v in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255u8] {
        let domain = Domain::from_u8(v).unwrap();
        let deps = DependencyInferrer::infer(domain, &[], &[]);
        // Every domain should at least get serde + serde_json + thiserror
        assert!(deps.len() >= 3, "Domain {:?} must have at least 3 deps, got {}", domain, deps.len());
    }
}

#[test]
fn test_api_spec_extractor_non_api_domains() {
    for domain in [Domain::Cli, Domain::Library, Domain::Embedded, Domain::Desktop] {
        let endpoints = ApiSpecExtractor::extract(&[Entity::new("X", "X")], domain);
        assert!(endpoints.is_empty(), "Domain {:?} should not produce API endpoints", domain);
    }
}

#[test]
fn test_conflict_resolver_no_deps() {
    let conflicts = ConflictResolver::resolve(&[]);
    assert!(conflicts.is_empty());
}

#[test]
fn test_conflict_resolver_single_dep() {
    let conflicts = ConflictResolver::resolve(&[Dependency::new("a", "1.0")]);
    assert!(conflicts.is_empty());
}

#[test]
fn test_test_case_generator_entity_with_many_fields() {
    let mut entity = Entity::new("Big", "big entity");
    for i in 0..20 {
        entity.fields.push(EntityField::new(&format!("field_{}", i), FieldType::String));
    }
    let tests = TestCaseGenerator::generate(&entity);
    // Should have CRUD tests + one requires_ test per required field + not_found + serialization
    assert!(tests.len() >= 27, "Expected 27+ tests, got {}", tests.len());
}

#[test]
fn test_wiring_builder_empty_entities() {
    let wirings = WiringDiagramBuilder::build(&[], &[]);
    assert!(wirings.is_empty());
}

#[test]
fn test_init_sequencer_empty_blueprint() {
    let bp = Blueprint::new("Empty", "empty", Domain::Library);
    let steps = InitSequencer::sequence(&bp);
    // Should always have config + logging at minimum
    assert!(steps.len() >= 2);
}

#[test]
fn test_shutdown_sequencer_empty_blueprint() {
    let bp = Blueprint::new("Empty", "empty", Domain::Library);
    let steps = ShutdownSequencer::sequence(&bp);
    assert!(!steps.is_empty(), "Should always flush logs");
}

#[test]
fn test_generation_planner_empty_blueprint() {
    let bp = Blueprint::new("Empty", "empty", Domain::Library);
    let steps = GenerationPlanner::plan(&bp);
    assert!(!steps.is_empty());
    assert_eq!(steps[0].name, "types");
}

#[test]
fn test_mock_specifier_empty() {
    let mocks = MockSpecifier::specify(&[]);
    assert!(mocks.is_empty());
}

#[test]
fn test_integration_test_planner_no_relationships() {
    let entities = vec![Entity::new("Solo", "no rels")];
    let plans = IntegrationTestPlanner::plan(&entities);
    assert_eq!(plans.len(), 1);
    assert!(plans[0].name.contains("crud"));
}

// ── Validator edge cases ─────────────────────────────────────────────

#[test]
fn test_validator_duplicate_file_paths() {
    let mut bp = Blueprint::new("Dup", "dup", Domain::Api);
    bp.files.push(FileBlueprint::new("src/main.rs", FileType::Source));
    bp.files.push(FileBlueprint::new("src/main.rs", FileType::Source));
    let report = BlueprintValidator::validate(&bp).unwrap();
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.contains("Duplicate file")));
}

#[test]
fn test_validator_duplicate_dependency_names() {
    let mut bp = Blueprint::new("Dup", "dup", Domain::Api);
    bp.dependencies.push(Dependency::new("serde", "1.0"));
    bp.dependencies.push(Dependency::new("serde", "2.0"));
    let report = BlueprintValidator::validate(&bp).unwrap();
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.contains("Duplicate dependency")));
}

#[test]
fn test_validator_entity_with_duplicate_fields() {
    let mut bp = Blueprint::new("Dup", "dup", Domain::Api);
    let mut entity = Entity::new("User", "A user");
    entity.fields.push(EntityField::new("name", FieldType::String));
    entity.fields.push(EntityField::new("name", FieldType::String));
    bp.entities.push(entity);
    let report = BlueprintValidator::validate(&bp).unwrap();
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| e.contains("duplicate field")));
}

#[test]
fn test_validator_wiring_empty_source_target() {
    let mut bp = Blueprint::new("Bad", "bad", Domain::Api);
    bp.wiring.push(ComponentWiring {
        source: "".into(),
        target: "".into(),
        wiring_type: WiringType::DirectCall,
        description: "".into(),
    });
    let report = BlueprintValidator::validate(&bp).unwrap();
    assert!(!report.is_valid);
}

#[test]
fn test_validator_data_flow_empty() {
    let mut bp = Blueprint::new("Bad", "bad", Domain::Api);
    bp.data_flows.push(DataFlow {
        source: "".into(),
        target: "".into(),
        data_type: "X".into(),
        direction: FlowDirection::Unidirectional,
        is_async: false,
    });
    let report = BlueprintValidator::validate(&bp).unwrap();
    assert!(!report.is_valid);
}

// ── Invention count ──────────────────────────────────────────────────

#[test]
fn test_invention_count_matches() {
    assert_eq!(agentic_forge_core::types::INVENTION_COUNT, 32);
    let names = all_invention_names();
    assert_eq!(names.len(), 32, "all_invention_names() must return exactly 32");
}

#[test]
fn test_invention_names_unique() {
    let names = all_invention_names();
    let mut seen = std::collections::HashSet::new();
    for name in &names {
        assert!(seen.insert(name), "Duplicate invention name: {}", name);
    }
}

#[test]
fn test_mcp_tool_count_matches() {
    assert_eq!(agentic_forge_core::types::MCP_TOOL_COUNT, 15);
}

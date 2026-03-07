//! Stress tests — concurrent access, large datasets, performance bounds.

use agentic_forge_core::engine::ForgeEngine;
use agentic_forge_core::engine::validator::BlueprintValidator;
use agentic_forge_core::format::{ForgeReader, ForgeWriter};
use agentic_forge_core::inventions::*;
use agentic_forge_core::storage::BlueprintStore;
use agentic_forge_core::types::blueprint::*;
use agentic_forge_core::types::ids::*;
use agentic_forge_core::types::intent::*;
use std::time::Instant;

// ── Large dataset stress ─────────────────────────────────────────────

#[test]
fn test_stress_100_blueprints() {
    let mut engine = ForgeEngine::new();
    let start = Instant::now();
    let mut ids = Vec::new();
    for i in 0..100 {
        let id = engine.create_blueprint(
            &format!("Project_{}", i),
            &format!("Description for project {}", i),
            Domain::Api,
        ).unwrap();
        ids.push(id);
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "100 blueprints took {}ms, want <1000ms", elapsed.as_millis());
    assert_eq!(engine.blueprint_count(), 100);

    // Verify all accessible
    for id in &ids {
        let bp = engine.store.load(id).unwrap();
        assert!(bp.name.starts_with("Project_"));
    }
}

#[test]
fn test_stress_1000_entities_in_one_blueprint() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("Big", "stress test", Domain::Api).unwrap();

    let start = Instant::now();
    for i in 0..1000 {
        engine.writer().add_entity(&id, Entity::new(
            &format!("Entity_{}", i),
            &format!("Entity number {}", i),
        )).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 5000, "1000 entities took {}ms, want <5000ms", elapsed.as_millis());

    let bp = engine.store.load(&id).unwrap();
    assert_eq!(bp.entity_count(), 1000);
}

#[test]
fn test_stress_1000_files_in_one_blueprint() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("ManyFiles", "stress test", Domain::Api).unwrap();

    let start = Instant::now();
    for i in 0..1000 {
        engine.writer().add_file(&id, FileBlueprint::new(
            &format!("src/module_{}/file_{}.rs", i / 10, i),
            FileType::Source,
        )).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 5000, "1000 files took {}ms", elapsed.as_millis());

    let bp = engine.store.load(&id).unwrap();
    assert_eq!(bp.file_count(), 1000);
}

#[test]
fn test_stress_500_dependencies() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("ManyDeps", "stress test", Domain::Api).unwrap();

    for i in 0..500 {
        engine.writer().add_dependency(&id, Dependency::new(
            &format!("dep-{}", i),
            &format!("{}.0.0", i),
        )).unwrap();
    }

    let bp = engine.store.load(&id).unwrap();
    assert_eq!(bp.dependency_count(), 500);
}

#[test]
fn test_stress_500_test_cases() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("ManyTests", "stress test", Domain::Api).unwrap();

    for i in 0..500 {
        engine.writer().add_test_case(&id, TestCase::new(
            &format!("test_{}", i),
            TestType::Unit,
            &format!("target_{}", i),
        )).unwrap();
    }

    let bp = engine.store.load(&id).unwrap();
    assert_eq!(bp.test_count(), 500);
}

// ── Rapid create/delete cycle ────────────────────────────────────────

#[test]
fn test_stress_create_delete_cycle() {
    let mut engine = ForgeEngine::new();

    for i in 0..200 {
        let id = engine.create_blueprint(&format!("Temp_{}", i), "temp", Domain::Cli).unwrap();
        engine.writer().delete_blueprint(&id).unwrap();
    }
    assert_eq!(engine.blueprint_count(), 0, "All blueprints should be deleted");
}

#[test]
fn test_stress_add_remove_entities_cycle() {
    let mut engine = ForgeEngine::new();
    let id = engine.create_blueprint("Cycle", "cycle test", Domain::Api).unwrap();

    for round in 0..50 {
        let mut eids = Vec::new();
        for j in 0..10 {
            let eid = engine.writer().add_entity(&id, Entity::new(
                &format!("E_{}_{}", round, j),
                "temp",
            )).unwrap();
            eids.push(eid);
        }
        for eid in eids {
            engine.writer().remove_entity(&id, &eid).unwrap();
        }
    }

    let bp = engine.store.load(&id).unwrap();
    assert_eq!(bp.entity_count(), 0, "All entities should be removed after cycles");
}

// ── Format stress ────────────────────────────────────────────────────

#[test]
fn test_stress_format_large_roundtrip() {
    let mut blueprints = Vec::new();
    for i in 0..20 {
        let mut bp = Blueprint::new(&format!("Heavy_{}", i), "stress", Domain::Api);
        for j in 0..50 {
            let mut entity = Entity::new(&format!("E_{}_{}", i, j), "ent");
            for k in 0..5 {
                entity.fields.push(EntityField::new(&format!("f_{}", k), FieldType::String));
            }
            bp.entities.push(entity);
        }
        for j in 0..20 {
            bp.files.push(FileBlueprint::new(&format!("src/mod_{}/file_{}.rs", i, j), FileType::Source));
        }
        for j in 0..10 {
            bp.dependencies.push(Dependency::new(&format!("dep_{}_{}", i, j), "1.0"));
        }
        blueprints.push(bp);
    }

    let start = Instant::now();
    let bytes = ForgeWriter::write_to_bytes(&blueprints).unwrap();
    let write_elapsed = start.elapsed();

    let start = Instant::now();
    let loaded = ForgeReader::read_from_bytes(&bytes).unwrap();
    let read_elapsed = start.elapsed();

    assert_eq!(loaded.len(), 20);
    assert!(write_elapsed.as_millis() < 5000, "Write took {}ms", write_elapsed.as_millis());
    assert!(read_elapsed.as_millis() < 5000, "Read took {}ms", read_elapsed.as_millis());

    // Verify data integrity
    for (orig, load) in blueprints.iter().zip(loaded.iter()) {
        assert_eq!(orig.name, load.name);
        assert_eq!(orig.entity_count(), load.entity_count());
        assert_eq!(orig.file_count(), load.file_count());
        assert_eq!(orig.dependency_count(), load.dependency_count());
    }
}

#[test]
fn test_stress_format_file_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stress.forge");

    let mut bp = Blueprint::new("FileStress", "test", Domain::Api);
    for i in 0..100 {
        bp.entities.push(Entity::new(&format!("E_{}", i), "x"));
    }

    ForgeWriter::write_to_file(&[bp.clone()], &path).unwrap();
    let loaded = ForgeReader::read_from_file(&path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].entity_count(), 100);
}

// ── Store persistence stress ─────────────────────────────────────────

#[test]
fn test_stress_store_persist_reload_many() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store_stress.json");

    let mut store = BlueprintStore::with_path(&path);
    for i in 0..100 {
        store.save(Blueprint::new(&format!("BP_{}", i), "test", Domain::Api)).unwrap();
    }
    store.persist().unwrap();

    let loaded = BlueprintStore::load_from_disk(&path).unwrap();
    assert_eq!(loaded.count(), 100);
}

#[test]
fn test_stress_store_repeated_persist() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("repeat.json");

    let mut store = BlueprintStore::with_path(&path);
    for round in 0..20 {
        store.save(Blueprint::new(&format!("R_{}", round), "test", Domain::Cli)).unwrap();
        store.persist().unwrap();
    }

    let loaded = BlueprintStore::load_from_disk(&path).unwrap();
    assert_eq!(loaded.count(), 20);
}

// ── Invention pipeline stress ────────────────────────────────────────

#[test]
fn test_stress_full_invention_pipeline() {
    let start = Instant::now();

    // Tier 1: Decomposition
    let layers = LayerDecomposer::decompose(Domain::Api);
    let intent = IntentSpec::new("StressProject", "A large API with many entities", Domain::Api);
    let concerns = ConcernAnalyzer::analyze(&intent);

    // Tier 2: Entity inference
    let entity_specs = EntityInferrer::infer(
        "users create posts with comments, tags, categories, products, orders, payments, notifications, settings"
    );
    let mut entities: Vec<Entity> = Vec::new();
    for spec in &entity_specs {
        let mut entity = Entity::new(&spec.name, &spec.description);
        let fields = FieldDeriver::derive_fields(&spec.name, Domain::Api);
        entity.fields = fields.into_iter().map(|f| f).collect();
        entities.push(entity);
    }

    // Tier 3: Operations
    let mut all_ops = Vec::new();
    for entity in &entities {
        let ops = OperationInferrer::infer_operations(entity);
        all_ops.extend(ops);
    }

    // Tier 4: Structure
    let mut bp = Blueprint::new("StressProject", "stress", Domain::Api);
    bp.entities = entities.clone();
    bp.layers = layers;
    bp.concerns = concerns;
    let files = FileStructureGenerator::generate(&bp);
    let import_graph = ImportGraphGenerator::generate(&files);

    // Tier 5: Dependencies
    let deps = DependencyInferrer::infer(Domain::Api, &entities, &[]);
    let resolved = VersionResolver::resolve(&deps);

    // Tier 6: Blueprint generation
    let type_defs = TypeFirstMaterializer::materialize(&entities);
    let contracts = ContractSpecifier::specify(&entities);
    let plan = GenerationPlanner::plan(&bp);

    // Tier 7: Integration
    let wirings = WiringDiagramBuilder::build(&entities, &bp.layers);
    let flows = DataFlowSpecifier::specify(&entities, true);
    let init_seq = InitSequencer::sequence(&bp);
    let shutdown_seq = ShutdownSequencer::sequence(&bp);

    // Tier 8: Tests
    let mut test_cases = Vec::new();
    for entity in &entities {
        test_cases.extend(TestCaseGenerator::generate(entity));
        TestFixtureDesigner::design(entity);
    }
    let integration_plans = IntegrationTestPlanner::plan(&entities);
    let mocks = MockSpecifier::specify(&entities);

    let elapsed = start.elapsed();

    // Verify pipeline produced results
    assert!(!entities.is_empty());
    assert!(!all_ops.is_empty());
    assert!(!files.is_empty());
    assert!(!deps.is_empty());
    assert!(!resolved.is_empty());
    assert!(!type_defs.is_empty());
    assert!(!contracts.is_empty());
    assert!(!plan.is_empty());
    assert!(!wirings.is_empty());
    assert!(!flows.is_empty());
    assert!(!init_seq.is_empty());
    assert!(!shutdown_seq.is_empty());
    assert!(!test_cases.is_empty());
    assert!(!integration_plans.is_empty());
    assert!(!mocks.is_empty());

    assert!(elapsed.as_millis() < 2000, "Full pipeline took {}ms, want <2000ms", elapsed.as_millis());
}

#[test]
fn test_stress_pipeline_repeated() {
    for _ in 0..50 {
        let entities = EntityInferrer::infer("users and posts with tags");
        let deps = DependencyInferrer::infer(Domain::Api, &[], &[]);
        let layers = LayerDecomposer::decompose(Domain::Api);
        assert!(!entities.is_empty());
        assert!(!deps.is_empty());
        assert!(!layers.is_empty());
    }
}

// ── Validation stress ────────────────────────────────────────────────

#[test]
fn test_stress_validate_large_blueprint() {
    let mut bp = Blueprint::new("Validated", "stress", Domain::Api);
    for i in 0..200 {
        let mut entity = Entity::new(&format!("Entity_{}", i), "entity");
        for j in 0..10 {
            entity.fields.push(EntityField::new(&format!("field_{}", j), FieldType::String));
        }
        bp.entities.push(entity);
    }
    for i in 0..500 {
        bp.files.push(FileBlueprint::new(&format!("src/file_{}.rs", i), FileType::Source));
    }
    for i in 0..100 {
        bp.dependencies.push(Dependency::new(&format!("dep_{}", i), "1.0"));
    }

    let start = Instant::now();
    let report = BlueprintValidator::validate(&bp).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 1000, "Validation took {}ms", elapsed.as_millis());
    // Should be valid since all names are unique and non-empty
    assert!(report.is_valid, "Errors: {:?}", report.errors);
}

// ── Concurrent-safe ID generation ────────────────────────────────────

#[test]
fn test_stress_id_uniqueness() {
    let mut ids = std::collections::HashSet::new();
    for _ in 0..10_000 {
        let id = BlueprintId::new();
        assert!(ids.insert(id), "Duplicate ID generated!");
    }
    assert_eq!(ids.len(), 10_000);
}

#[test]
fn test_stress_mixed_id_types() {
    let mut all = std::collections::HashSet::new();
    for _ in 0..1000 {
        all.insert(format!("{}", ForgeId::new()));
        all.insert(format!("{}", BlueprintId::new()));
        all.insert(format!("{}", EntityId::new()));
        all.insert(format!("{}", OperationId::new()));
        all.insert(format!("{}", FileId::new()));
        all.insert(format!("{}", DependencyId::new()));
        all.insert(format!("{}", TestCaseId::new()));
    }
    assert_eq!(all.len(), 7000, "All 7000 IDs should be unique");
}

// ── Query engine under load ──────────────────────────────────────────

#[test]
fn test_stress_query_search() {
    let mut engine = ForgeEngine::new();
    for i in 0..100 {
        engine.create_blueprint(&format!("Project_{}", i), &format!("desc {}", i), Domain::Api).unwrap();
    }

    let start = Instant::now();
    let r = engine.reader();
    for _ in 0..1000 {
        let results = r.search_blueprints("Project_5");
        assert!(!results.is_empty());
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 5000, "1000 searches took {}ms", elapsed.as_millis());
}

#[test]
fn test_stress_blueprint_summary_many() {
    let mut engine = ForgeEngine::new();
    let mut ids = Vec::new();
    for i in 0..50 {
        let id = engine.create_blueprint(&format!("P_{}", i), "d", Domain::Api).unwrap();
        for j in 0..5 {
            engine.writer().add_entity(&id, Entity::new(&format!("E_{}_{}", i, j), "e")).unwrap();
        }
        ids.push(id);
    }

    let start = Instant::now();
    let r = engine.reader();
    for id in &ids {
        let summary = r.blueprint_summary(id).unwrap();
        assert_eq!(summary.entity_count, 5);
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 2000, "50 summaries took {}ms", elapsed.as_millis());
}

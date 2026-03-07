use agentic_forge_core::engine::ForgeEngine;
use agentic_forge_core::types::blueprint::*;
use agentic_forge_core::types::intent::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_create_blueprint(c: &mut Criterion) {
    c.bench_function("create_blueprint", |b| {
        b.iter(|| {
            let mut engine = ForgeEngine::new();
            black_box(
                engine
                    .create_blueprint("Test", "desc", Domain::Api)
                    .unwrap(),
            );
        })
    });
}

fn bench_add_entity(c: &mut Criterion) {
    let mut engine = ForgeEngine::new();
    let id = engine
        .create_blueprint("Bench", "bench", Domain::Api)
        .unwrap();

    c.bench_function("add_entity", |b| {
        let mut i = 0u64;
        b.iter(|| {
            i += 1;
            let entity = Entity::new(&format!("E_{}", i), "bench");
            black_box(engine.writer().add_entity(&id, entity).unwrap());
        })
    });
}

fn bench_blueprint_serialization(c: &mut Criterion) {
    let mut bp = Blueprint::new("SerBench", "bench", Domain::Api);
    for i in 0..50 {
        bp.entities.push(Entity::new(&format!("E_{}", i), "e"));
    }

    c.bench_function("blueprint_serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&bp).unwrap());
        })
    });
}

criterion_group!(
    benches,
    bench_create_blueprint,
    bench_add_entity,
    bench_blueprint_serialization
);
criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use agentic_forge_core::cache::Cache;
use std::time::Duration;

fn bench_cache_insert(c: &mut Criterion) {
    let cache: Cache<String, String> = Cache::new(10000, Duration::from_secs(300));

    c.bench_function("cache_insert", |b| {
        let mut i = 0u64;
        b.iter(|| {
            i += 1;
            cache.insert(format!("key_{}", i), format!("val_{}", i));
        })
    });
}

fn bench_cache_get_hit(c: &mut Criterion) {
    let cache: Cache<String, String> = Cache::new(10000, Duration::from_secs(300));
    cache.insert("hit_key".into(), "value".into());

    c.bench_function("cache_get_hit", |b| {
        b.iter(|| {
            black_box(cache.get(&"hit_key".into()));
        })
    });
}

fn bench_cache_get_miss(c: &mut Criterion) {
    let cache: Cache<String, String> = Cache::new(10000, Duration::from_secs(300));

    c.bench_function("cache_get_miss", |b| {
        b.iter(|| {
            black_box(cache.get(&"miss_key".into()));
        })
    });
}

criterion_group!(benches, bench_cache_insert, bench_cache_get_hit, bench_cache_get_miss);
criterion_main!(benches);

//! Conservation tests — verifying the Token Conservation Architecture.
//! Every test here validates a guarantee from the Sacred Covenant.

use agentic_forge_core::cache::Cache;
use agentic_forge_core::cache::CacheInvalidator;
use agentic_forge_core::metrics::tokens::{Layer, TokenMetrics, ResponseMetrics};
use agentic_forge_core::metrics::audit::{AuditLog, AuditEntry};
use agentic_forge_core::metrics::conservation::{generate_report, ConservationVerdict};
use agentic_forge_core::query::intent::ExtractionIntent;
use agentic_forge_core::query::delta::{VersionedState, ChangeType};
use agentic_forge_core::query::budget::TokenBudget;
use agentic_forge_core::query::pagination::CursorPage;
use std::time::Duration;

// ── RULE 2: Second query MUST be cheaper (cache hit) ─────────────────

#[test]
fn test_second_query_cheaper_than_first() {
    let cache: Cache<String, Vec<u8>> = Cache::new(100, Duration::from_secs(60));
    let metrics = TokenMetrics::new();

    // First query: full cost (cache miss)
    let key = "blueprint_query_X".to_string();
    let data = vec![0u8; 5000]; // Simulate 5000-byte result
    assert!(cache.get(&key).is_none());
    metrics.record(Layer::Full, 500, 500);
    cache.insert(key.clone(), data.clone());

    let first_total = metrics.total_tokens();

    // Second query: should be cache hit (0 tokens)
    let cached = cache.get(&key);
    assert!(cached.is_some());
    metrics.record(Layer::Cache, 0, 500);

    let second_cost = metrics.total_tokens() - first_total;
    assert_eq!(second_cost, 0, "Second query must be 0 tokens (cache hit)");
    assert!(metrics.total_savings() >= 500, "Must save >=500 tokens on cache hit");
}

// ── RULE 9: Unchanged state MUST be free to query ────────────────────

#[test]
fn test_unchanged_state_free() {
    let mut state = VersionedState::new();
    state.record_change("entity_1", ChangeType::Created);
    state.record_change("entity_2", ChangeType::Created);

    let current_version = state.version();

    // No changes since current version
    let changes = state.changes_since_version(current_version);
    assert!(changes.is_empty(), "No changes = empty delta");
    assert!(state.is_unchanged_since(current_version));
}

// ── RULE 10: Cost scales with answer complexity, not source size ─────

#[test]
fn test_scoped_query_10x_cheaper() {
    let ids_cost = ExtractionIntent::IdsOnly.estimated_tokens();
    let full_cost = ExtractionIntent::Full.estimated_tokens();
    assert!(ids_cost * 10 <= full_cost,
        "IdsOnly ({}) must be >=10x cheaper than Full ({})", ids_cost, full_cost);

    let exists_cost = ExtractionIntent::Exists.estimated_tokens();
    assert!(exists_cost * 100 <= full_cost,
        "Exists ({}) must be >=100x cheaper than Full ({})", exists_cost, full_cost);
}

// ── RULE 4: Delta retrieval proportional to changes ──────────────────

#[test]
fn test_delta_proportional_to_changes() {
    let mut state = VersionedState::new();

    // Create 1000 items
    for i in 0..1000 {
        state.record_change(format!("item_{}", i), ChangeType::Created);
    }
    let v_before = state.version();

    // Add 2 more items
    state.record_change("item_1000", ChangeType::Created);
    state.record_change("item_1001", ChangeType::Created);

    let all = state.changes_since_version(0);
    let delta = state.changes_since_version(v_before);

    assert_eq!(all.len(), 1002);
    assert_eq!(delta.len(), 2);

    // Delta is 501x cheaper
    assert!(delta.len() < all.len() / 100,
        "Delta ({}) must be <1% of full ({})", delta.len(), all.len());
}

// ── RULE 7: Cache populated after expensive operations ───────────────

#[test]
fn test_cache_populated_after_full_extraction() {
    let cache: Cache<String, String> = Cache::new(100, Duration::from_secs(60));

    // Full extraction
    let result = "expensive_computation_result".to_string();
    cache.insert("query_key".into(), result.clone());

    // Next access is a cache hit
    let cached = cache.get(&"query_key".into());
    assert_eq!(cached, Some(result));
    assert_eq!(cache.metrics.hits(), 1);
}

// ── RULE 3: Index before scan ────────────────────────────────────────

#[test]
fn test_index_lookup_cheaper_than_scan() {
    let metrics = TokenMetrics::new();

    // Index lookup: ~10 tokens
    metrics.record(Layer::Index, 10, 500);

    // Full scan would have been 500 tokens
    assert_eq!(metrics.total_tokens(), 10);
    assert_eq!(metrics.total_savings(), 0); // Index doesn't track savings itself
}

// ── RULE 6: Log token cost for every MCP call ────────────────────────

#[test]
fn test_audit_every_call() {
    let audit = AuditLog::new(1000);

    for i in 0..20 {
        audit.record(AuditEntry {
            timestamp: chrono::Utc::now().timestamp_micros(),
            tool: format!("forge_tool_{}", i % 5),
            layer: if i % 3 == 0 { Layer::Cache } else { Layer::Full },
            tokens_used: if i % 3 == 0 { 0 } else { 500 },
            tokens_saved: if i % 3 == 0 { 500 } else { 0 },
            cache_hit: i % 3 == 0,
            intent: "test".into(),
            source_size: 10000,
            result_size: if i % 3 == 0 { 0 } else { 500 },
        });
    }

    assert_eq!(audit.len(), 20, "Must log every call");
    assert!(audit.cache_hit_rate() > 0.3);
    assert!(audit.total_tokens_saved() > 0);
}

// ── Conservation score must improve with warm cache ──────────────────

#[test]
fn test_conservation_score_improves_with_warmup() {
    let metrics = TokenMetrics::new();
    let audit = AuditLog::new(100);

    // Cold phase: 10 full queries
    for _ in 0..10 {
        metrics.record(Layer::Full, 500, 500);
    }
    let cold_report = generate_report(&metrics, &audit);
    assert_eq!(cold_report.verdict, ConservationVerdict::Wasteful,
        "Cold cache should be wasteful");

    // Warm phase: 30 cache hits
    for _ in 0..30 {
        metrics.record(Layer::Cache, 0, 500);
    }
    let warm_report = generate_report(&metrics, &audit);

    assert!(warm_report.score > cold_report.score,
        "Score must improve: cold={} warm={}", cold_report.score, warm_report.score);
    assert!(warm_report.score >= 0.7,
        "After 3:1 cache hit ratio, score should be >=0.7: {}", warm_report.score);
}

// ── Conservation score >= 0.7 after warmup ───────────────────────────

#[test]
fn test_conservation_target_07() {
    let metrics = TokenMetrics::new();
    let audit = AuditLog::new(100);

    // 5 cold + 15 cache hits = 75% from cache
    for _ in 0..5 {
        metrics.record(Layer::Full, 500, 500);
    }
    for _ in 0..15 {
        metrics.record(Layer::Cache, 0, 500);
    }

    let report = generate_report(&metrics, &audit);
    assert!(report.score >= 0.7,
        "Conservation score should be >= 0.7 after warmup: {}", report.score);
}

// ── Token budget enforcement ─────────────────────────────────────────

#[test]
fn test_token_budget_enforced() {
    let mut budget = TokenBudget::new(1000);
    assert!(budget.spend(500));
    assert!(budget.spend(400));
    assert!(!budget.spend(200), "Should reject when over budget");
    assert!(budget.spend(100)); // Exact remaining
    assert!(budget.is_exhausted());
}

// ── Default response is minimal ──────────────────────────────────────

#[test]
fn test_default_intent_is_minimal() {
    let default = ExtractionIntent::default();
    assert_eq!(default, ExtractionIntent::IdsOnly);
    assert!(default.is_minimal());
    assert!(!default.includes_content());
    assert!(default.estimated_tokens() <= 10);
}

// ── Cursor pagination ────────────────────────────────────────────────

#[test]
fn test_pagination_limits_response_size() {
    let data: Vec<u32> = (0..1000).collect();

    let page = CursorPage::from_slice(data, None, 10);
    assert_eq!(page.len(), 10, "Must respect max_results");
    assert!(page.has_more);
}

// ── Cache invalidation on mutation ───────────────────────────────────

#[test]
fn test_cache_invalidation_on_mutation() {
    let cache: Cache<String, String> = Cache::new(100, Duration::from_secs(60));
    let inv: CacheInvalidator<String> = CacheInvalidator::new();

    // Populate cache
    cache.insert("blueprint_list".into(), "cached_list".into());
    cache.insert("blueprint_X".into(), "cached_X".into());

    // Register dependency: list depends on any blueprint
    inv.register_dependency("blueprint_list".into(), "blueprint_X".into());

    // Mutate blueprint X → should invalidate both
    let count = inv.invalidate_cascade(&"blueprint_X".into(), &cache);
    assert_eq!(count, 2);
    assert!(cache.get(&"blueprint_X".into()).is_none());
    assert!(cache.get(&"blueprint_list".into()).is_none());
}

// ── ResponseMetrics ──────────────────────────────────────────────────

#[test]
fn test_response_metrics_from_cache() {
    let rm = ResponseMetrics::from_cache(500);
    assert!(rm.cache_hit);
    assert_eq!(rm.tokens_used, 0);
    assert_eq!(rm.tokens_saved, 500);
    assert_eq!(rm.layer, Layer::Cache);
}

#[test]
fn test_response_metrics_scoped() {
    let rm = ResponseMetrics::from_query(Layer::Scoped, 50, 500);
    assert!(!rm.cache_hit);
    assert_eq!(rm.tokens_used, 50);
    assert_eq!(rm.tokens_saved, 450);
}

// ── End-to-end conservation flow ─────────────────────────────────────

#[test]
fn test_end_to_end_conservation_flow() {
    let cache: Cache<String, String> = Cache::new(100, Duration::from_secs(60));
    let metrics = TokenMetrics::new();
    let audit = AuditLog::new(100);

    // Simulate 50 tool calls with realistic patterns
    for i in 0..50 {
        let key = format!("query_{}", i % 10); // 10 unique queries, repeated

        if let Some(_cached) = cache.get(&key) {
            // Cache hit
            metrics.record(Layer::Cache, 0, 500);
            audit.record(AuditEntry {
                timestamp: i as i64, tool: "forge_blueprint_get".into(),
                layer: Layer::Cache, tokens_used: 0, tokens_saved: 500,
                cache_hit: true, intent: "summary".into(),
                source_size: 10000, result_size: 0,
            });
        } else {
            // Cache miss → full extraction + populate cache
            metrics.record(Layer::Full, 500, 500);
            cache.insert(key, "result_data".into());
            audit.record(AuditEntry {
                timestamp: i as i64, tool: "forge_blueprint_get".into(),
                layer: Layer::Full, tokens_used: 500, tokens_saved: 0,
                cache_hit: false, intent: "full".into(),
                source_size: 10000, result_size: 500,
            });
        }
    }

    let report = generate_report(&metrics, &audit);

    // 10 unique queries = 10 misses, 40 hits
    assert!(report.score >= 0.7,
        "With 80% cache hit rate, score should be >= 0.7: {}", report.score);
    assert!(report.cache_hit_rate >= 0.7,
        "Cache hit rate should be >= 0.7: {}", report.cache_hit_rate);
    assert_eq!(report.total_tokens, 5000); // 10 misses × 500
    assert_eq!(report.total_savings, 20000); // 40 hits × 500
}

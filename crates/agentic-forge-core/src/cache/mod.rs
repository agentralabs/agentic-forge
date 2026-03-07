//! Caching layer — LRU cache with TTL, invalidation, metrics.

pub mod lru;
pub mod invalidation;
pub mod metrics;

pub use lru::Cache;
pub use invalidation::CacheInvalidator;
pub use metrics::CacheMetrics;

//! Per-call token tracking.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Cache,
    Index,
    Scoped,
    Delta,
    Full,
}

impl Layer {
    pub fn number(&self) -> u8 {
        match self {
            Self::Cache => 0,
            Self::Index => 1,
            Self::Scoped => 2,
            Self::Delta => 3,
            Self::Full => 4,
        }
    }
}

pub struct TokenMetrics {
    pub total: AtomicU64,
    pub layer0_cache: AtomicU64,
    pub layer1_index: AtomicU64,
    pub layer2_scoped: AtomicU64,
    pub layer3_delta: AtomicU64,
    pub layer4_full: AtomicU64,
    pub cache_savings: AtomicU64,
    pub scope_savings: AtomicU64,
    pub delta_savings: AtomicU64,
}

impl TokenMetrics {
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            layer0_cache: AtomicU64::new(0),
            layer1_index: AtomicU64::new(0),
            layer2_scoped: AtomicU64::new(0),
            layer3_delta: AtomicU64::new(0),
            layer4_full: AtomicU64::new(0),
            cache_savings: AtomicU64::new(0),
            scope_savings: AtomicU64::new(0),
            delta_savings: AtomicU64::new(0),
        }
    }

    pub fn record(&self, layer: Layer, tokens: u64, potential: u64) {
        self.total.fetch_add(tokens, Ordering::Relaxed);
        match layer {
            Layer::Cache => { self.layer0_cache.fetch_add(tokens, Ordering::Relaxed); }
            Layer::Index => { self.layer1_index.fetch_add(tokens, Ordering::Relaxed); }
            Layer::Scoped => { self.layer2_scoped.fetch_add(tokens, Ordering::Relaxed); }
            Layer::Delta => { self.layer3_delta.fetch_add(tokens, Ordering::Relaxed); }
            Layer::Full => { self.layer4_full.fetch_add(tokens, Ordering::Relaxed); }
        }
        let saved = potential.saturating_sub(tokens);
        match layer {
            Layer::Cache => { self.cache_savings.fetch_add(saved, Ordering::Relaxed); }
            Layer::Scoped => { self.scope_savings.fetch_add(saved, Ordering::Relaxed); }
            Layer::Delta => { self.delta_savings.fetch_add(saved, Ordering::Relaxed); }
            _ => {}
        }
    }

    pub fn total_tokens(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    pub fn total_savings(&self) -> u64 {
        self.cache_savings.load(Ordering::Relaxed)
            + self.scope_savings.load(Ordering::Relaxed)
            + self.delta_savings.load(Ordering::Relaxed)
    }

    pub fn conservation_score(&self) -> f64 {
        let total = self.total_tokens();
        let saved = self.total_savings();
        let potential = total + saved;
        if potential == 0 { return 1.0; }
        saved as f64 / potential as f64
    }

    pub fn reset(&self) {
        self.total.store(0, Ordering::Relaxed);
        self.layer0_cache.store(0, Ordering::Relaxed);
        self.layer1_index.store(0, Ordering::Relaxed);
        self.layer2_scoped.store(0, Ordering::Relaxed);
        self.layer3_delta.store(0, Ordering::Relaxed);
        self.layer4_full.store(0, Ordering::Relaxed);
        self.cache_savings.store(0, Ordering::Relaxed);
        self.scope_savings.store(0, Ordering::Relaxed);
        self.delta_savings.store(0, Ordering::Relaxed);
    }
}

impl Default for TokenMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    pub layer: Layer,
    pub tokens_used: u64,
    pub tokens_saved: u64,
    pub cache_hit: bool,
    pub response_size: usize,
}

impl ResponseMetrics {
    pub fn from_cache(full_cost: u64) -> Self {
        Self { layer: Layer::Cache, tokens_used: 0, tokens_saved: full_cost, cache_hit: true, response_size: 0 }
    }

    pub fn from_query(layer: Layer, tokens: u64, full_cost: u64) -> Self {
        Self { layer, tokens_used: tokens, tokens_saved: full_cost.saturating_sub(tokens), cache_hit: false, response_size: 0 }
    }

    pub fn full(tokens: u64) -> Self {
        Self { layer: Layer::Full, tokens_used: tokens, tokens_saved: 0, cache_hit: false, response_size: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_metrics_new() {
        let m = TokenMetrics::new();
        assert_eq!(m.total_tokens(), 0);
        assert_eq!(m.conservation_score(), 1.0);
    }

    #[test]
    fn test_token_metrics_record() {
        let m = TokenMetrics::new();
        m.record(Layer::Full, 500, 500);
        assert_eq!(m.total_tokens(), 500);

        m.record(Layer::Cache, 0, 500);
        assert_eq!(m.total_tokens(), 500);
        assert_eq!(m.total_savings(), 500);
    }

    #[test]
    fn test_conservation_score() {
        let m = TokenMetrics::new();
        // Full query: 500 tokens, no savings
        m.record(Layer::Full, 500, 500);
        assert_eq!(m.conservation_score(), 0.0);

        // Cache hit: 0 tokens, saved 500
        m.record(Layer::Cache, 0, 500);
        // total=500, savings=500, potential=1000
        assert!((m.conservation_score() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_conservation_score_improves() {
        let m = TokenMetrics::new();

        // Cold: 10 full queries
        for _ in 0..10 {
            m.record(Layer::Full, 500, 500);
        }
        let cold = m.conservation_score();

        // Warm: 10 cache hits
        for _ in 0..10 {
            m.record(Layer::Cache, 0, 500);
        }
        let warm = m.conservation_score();

        assert!(warm > cold, "Conservation should improve: cold={} warm={}", cold, warm);
    }

    #[test]
    fn test_scoped_savings() {
        let m = TokenMetrics::new();
        m.record(Layer::Scoped, 50, 500);
        assert_eq!(m.total_tokens(), 50);
        assert_eq!(m.total_savings(), 450);
    }

    #[test]
    fn test_delta_savings() {
        let m = TokenMetrics::new();
        m.record(Layer::Delta, 10, 500);
        assert_eq!(m.total_savings(), 490);
    }

    #[test]
    fn test_metrics_reset() {
        let m = TokenMetrics::new();
        m.record(Layer::Full, 1000, 1000);
        m.reset();
        assert_eq!(m.total_tokens(), 0);
        assert_eq!(m.total_savings(), 0);
    }

    #[test]
    fn test_response_metrics_cache() {
        let rm = ResponseMetrics::from_cache(500);
        assert!(rm.cache_hit);
        assert_eq!(rm.tokens_used, 0);
        assert_eq!(rm.tokens_saved, 500);
    }

    #[test]
    fn test_response_metrics_query() {
        let rm = ResponseMetrics::from_query(Layer::Scoped, 50, 500);
        assert!(!rm.cache_hit);
        assert_eq!(rm.tokens_used, 50);
        assert_eq!(rm.tokens_saved, 450);
    }

    #[test]
    fn test_layer_numbers() {
        assert_eq!(Layer::Cache.number(), 0);
        assert_eq!(Layer::Index.number(), 1);
        assert_eq!(Layer::Scoped.number(), 2);
        assert_eq!(Layer::Delta.number(), 3);
        assert_eq!(Layer::Full.number(), 4);
    }
}

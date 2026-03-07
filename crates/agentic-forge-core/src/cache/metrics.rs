//! Cache metrics — hit rate, size, evictions.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct CacheMetrics {
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    eviction_count: AtomicU64,
    current_size: AtomicUsize,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            current_size: AtomicUsize::new(0),
        }
    }

    pub fn record_hit(&self) {
        self.hit_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.miss_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.eviction_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_size(&self, size: usize) {
        self.current_size.store(size, Ordering::Relaxed);
    }

    pub fn hits(&self) -> u64 {
        self.hit_count.load(Ordering::Relaxed)
    }

    pub fn misses(&self) -> u64 {
        self.miss_count.load(Ordering::Relaxed)
    }

    pub fn evictions(&self) -> u64 {
        self.eviction_count.load(Ordering::Relaxed)
    }

    pub fn size(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }

    pub fn total_requests(&self) -> u64 {
        self.hits() + self.misses()
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 { return 0.0; }
        self.hits() as f64 / total as f64
    }

    pub fn reset(&self) {
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
        self.eviction_count.store(0, Ordering::Relaxed);
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initial() {
        let m = CacheMetrics::new();
        assert_eq!(m.hits(), 0);
        assert_eq!(m.misses(), 0);
        assert_eq!(m.hit_rate(), 0.0);
    }

    #[test]
    fn test_metrics_hit_rate() {
        let m = CacheMetrics::new();
        m.record_hit();
        m.record_hit();
        m.record_miss();
        assert!((m.hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_metrics_reset() {
        let m = CacheMetrics::new();
        m.record_hit();
        m.record_miss();
        m.reset();
        assert_eq!(m.total_requests(), 0);
    }

    #[test]
    fn test_metrics_evictions() {
        let m = CacheMetrics::new();
        m.record_eviction();
        m.record_eviction();
        assert_eq!(m.evictions(), 2);
    }
}

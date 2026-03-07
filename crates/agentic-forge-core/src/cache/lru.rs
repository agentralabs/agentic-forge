//! Generic LRU cache with TTL.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use super::metrics::CacheMetrics;

struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

pub struct Cache<K, V> {
    entries: RwLock<HashMap<K, CacheEntry<V>>>,
    max_size: usize,
    ttl: Duration,
    pub metrics: CacheMetrics,
}

impl<K: Hash + Eq + Clone, V: Clone> Cache<K, V> {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_size,
            ttl,
            metrics: CacheMetrics::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(key) {
            if entry.inserted_at.elapsed() < self.ttl {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                self.metrics.record_hit();
                return Some(entry.value.clone());
            } else {
                entries.remove(key);
                self.metrics.record_eviction();
            }
        }
        self.metrics.record_miss();
        None
    }

    pub fn insert(&self, key: K, value: V) {
        let mut entries = self.entries.write().unwrap();
        if entries.len() >= self.max_size {
            self.evict_lru(&mut entries);
        }
        entries.insert(key, CacheEntry {
            value,
            inserted_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        });
        self.metrics.set_size(entries.len());
    }

    pub fn invalidate(&self, key: &K) -> bool {
        let mut entries = self.entries.write().unwrap();
        let removed = entries.remove(key).is_some();
        self.metrics.set_size(entries.len());
        removed
    }

    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
        self.metrics.set_size(0);
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    pub fn contains(&self, key: &K) -> bool {
        let entries = self.entries.read().unwrap();
        if let Some(entry) = entries.get(key) {
            entry.inserted_at.elapsed() < self.ttl
        } else {
            false
        }
    }

    fn evict_lru(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if let Some(lru_key) = entries.iter()
            .min_by_key(|(_, e)| e.last_accessed)
            .map(|(k, _)| k.clone())
        {
            entries.remove(&lru_key);
            self.metrics.record_eviction();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache: Cache<String, String> = Cache::new(10, Duration::from_secs(60));
        cache.insert("key".into(), "value".into());
        assert_eq!(cache.get(&"key".into()), Some("value".into()));
    }

    #[test]
    fn test_cache_miss() {
        let cache: Cache<String, String> = Cache::new(10, Duration::from_secs(60));
        assert_eq!(cache.get(&"missing".into()), None);
    }

    #[test]
    fn test_cache_ttl_expiry() {
        let cache: Cache<String, String> = Cache::new(10, Duration::from_millis(1));
        cache.insert("key".into(), "value".into());
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(cache.get(&"key".into()), None);
    }

    #[test]
    fn test_cache_eviction_on_full() {
        let cache: Cache<u32, u32> = Cache::new(3, Duration::from_secs(60));
        cache.insert(1, 10);
        cache.insert(2, 20);
        cache.insert(3, 30);
        cache.insert(4, 40); // Should evict LRU
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache: Cache<String, String> = Cache::new(10, Duration::from_secs(60));
        cache.insert("key".into(), "value".into());
        assert!(cache.invalidate(&"key".into()));
        assert_eq!(cache.get(&"key".into()), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache: Cache<u32, u32> = Cache::new(10, Duration::from_secs(60));
        for i in 0..5 { cache.insert(i, i * 10); }
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_contains() {
        let cache: Cache<String, String> = Cache::new(10, Duration::from_secs(60));
        cache.insert("k".into(), "v".into());
        assert!(cache.contains(&"k".into()));
        assert!(!cache.contains(&"missing".into()));
    }

    #[test]
    fn test_cache_metrics_hit_miss() {
        let cache: Cache<String, String> = Cache::new(10, Duration::from_secs(60));
        cache.insert("key".into(), "value".into());
        let _ = cache.get(&"key".into());    // hit
        let _ = cache.get(&"miss".into());   // miss
        let _ = cache.get(&"key".into());    // hit
        assert_eq!(cache.metrics.hits(), 2);
        assert_eq!(cache.metrics.misses(), 1);
        assert!(cache.metrics.hit_rate() > 0.6);
    }

    #[test]
    fn test_cache_overwrite_same_key() {
        let cache: Cache<String, String> = Cache::new(10, Duration::from_secs(60));
        cache.insert("k".into(), "v1".into());
        cache.insert("k".into(), "v2".into());
        assert_eq!(cache.get(&"k".into()), Some("v2".into()));
    }

    #[test]
    fn test_cache_second_query_is_cache_hit() {
        let cache: Cache<String, Vec<u8>> = Cache::new(100, Duration::from_secs(60));
        let big_data = vec![0u8; 10_000];
        cache.insert("query".into(), big_data.clone());

        // First get = hit
        let r1 = cache.get(&"query".into());
        assert!(r1.is_some());
        assert_eq!(cache.metrics.hits(), 1);

        // Second get = still hit
        let r2 = cache.get(&"query".into());
        assert!(r2.is_some());
        assert_eq!(cache.metrics.hits(), 2);
        assert_eq!(cache.metrics.misses(), 0);
    }
}

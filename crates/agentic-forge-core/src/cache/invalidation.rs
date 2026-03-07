//! Cache invalidation on mutation.

use super::lru::Cache;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::RwLock;

pub struct CacheInvalidator<K: Hash + Eq + Clone> {
    dependencies: RwLock<std::collections::HashMap<K, HashSet<K>>>,
}

impl<K: Hash + Eq + Clone> CacheInvalidator<K> {
    pub fn new() -> Self {
        Self {
            dependencies: RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn register_dependency(&self, key: K, depends_on: K) {
        let mut deps = self.dependencies.write().unwrap();
        deps.entry(depends_on).or_default().insert(key);
    }

    pub fn invalidate_cascade<V: Clone>(&self, changed_key: &K, cache: &Cache<K, V>) -> usize {
        let mut invalidated = 0;
        cache.invalidate(changed_key);
        invalidated += 1;

        let deps = self.dependencies.read().unwrap();
        if let Some(dependents) = deps.get(changed_key) {
            for dependent in dependents {
                if cache.invalidate(dependent) {
                    invalidated += 1;
                }
            }
        }
        invalidated
    }

    pub fn clear(&self) {
        self.dependencies.write().unwrap().clear();
    }
}

impl<K: Hash + Eq + Clone> Default for CacheInvalidator<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_invalidate_cascade() {
        let cache: Cache<String, String> = Cache::new(100, Duration::from_secs(60));
        let inv: CacheInvalidator<String> = CacheInvalidator::new();

        cache.insert("parent".into(), "p_val".into());
        cache.insert("child".into(), "c_val".into());
        inv.register_dependency("child".into(), "parent".into());

        let count = inv.invalidate_cascade(&"parent".into(), &cache);
        assert_eq!(count, 2);
        assert!(cache.get(&"parent".into()).is_none());
        assert!(cache.get(&"child".into()).is_none());
    }

    #[test]
    fn test_invalidate_no_dependents() {
        let cache: Cache<String, String> = Cache::new(100, Duration::from_secs(60));
        let inv: CacheInvalidator<String> = CacheInvalidator::new();

        cache.insert("solo".into(), "val".into());
        let count = inv.invalidate_cascade(&"solo".into(), &cache);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_invalidator_clear() {
        let inv: CacheInvalidator<String> = CacheInvalidator::new();
        inv.register_dependency("a".into(), "b".into());
        inv.clear();
        // After clear, no cascading should happen
        let cache: Cache<String, String> = Cache::new(10, Duration::from_secs(60));
        cache.insert("b".into(), "val".into());
        cache.insert("a".into(), "val".into());
        let count = inv.invalidate_cascade(&"b".into(), &cache);
        assert_eq!(count, 1); // Only "b", not "a"
    }
}

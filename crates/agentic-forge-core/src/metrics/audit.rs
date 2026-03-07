//! Audit log generation for every MCP call.

use super::tokens::Layer;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: i64,
    pub tool: String,
    pub layer: Layer,
    pub tokens_used: u64,
    pub tokens_saved: u64,
    pub cache_hit: bool,
    pub intent: String,
    pub source_size: u64,
    pub result_size: u64,
}

impl AuditEntry {
    pub fn waste_ratio(&self) -> f64 {
        if self.source_size == 0 {
            return 0.0;
        }
        self.result_size as f64 / self.source_size as f64
    }
}

pub struct AuditLog {
    entries: Mutex<Vec<AuditEntry>>,
    max_entries: usize,
}

impl AuditLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            max_entries,
        }
    }

    pub fn record(&self, entry: AuditEntry) {
        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= self.max_entries {
            entries.remove(0);
        }
        entries.push(entry);
    }

    pub fn entries(&self) -> Vec<AuditEntry> {
        self.entries.lock().unwrap().clone()
    }

    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    pub fn average_waste_ratio(&self) -> f64 {
        let entries = self.entries.lock().unwrap();
        if entries.is_empty() {
            return 0.0;
        }
        let total: f64 = entries.iter().map(|e| e.waste_ratio()).sum();
        total / entries.len() as f64
    }

    pub fn total_tokens_used(&self) -> u64 {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .map(|e| e.tokens_used)
            .sum()
    }

    pub fn total_tokens_saved(&self) -> u64 {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .map(|e| e.tokens_saved)
            .sum()
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let entries = self.entries.lock().unwrap();
        if entries.is_empty() {
            return 0.0;
        }
        let hits = entries.iter().filter(|e| e.cache_hit).count();
        hits as f64 / entries.len() as f64
    }

    pub fn layer_distribution(&self) -> std::collections::HashMap<u8, usize> {
        let entries = self.entries.lock().unwrap();
        let mut dist = std::collections::HashMap::new();
        for entry in entries.iter() {
            *dist.entry(entry.layer.number()).or_insert(0) += 1;
        }
        dist
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(
        tool: &str,
        layer: Layer,
        tokens: u64,
        saved: u64,
        cache_hit: bool,
    ) -> AuditEntry {
        AuditEntry {
            timestamp: chrono::Utc::now().timestamp_micros(),
            tool: tool.into(),
            layer,
            tokens_used: tokens,
            tokens_saved: saved,
            cache_hit,
            intent: "test".into(),
            source_size: 1000,
            result_size: tokens,
        }
    }

    #[test]
    fn test_audit_log_record() {
        let log = AuditLog::new(100);
        log.record(make_entry("test_tool", Layer::Full, 500, 0, false));
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_audit_log_max_entries() {
        let log = AuditLog::new(3);
        for i in 0..5 {
            let tool = format!("tool_{}", i);
            log.record(make_entry(&tool, Layer::Full, 100, 0, false));
        }
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn test_audit_log_cache_hit_rate() {
        let log = AuditLog::new(100);
        log.record(make_entry("a", Layer::Cache, 0, 500, true));
        log.record(make_entry("b", Layer::Full, 500, 0, false));
        log.record(make_entry("c", Layer::Cache, 0, 500, true));
        assert!((log.cache_hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_audit_log_total_tokens() {
        let log = AuditLog::new(100);
        log.record(make_entry("a", Layer::Full, 100, 0, false));
        log.record(make_entry("b", Layer::Scoped, 50, 450, false));
        assert_eq!(log.total_tokens_used(), 150);
        assert_eq!(log.total_tokens_saved(), 450);
    }

    #[test]
    fn test_audit_log_layer_distribution() {
        let log = AuditLog::new(100);
        log.record(make_entry("a", Layer::Cache, 0, 500, true));
        log.record(make_entry("b", Layer::Cache, 0, 500, true));
        log.record(make_entry("c", Layer::Full, 500, 0, false));
        let dist = log.layer_distribution();
        assert_eq!(dist.get(&0), Some(&2)); // Layer::Cache = 0
        assert_eq!(dist.get(&4), Some(&1)); // Layer::Full = 4
    }

    #[test]
    fn test_audit_log_clear() {
        let log = AuditLog::new(100);
        log.record(make_entry("a", Layer::Full, 100, 0, false));
        log.clear();
        assert!(log.is_empty());
    }

    #[test]
    fn test_waste_ratio() {
        let entry = AuditEntry {
            timestamp: 0,
            tool: "test".into(),
            layer: Layer::Full,
            tokens_used: 500,
            tokens_saved: 0,
            cache_hit: false,
            intent: "full".into(),
            source_size: 50000,
            result_size: 150,
        };
        assert!(entry.waste_ratio() < 0.01);
    }
}

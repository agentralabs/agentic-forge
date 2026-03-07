//! Delta retrieval — change-proportional queries.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change<T> {
    pub id: String,
    pub change_type: ChangeType,
    pub timestamp: i64,
    pub value: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaResult<T> {
    pub changes: Vec<Change<T>>,
    pub from_version: u64,
    pub to_version: u64,
    pub has_more: bool,
}

impl<T> DeltaResult<T> {
    pub fn empty(version: u64) -> Self {
        Self { changes: Vec::new(), from_version: version, to_version: version, has_more: false }
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.changes.len()
    }
}

#[derive(Debug, Clone)]
pub struct VersionedState {
    version: u64,
    timestamps: Vec<(String, i64, ChangeType)>,
}

impl VersionedState {
    pub fn new() -> Self {
        Self { version: 0, timestamps: Vec::new() }
    }

    pub fn record_change(&mut self, id: impl Into<String>, change_type: ChangeType) {
        self.version += 1;
        let ts = chrono::Utc::now().timestamp_micros();
        self.timestamps.push((id.into(), ts, change_type));
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn changes_since(&self, since_ts: i64) -> Vec<&(String, i64, ChangeType)> {
        self.timestamps.iter().filter(|(_, ts, _)| *ts > since_ts).collect()
    }

    pub fn changes_since_version(&self, since_version: u64) -> Vec<&(String, i64, ChangeType)> {
        if since_version as usize >= self.timestamps.len() {
            return Vec::new();
        }
        self.timestamps[since_version as usize..].iter().collect()
    }

    pub fn last_change_timestamp(&self) -> i64 {
        self.timestamps.last().map(|(_, ts, _)| *ts).unwrap_or(0)
    }

    pub fn is_unchanged_since(&self, since_version: u64) -> bool {
        self.version <= since_version
    }
}

impl Default for VersionedState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_state_new() {
        let state = VersionedState::new();
        assert_eq!(state.version(), 0);
    }

    #[test]
    fn test_versioned_state_record() {
        let mut state = VersionedState::new();
        state.record_change("entity_1", ChangeType::Created);
        assert_eq!(state.version(), 1);
        state.record_change("entity_2", ChangeType::Created);
        assert_eq!(state.version(), 2);
    }

    #[test]
    fn test_versioned_state_changes_since_version() {
        let mut state = VersionedState::new();
        state.record_change("a", ChangeType::Created);
        state.record_change("b", ChangeType::Created);
        state.record_change("c", ChangeType::Updated);

        let changes = state.changes_since_version(1);
        assert_eq!(changes.len(), 2); // b and c
    }

    #[test]
    fn test_versioned_state_unchanged() {
        let mut state = VersionedState::new();
        state.record_change("a", ChangeType::Created);
        assert!(state.is_unchanged_since(1));
        assert!(!state.is_unchanged_since(0));
    }

    #[test]
    fn test_delta_result_empty() {
        let result: DeltaResult<String> = DeltaResult::empty(5);
        assert!(result.is_empty());
        assert_eq!(result.from_version, 5);
    }

    #[test]
    fn test_delta_proportional_cost() {
        let mut state = VersionedState::new();
        for i in 0..100 {
            state.record_change(format!("item_{}", i), ChangeType::Created);
        }
        let v_before = state.version();

        // Add one more
        state.record_change("item_100", ChangeType::Created);

        let all_changes = state.changes_since_version(0);
        let delta_changes = state.changes_since_version(v_before);

        assert_eq!(all_changes.len(), 101);
        assert_eq!(delta_changes.len(), 1);
        // Delta is 101x cheaper than full scan
        assert!(delta_changes.len() < all_changes.len() / 50);
    }

    #[test]
    fn test_unchanged_state_free() {
        let mut state = VersionedState::new();
        state.record_change("a", ChangeType::Created);
        let v = state.version();
        // No changes since v
        let changes = state.changes_since_version(v);
        assert!(changes.is_empty());
        assert!(state.is_unchanged_since(v));
    }
}

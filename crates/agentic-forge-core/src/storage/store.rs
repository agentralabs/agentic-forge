//! Blueprint store — save/load/delete/list blueprints.

use crate::types::blueprint::{Blueprint, BlueprintStatus};
use crate::types::ids::BlueprintId;
use crate::types::{ForgeError, ForgeResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BlueprintStore {
    blueprints: HashMap<BlueprintId, Blueprint>,
    storage_path: Option<PathBuf>,
    dirty: bool,
}

impl BlueprintStore {
    pub fn new() -> Self {
        Self {
            blueprints: HashMap::new(),
            storage_path: None,
            dirty: false,
        }
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            blueprints: HashMap::new(),
            storage_path: Some(path.into()),
            dirty: false,
        }
    }

    pub fn save(&mut self, blueprint: Blueprint) -> ForgeResult<BlueprintId> {
        let id = blueprint.id;
        self.blueprints.insert(id, blueprint);
        self.dirty = true;
        Ok(id)
    }

    pub fn load(&self, id: &BlueprintId) -> ForgeResult<&Blueprint> {
        self.blueprints
            .get(id)
            .ok_or_else(|| ForgeError::BlueprintNotFound(id.to_string()))
    }

    pub fn load_mut(&mut self, id: &BlueprintId) -> ForgeResult<&mut Blueprint> {
        self.blueprints
            .get_mut(id)
            .ok_or_else(|| ForgeError::BlueprintNotFound(id.to_string()))
    }

    pub fn delete(&mut self, id: &BlueprintId) -> ForgeResult<Blueprint> {
        self.blueprints
            .remove(id)
            .ok_or_else(|| ForgeError::BlueprintNotFound(id.to_string()))
            .inspect(|_| self.dirty = true)
    }

    pub fn list(&self) -> Vec<&Blueprint> {
        self.blueprints.values().collect()
    }

    pub fn list_by_status(&self, status: BlueprintStatus) -> Vec<&Blueprint> {
        self.blueprints
            .values()
            .filter(|bp| bp.status == status)
            .collect()
    }

    pub fn list_by_name(&self, name: &str) -> Vec<&Blueprint> {
        self.blueprints
            .values()
            .filter(|bp| bp.name.contains(name))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.blueprints.len()
    }

    pub fn contains(&self, id: &BlueprintId) -> bool {
        self.blueprints.contains_key(id)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn clear(&mut self) {
        self.blueprints.clear();
        self.dirty = true;
    }

    pub fn persist(&mut self) -> ForgeResult<()> {
        if let Some(path) = &self.storage_path {
            let data = serde_json::to_vec(&self.blueprints)?;
            std::fs::write(path, data)?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn load_from_disk(path: impl AsRef<Path>) -> ForgeResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::with_path(path));
        }
        let data = std::fs::read(path)?;
        let blueprints: HashMap<BlueprintId, Blueprint> = serde_json::from_slice(&data)?;
        Ok(Self {
            blueprints,
            storage_path: Some(path.to_path_buf()),
            dirty: false,
        })
    }

    pub fn storage_path(&self) -> Option<&Path> {
        self.storage_path.as_deref()
    }

    pub fn ids(&self) -> Vec<BlueprintId> {
        self.blueprints.keys().copied().collect()
    }
}

impl Default for BlueprintStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::intent::Domain;

    fn make_blueprint(name: &str) -> Blueprint {
        Blueprint::new(name, "test blueprint", Domain::Api)
    }

    #[test]
    fn test_store_save_and_load() {
        let mut store = BlueprintStore::new();
        let bp = make_blueprint("Test");
        let id = bp.id;
        store.save(bp).unwrap();
        let loaded = store.load(&id).unwrap();
        assert_eq!(loaded.name, "Test");
    }

    #[test]
    fn test_store_delete() {
        let mut store = BlueprintStore::new();
        let bp = make_blueprint("Delete Me");
        let id = bp.id;
        store.save(bp).unwrap();
        assert_eq!(store.count(), 1);
        store.delete(&id).unwrap();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_store_delete_not_found() {
        let mut store = BlueprintStore::new();
        let id = BlueprintId::new();
        assert!(store.delete(&id).is_err());
    }

    #[test]
    fn test_store_list() {
        let mut store = BlueprintStore::new();
        store.save(make_blueprint("A")).unwrap();
        store.save(make_blueprint("B")).unwrap();
        store.save(make_blueprint("C")).unwrap();
        assert_eq!(store.list().len(), 3);
    }

    #[test]
    fn test_store_list_by_status() {
        let mut store = BlueprintStore::new();
        let mut bp1 = make_blueprint("Draft");
        bp1.status = BlueprintStatus::Draft;
        let mut bp2 = make_blueprint("Complete");
        bp2.status = BlueprintStatus::Complete;
        store.save(bp1).unwrap();
        store.save(bp2).unwrap();
        assert_eq!(store.list_by_status(BlueprintStatus::Draft).len(), 1);
        assert_eq!(store.list_by_status(BlueprintStatus::Complete).len(), 1);
    }

    #[test]
    fn test_store_list_by_name() {
        let mut store = BlueprintStore::new();
        store.save(make_blueprint("WebApp")).unwrap();
        store.save(make_blueprint("WebApi")).unwrap();
        store.save(make_blueprint("CLI Tool")).unwrap();
        assert_eq!(store.list_by_name("Web").len(), 2);
    }

    #[test]
    fn test_store_contains() {
        let mut store = BlueprintStore::new();
        let bp = make_blueprint("Test");
        let id = bp.id;
        store.save(bp).unwrap();
        assert!(store.contains(&id));
        assert!(!store.contains(&BlueprintId::new()));
    }

    #[test]
    fn test_store_dirty_tracking() {
        let mut store = BlueprintStore::new();
        assert!(!store.is_dirty());
        store.save(make_blueprint("Test")).unwrap();
        assert!(store.is_dirty());
        store.mark_clean();
        assert!(!store.is_dirty());
    }

    #[test]
    fn test_store_clear() {
        let mut store = BlueprintStore::new();
        store.save(make_blueprint("A")).unwrap();
        store.save(make_blueprint("B")).unwrap();
        store.clear();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_store_load_mut() {
        let mut store = BlueprintStore::new();
        let bp = make_blueprint("Mutable");
        let id = bp.id;
        store.save(bp).unwrap();
        let bp_mut = store.load_mut(&id).unwrap();
        bp_mut.name = "Changed".into();
        assert_eq!(store.load(&id).unwrap().name, "Changed");
    }

    #[test]
    fn test_store_persist_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_store.json");

        let mut store = BlueprintStore::with_path(&path);
        store.save(make_blueprint("Persisted")).unwrap();
        store.persist().unwrap();

        let loaded = BlueprintStore::load_from_disk(&path).unwrap();
        assert_eq!(loaded.count(), 1);
        let bp = loaded.list()[0];
        assert_eq!(bp.name, "Persisted");
    }

    #[test]
    fn test_store_load_nonexistent() {
        let store = BlueprintStore::load_from_disk("/tmp/nonexistent_forge_test.json").unwrap();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_store_ids() {
        let mut store = BlueprintStore::new();
        let bp1 = make_blueprint("A");
        let bp2 = make_blueprint("B");
        let id1 = bp1.id;
        let id2 = bp2.id;
        store.save(bp1).unwrap();
        store.save(bp2).unwrap();
        let ids = store.ids();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }
}

//! Indexes for fast blueprint lookups.

use crate::types::ids::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct ForgeIndexes {
    pub blueprint_index: HashSet<BlueprintId>,
    pub entity_index: HashMap<EntityId, BlueprintId>,
    pub file_index: HashMap<FileId, BlueprintId>,
    pub dependency_index: HashMap<DependencyId, BlueprintId>,
    pub name_index: HashMap<String, BlueprintId>,
}

impl ForgeIndexes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_blueprint(&mut self, id: BlueprintId) {
        self.blueprint_index.insert(id);
    }

    pub fn remove_blueprint(&mut self, id: &BlueprintId) {
        self.blueprint_index.remove(id);
        self.entity_index.retain(|_, bp_id| bp_id != id);
        self.file_index.retain(|_, bp_id| bp_id != id);
        self.dependency_index.retain(|_, bp_id| bp_id != id);
        self.name_index.retain(|_, bp_id| bp_id != id);
    }

    pub fn add_entity(&mut self, entity_id: EntityId, bp_id: BlueprintId) {
        self.entity_index.insert(entity_id, bp_id);
    }

    pub fn add_file(&mut self, file_id: FileId, bp_id: BlueprintId) {
        self.file_index.insert(file_id, bp_id);
    }

    pub fn add_dependency(&mut self, dep_id: DependencyId, bp_id: BlueprintId) {
        self.dependency_index.insert(dep_id, bp_id);
    }

    pub fn add_name(&mut self, name: String, bp_id: BlueprintId) {
        self.name_index.insert(name, bp_id);
    }

    pub fn lookup_entity_blueprint(&self, entity_id: &EntityId) -> Option<&BlueprintId> {
        self.entity_index.get(entity_id)
    }

    pub fn lookup_file_blueprint(&self, file_id: &FileId) -> Option<&BlueprintId> {
        self.file_index.get(file_id)
    }

    pub fn lookup_by_name(&self, name: &str) -> Option<&BlueprintId> {
        self.name_index.get(name)
    }

    pub fn clear(&mut self) {
        self.blueprint_index.clear();
        self.entity_index.clear();
        self.file_index.clear();
        self.dependency_index.clear();
        self.name_index.clear();
    }

    pub fn blueprint_count(&self) -> usize {
        self.blueprint_index.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_add_remove() {
        let mut idx = ForgeIndexes::new();
        let bp_id = BlueprintId::new();
        idx.add_blueprint(bp_id);
        assert_eq!(idx.blueprint_count(), 1);
        idx.remove_blueprint(&bp_id);
        assert_eq!(idx.blueprint_count(), 0);
    }

    #[test]
    fn test_entity_index() {
        let mut idx = ForgeIndexes::new();
        let bp_id = BlueprintId::new();
        let eid = EntityId::new();
        idx.add_entity(eid, bp_id);
        assert_eq!(idx.lookup_entity_blueprint(&eid), Some(&bp_id));
    }

    #[test]
    fn test_file_index() {
        let mut idx = ForgeIndexes::new();
        let bp_id = BlueprintId::new();
        let fid = FileId::new();
        idx.add_file(fid, bp_id);
        assert_eq!(idx.lookup_file_blueprint(&fid), Some(&bp_id));
    }

    #[test]
    fn test_name_index() {
        let mut idx = ForgeIndexes::new();
        let bp_id = BlueprintId::new();
        idx.add_name("MyProject".into(), bp_id);
        assert_eq!(idx.lookup_by_name("MyProject"), Some(&bp_id));
    }

    #[test]
    fn test_clear() {
        let mut idx = ForgeIndexes::new();
        idx.add_blueprint(BlueprintId::new());
        idx.add_blueprint(BlueprintId::new());
        idx.clear();
        assert_eq!(idx.blueprint_count(), 0);
    }

    #[test]
    fn test_remove_cascades() {
        let mut idx = ForgeIndexes::new();
        let bp_id = BlueprintId::new();
        let eid = EntityId::new();
        idx.add_blueprint(bp_id);
        idx.add_entity(eid, bp_id);
        idx.remove_blueprint(&bp_id);
        assert!(idx.lookup_entity_blueprint(&eid).is_none());
    }
}

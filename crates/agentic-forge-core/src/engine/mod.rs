//! Engine layer — ForgeEngine, WriteEngine, QueryEngine.

pub mod query;
pub mod validator;
pub mod write;

pub use query::QueryEngine;
pub use write::WriteEngine;

use crate::index::ForgeIndexes;
use crate::storage::BlueprintStore;
use crate::types::blueprint::Blueprint;
use crate::types::ids::BlueprintId;
use crate::types::intent::Domain;
use crate::types::ForgeResult;

#[derive(Debug)]
pub struct ForgeEngine {
    pub store: BlueprintStore,
    pub indexes: ForgeIndexes,
    dirty: bool,
}

impl ForgeEngine {
    pub fn new() -> Self {
        Self {
            store: BlueprintStore::new(),
            indexes: ForgeIndexes::new(),
            dirty: false,
        }
    }

    pub fn with_store(store: BlueprintStore) -> Self {
        Self {
            store,
            indexes: ForgeIndexes::new(),
            dirty: false,
        }
    }

    pub fn writer(&mut self) -> WriteEngine<'_> {
        WriteEngine::new(self)
    }

    pub fn reader(&self) -> QueryEngine<'_> {
        QueryEngine::new(self)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty || self.store.is_dirty()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.store.mark_clean();
    }

    pub fn create_blueprint(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        domain: Domain,
    ) -> ForgeResult<BlueprintId> {
        let bp = Blueprint::new(name, description, domain);
        let id = bp.id;
        self.store.save(bp)?;
        self.indexes.add_blueprint(id);
        self.dirty = true;
        Ok(id)
    }

    pub fn blueprint_count(&self) -> usize {
        self.store.count()
    }
}

impl Default for ForgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = ForgeEngine::new();
        assert_eq!(engine.blueprint_count(), 0);
        assert!(!engine.is_dirty());
    }

    #[test]
    fn test_engine_create_blueprint() {
        let mut engine = ForgeEngine::new();
        let id = engine
            .create_blueprint("Test", "A test", Domain::Api)
            .unwrap();
        assert_eq!(engine.blueprint_count(), 1);
        assert!(engine.is_dirty());
        let bp = engine.store.load(&id).unwrap();
        assert_eq!(bp.name, "Test");
    }

    #[test]
    fn test_engine_writer_reader() {
        let mut engine = ForgeEngine::new();
        let id = engine
            .create_blueprint("Test", "A test", Domain::Web)
            .unwrap();
        {
            let reader = engine.reader();
            let bp = reader.get_blueprint(&id).unwrap();
            assert_eq!(bp.name, "Test");
        }
        {
            let mut writer = engine.writer();
            writer.rename_blueprint(&id, "Renamed").unwrap();
        }
        let reader = engine.reader();
        assert_eq!(reader.get_blueprint(&id).unwrap().name, "Renamed");
    }

    #[test]
    fn test_engine_dirty_tracking() {
        let mut engine = ForgeEngine::new();
        assert!(!engine.is_dirty());
        engine
            .create_blueprint("Test", "Test", Domain::Cli)
            .unwrap();
        assert!(engine.is_dirty());
        engine.mark_clean();
        assert!(!engine.is_dirty());
    }
}

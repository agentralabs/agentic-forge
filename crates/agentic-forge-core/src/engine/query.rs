//! QueryEngine — read-only operations for blueprints.

use crate::engine::ForgeEngine;
use crate::types::blueprint::*;
use crate::types::ids::*;
use crate::types::{ForgeError, ForgeResult};

pub struct QueryEngine<'a> {
    engine: &'a ForgeEngine,
}

impl<'a> QueryEngine<'a> {
    pub fn new(engine: &'a ForgeEngine) -> Self {
        Self { engine }
    }

    // Blueprint queries

    pub fn get_blueprint(&self, id: &BlueprintId) -> ForgeResult<&Blueprint> {
        self.engine.store.load(id)
    }

    pub fn list_blueprints(&self) -> Vec<&Blueprint> {
        self.engine.store.list()
    }

    pub fn list_by_status(&self, status: BlueprintStatus) -> Vec<&Blueprint> {
        self.engine.store.list_by_status(status)
    }

    pub fn search_blueprints(&self, query: &str) -> Vec<&Blueprint> {
        self.engine.store.list().into_iter().filter(|bp| {
            bp.name.to_lowercase().contains(&query.to_lowercase())
                || bp.description.to_lowercase().contains(&query.to_lowercase())
        }).collect()
    }

    pub fn blueprint_count(&self) -> usize {
        self.engine.store.count()
    }

    pub fn blueprint_exists(&self, id: &BlueprintId) -> bool {
        self.engine.store.contains(id)
    }

    // Entity queries

    pub fn get_entity(&self, bp_id: &BlueprintId, entity_id: &EntityId) -> ForgeResult<&Entity> {
        let bp = self.engine.store.load(bp_id)?;
        bp.entities.iter().find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))
    }

    pub fn get_entity_by_name(&self, bp_id: &BlueprintId, name: &str) -> ForgeResult<&Entity> {
        let bp = self.engine.store.load(bp_id)?;
        bp.find_entity(name)
            .ok_or_else(|| ForgeError::EntityNotFound(name.to_string()))
    }

    pub fn list_entities(&self, bp_id: &BlueprintId) -> ForgeResult<&[Entity]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.entities)
    }

    pub fn entity_count(&self, bp_id: &BlueprintId) -> ForgeResult<usize> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.entity_count())
    }

    pub fn search_entities(&self, bp_id: &BlueprintId, query: &str) -> ForgeResult<Vec<&Entity>> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.entities.iter().filter(|e| {
            e.name.to_lowercase().contains(&query.to_lowercase())
                || e.description.to_lowercase().contains(&query.to_lowercase())
        }).collect())
    }

    pub fn list_aggregate_roots(&self, bp_id: &BlueprintId) -> ForgeResult<Vec<&Entity>> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.entities.iter().filter(|e| e.is_aggregate_root).collect())
    }

    // File queries

    pub fn get_file(&self, bp_id: &BlueprintId, file_id: &FileId) -> ForgeResult<&FileBlueprint> {
        let bp = self.engine.store.load(bp_id)?;
        bp.files.iter().find(|f| f.id == *file_id)
            .ok_or_else(|| ForgeError::FileNotFound(file_id.to_string()))
    }

    pub fn get_file_by_path(&self, bp_id: &BlueprintId, path: &str) -> ForgeResult<&FileBlueprint> {
        let bp = self.engine.store.load(bp_id)?;
        bp.find_file(path)
            .ok_or_else(|| ForgeError::FileNotFound(path.to_string()))
    }

    pub fn list_files(&self, bp_id: &BlueprintId) -> ForgeResult<&[FileBlueprint]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.files)
    }

    pub fn list_files_by_type(&self, bp_id: &BlueprintId, ft: FileType) -> ForgeResult<Vec<&FileBlueprint>> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.files.iter().filter(|f| f.file_type == ft).collect())
    }

    pub fn file_count(&self, bp_id: &BlueprintId) -> ForgeResult<usize> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.file_count())
    }

    // Dependency queries

    pub fn get_dependency(&self, bp_id: &BlueprintId, dep_id: &DependencyId) -> ForgeResult<&Dependency> {
        let bp = self.engine.store.load(bp_id)?;
        bp.dependencies.iter().find(|d| d.id == *dep_id)
            .ok_or_else(|| ForgeError::DependencyNotFound(dep_id.to_string()))
    }

    pub fn get_dependency_by_name(&self, bp_id: &BlueprintId, name: &str) -> ForgeResult<&Dependency> {
        let bp = self.engine.store.load(bp_id)?;
        bp.find_dependency(name)
            .ok_or_else(|| ForgeError::DependencyNotFound(name.to_string()))
    }

    pub fn list_dependencies(&self, bp_id: &BlueprintId) -> ForgeResult<&[Dependency]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.dependencies)
    }

    pub fn list_dependencies_by_type(&self, bp_id: &BlueprintId, dt: DependencyType) -> ForgeResult<Vec<&Dependency>> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.dependencies.iter().filter(|d| d.dep_type == dt).collect())
    }

    pub fn dependency_count(&self, bp_id: &BlueprintId) -> ForgeResult<usize> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.dependency_count())
    }

    // Test queries

    pub fn get_test_case(&self, bp_id: &BlueprintId, tc_id: &TestCaseId) -> ForgeResult<&TestCase> {
        let bp = self.engine.store.load(bp_id)?;
        bp.test_cases.iter().find(|t| t.id == *tc_id)
            .ok_or_else(|| ForgeError::TestCaseNotFound(tc_id.to_string()))
    }

    pub fn list_test_cases(&self, bp_id: &BlueprintId) -> ForgeResult<&[TestCase]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.test_cases)
    }

    pub fn list_tests_by_type(&self, bp_id: &BlueprintId, tt: TestType) -> ForgeResult<Vec<&TestCase>> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.test_cases.iter().filter(|t| t.test_type == tt).collect())
    }

    pub fn test_count(&self, bp_id: &BlueprintId) -> ForgeResult<usize> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(bp.test_count())
    }

    // Type queries

    pub fn list_type_definitions(&self, bp_id: &BlueprintId) -> ForgeResult<&[TypeDefinition]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.type_definitions)
    }

    pub fn get_type_definition(&self, bp_id: &BlueprintId, name: &str) -> ForgeResult<&TypeDefinition> {
        let bp = self.engine.store.load(bp_id)?;
        bp.type_definitions.iter().find(|t| t.name == name)
            .ok_or_else(|| ForgeError::MissingField(name.to_string()))
    }

    // Function queries

    pub fn list_function_blueprints(&self, bp_id: &BlueprintId) -> ForgeResult<&[FunctionBlueprint]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.function_blueprints)
    }

    // Architecture queries

    pub fn list_layers(&self, bp_id: &BlueprintId) -> ForgeResult<&[ArchitectureLayer]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.layers)
    }

    pub fn list_concerns(&self, bp_id: &BlueprintId) -> ForgeResult<&[CrossCuttingConcern]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.concerns)
    }

    // Wiring queries

    pub fn list_wiring(&self, bp_id: &BlueprintId) -> ForgeResult<&[ComponentWiring]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.wiring)
    }

    pub fn list_data_flows(&self, bp_id: &BlueprintId) -> ForgeResult<&[DataFlow]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.data_flows)
    }

    pub fn list_import_graph(&self, bp_id: &BlueprintId) -> ForgeResult<&[ImportEdge]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.import_graph)
    }

    pub fn get_generation_order(&self, bp_id: &BlueprintId) -> ForgeResult<&[String]> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(&bp.generation_order)
    }

    // Validation queries

    pub fn validate_blueprint(&self, bp_id: &BlueprintId) -> ForgeResult<Vec<String>> {
        let bp = self.engine.store.load(bp_id)?;
        let mut issues = Vec::new();

        if bp.name.is_empty() {
            issues.push("Blueprint name is empty".into());
        }
        if bp.entities.is_empty() {
            issues.push("Blueprint has no entities".into());
        }
        if bp.files.is_empty() {
            issues.push("Blueprint has no files".into());
        }

        for entity in &bp.entities {
            if entity.fields.is_empty() {
                issues.push(format!("Entity '{}' has no fields", entity.name));
            }
        }

        for file in &bp.files {
            if file.path.is_empty() {
                issues.push("File has empty path".into());
            }
        }

        Ok(issues)
    }

    pub fn blueprint_summary(&self, bp_id: &BlueprintId) -> ForgeResult<BlueprintSummary> {
        let bp = self.engine.store.load(bp_id)?;
        Ok(BlueprintSummary {
            id: bp.id,
            name: bp.name.clone(),
            domain: bp.domain,
            status: bp.status,
            entity_count: bp.entity_count(),
            file_count: bp.file_count(),
            dependency_count: bp.dependency_count(),
            test_count: bp.test_count(),
            type_count: bp.type_definitions.len(),
            function_count: bp.function_blueprints.len(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlueprintSummary {
    pub id: BlueprintId,
    pub name: String,
    pub domain: Domain,
    pub status: BlueprintStatus,
    pub entity_count: usize,
    pub file_count: usize,
    pub dependency_count: usize,
    pub test_count: usize,
    pub type_count: usize,
    pub function_count: usize,
}

use crate::types::intent::Domain;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ForgeEngine;
    use crate::types::blueprint::*;
    use crate::types::intent::*;

    fn setup() -> (ForgeEngine, BlueprintId) {
        let mut engine = ForgeEngine::new();
        let id = engine.create_blueprint("Test", "Test blueprint", Domain::Api).unwrap();
        // Add some entities, files, deps, tests
        {
            let mut w = engine.writer();
            w.add_entity(&id, Entity::new("User", "A user entity")).unwrap();
            w.add_entity(&id, Entity::new("Post", "A post entity")).unwrap();
            w.add_file(&id, FileBlueprint::new("src/main.rs", FileType::Source)).unwrap();
            w.add_file(&id, FileBlueprint::new("src/models.rs", FileType::Source)).unwrap();
            w.add_file(&id, FileBlueprint::new("tests/test.rs", FileType::Test)).unwrap();
            w.add_dependency(&id, Dependency::new("serde", "1.0")).unwrap();
            w.add_dependency(&id, Dependency::new("tokio", "1.35")).unwrap();
            w.add_test_case(&id, TestCase::new("test_create", TestType::Unit, "User::create")).unwrap();
            w.add_test_case(&id, TestCase::new("test_e2e", TestType::Integration, "api")).unwrap();
        }
        (engine, id)
    }

    #[test]
    fn test_get_blueprint() {
        let (engine, id) = setup();
        let r = engine.reader();
        let bp = r.get_blueprint(&id).unwrap();
        assert_eq!(bp.name, "Test");
    }

    #[test]
    fn test_list_blueprints() {
        let (engine, _) = setup();
        let r = engine.reader();
        assert_eq!(r.list_blueprints().len(), 1);
    }

    #[test]
    fn test_search_blueprints() {
        let (engine, _) = setup();
        let r = engine.reader();
        assert_eq!(r.search_blueprints("test").len(), 1);
        assert_eq!(r.search_blueprints("nonexistent").len(), 0);
    }

    #[test]
    fn test_blueprint_count() {
        let (engine, _) = setup();
        let r = engine.reader();
        assert_eq!(r.blueprint_count(), 1);
    }

    #[test]
    fn test_get_entity() {
        let (engine, id) = setup();
        let r = engine.reader();
        let entities = r.list_entities(&id).unwrap();
        let eid = entities[0].id;
        let entity = r.get_entity(&id, &eid).unwrap();
        assert!(!entity.name.is_empty());
    }

    #[test]
    fn test_get_entity_by_name() {
        let (engine, id) = setup();
        let r = engine.reader();
        let entity = r.get_entity_by_name(&id, "User").unwrap();
        assert_eq!(entity.name, "User");
    }

    #[test]
    fn test_search_entities() {
        let (engine, id) = setup();
        let r = engine.reader();
        let results = r.search_entities(&id, "user").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_entity_count() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.entity_count(&id).unwrap(), 2);
    }

    #[test]
    fn test_list_files() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.list_files(&id).unwrap().len(), 3);
    }

    #[test]
    fn test_list_files_by_type() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.list_files_by_type(&id, FileType::Source).unwrap().len(), 2);
        assert_eq!(r.list_files_by_type(&id, FileType::Test).unwrap().len(), 1);
    }

    #[test]
    fn test_file_count() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.file_count(&id).unwrap(), 3);
    }

    #[test]
    fn test_get_file_by_path() {
        let (engine, id) = setup();
        let r = engine.reader();
        let file = r.get_file_by_path(&id, "src/main.rs").unwrap();
        assert_eq!(file.path, "src/main.rs");
    }

    #[test]
    fn test_list_dependencies() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.list_dependencies(&id).unwrap().len(), 2);
    }

    #[test]
    fn test_get_dependency_by_name() {
        let (engine, id) = setup();
        let r = engine.reader();
        let dep = r.get_dependency_by_name(&id, "serde").unwrap();
        assert_eq!(dep.version, "1.0");
    }

    #[test]
    fn test_dependency_count() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.dependency_count(&id).unwrap(), 2);
    }

    #[test]
    fn test_list_test_cases() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.list_test_cases(&id).unwrap().len(), 2);
    }

    #[test]
    fn test_list_tests_by_type() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.list_tests_by_type(&id, TestType::Unit).unwrap().len(), 1);
        assert_eq!(r.list_tests_by_type(&id, TestType::Integration).unwrap().len(), 1);
    }

    #[test]
    fn test_validate_blueprint() {
        let (engine, id) = setup();
        let r = engine.reader();
        let issues = r.validate_blueprint(&id).unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_blueprint_summary() {
        let (engine, id) = setup();
        let r = engine.reader();
        let summary = r.blueprint_summary(&id).unwrap();
        assert_eq!(summary.name, "Test");
        assert_eq!(summary.entity_count, 2);
        assert_eq!(summary.file_count, 3);
        assert_eq!(summary.dependency_count, 2);
        assert_eq!(summary.test_count, 2);
    }

    #[test]
    fn test_blueprint_not_found() {
        let (engine, _) = setup();
        let fake_id = BlueprintId::new();
        let r = engine.reader();
        assert!(r.get_blueprint(&fake_id).is_err());
    }

    #[test]
    fn test_entity_not_found() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert!(r.get_entity_by_name(&id, "Nonexistent").is_err());
    }

    #[test]
    fn test_list_aggregate_roots() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert_eq!(r.list_aggregate_roots(&id).unwrap().len(), 0);
    }

    #[test]
    fn test_get_generation_order() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert!(r.get_generation_order(&id).unwrap().is_empty());
    }

    #[test]
    fn test_list_wiring() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert!(r.list_wiring(&id).unwrap().is_empty());
    }

    #[test]
    fn test_list_data_flows() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert!(r.list_data_flows(&id).unwrap().is_empty());
    }

    #[test]
    fn test_list_import_graph() {
        let (engine, id) = setup();
        let r = engine.reader();
        assert!(r.list_import_graph(&id).unwrap().is_empty());
    }
}

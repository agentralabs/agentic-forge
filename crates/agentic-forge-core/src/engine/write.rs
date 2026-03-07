//! WriteEngine — mutation operations for blueprints.

use crate::engine::ForgeEngine;
use crate::types::blueprint::*;
use crate::types::ids::*;
use crate::types::intent::*;
use crate::types::{ForgeError, ForgeResult, MAX_DEPENDENCIES, MAX_ENTITIES, MAX_FILES};

pub struct WriteEngine<'a> {
    engine: &'a mut ForgeEngine,
}

impl<'a> WriteEngine<'a> {
    pub fn new(engine: &'a mut ForgeEngine) -> Self {
        Self { engine }
    }

    // Blueprint operations

    pub fn rename_blueprint(
        &mut self,
        id: &BlueprintId,
        name: impl Into<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(id)?;
        bp.name = name.into();
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn set_description(
        &mut self,
        id: &BlueprintId,
        desc: impl Into<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(id)?;
        bp.description = desc.into();
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn set_status(&mut self, id: &BlueprintId, status: BlueprintStatus) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(id)?;
        bp.status = status;
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn set_version(&mut self, id: &BlueprintId, version: impl Into<String>) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(id)?;
        bp.version = version.into();
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn set_metadata(
        &mut self,
        id: &BlueprintId,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(id)?;
        bp.metadata.insert(key.into(), value.into());
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn delete_blueprint(&mut self, id: &BlueprintId) -> ForgeResult<Blueprint> {
        let bp = self.engine.store.delete(id)?;
        self.engine.indexes.remove_blueprint(id);
        self.engine.dirty = true;
        Ok(bp)
    }

    // Entity operations

    pub fn add_entity(&mut self, bp_id: &BlueprintId, entity: Entity) -> ForgeResult<EntityId> {
        let bp = self.engine.store.load_mut(bp_id)?;
        if bp.entities.len() >= MAX_ENTITIES {
            return Err(ForgeError::capacity("entities", MAX_ENTITIES));
        }
        if bp.entities.iter().any(|e| e.name == entity.name) {
            return Err(ForgeError::DuplicateEntity(entity.name.clone()));
        }
        let id = entity.id;
        bp.entities.push(entity);
        bp.touch();
        self.engine.dirty = true;
        Ok(id)
    }

    pub fn add_entity_from_spec(
        &mut self,
        bp_id: &BlueprintId,
        spec: &EntitySpec,
    ) -> ForgeResult<EntityId> {
        let mut entity = Entity::new(&spec.name, &spec.description);
        entity.is_aggregate_root = spec.is_aggregate_root;
        for field in &spec.fields {
            entity.fields.push(EntityField {
                name: field.name.clone(),
                field_type: field.field_type.clone(),
                required: field.required,
                default_value: field.default_value.clone(),
                description: String::new(),
            });
        }
        for op in &spec.operations {
            let mut eop = EntityOperation::new(&op.name, op.operation_type);
            eop.description = op.description.clone();
            eop.is_async = op.is_async;
            eop.return_type = op.return_type.clone();
            eop.error_types = op.error_cases.clone();
            for p in &op.parameters {
                eop.parameters.push(OperationParameter {
                    name: p.name.clone(),
                    param_type: p.param_type.clone(),
                    required: p.required,
                });
            }
            entity.operations.push(eop);
        }
        self.add_entity(bp_id, entity)
    }

    pub fn remove_entity(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
    ) -> ForgeResult<Entity> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let pos = bp
            .entities
            .iter()
            .position(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        let entity = bp.entities.remove(pos);
        bp.touch();
        self.engine.dirty = true;
        Ok(entity)
    }

    pub fn update_entity_name(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
        name: impl Into<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let entity = bp
            .entities
            .iter_mut()
            .find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        entity.name = name.into();
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn add_field_to_entity(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
        field: EntityField,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let entity = bp
            .entities
            .iter_mut()
            .find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        entity.fields.push(field);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn remove_field_from_entity(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
        field_name: &str,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let entity = bp
            .entities
            .iter_mut()
            .find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        let pos = entity
            .fields
            .iter()
            .position(|f| f.name == field_name)
            .ok_or_else(|| ForgeError::MissingField(field_name.to_string()))?;
        entity.fields.remove(pos);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn add_operation_to_entity(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
        op: EntityOperation,
    ) -> ForgeResult<OperationId> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let entity = bp
            .entities
            .iter_mut()
            .find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        let id = op.id;
        entity.operations.push(op);
        bp.touch();
        self.engine.dirty = true;
        Ok(id)
    }

    pub fn remove_operation_from_entity(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
        op_id: &OperationId,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let entity = bp
            .entities
            .iter_mut()
            .find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        let pos = entity
            .operations
            .iter()
            .position(|o| o.id == *op_id)
            .ok_or_else(|| ForgeError::OperationNotFound(op_id.to_string()))?;
        entity.operations.remove(pos);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn add_relationship(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
        rel: Relationship,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let entity = bp
            .entities
            .iter_mut()
            .find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        entity.relationships.push(rel);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn add_validation_rule(
        &mut self,
        bp_id: &BlueprintId,
        entity_id: &EntityId,
        rule: ValidationRule,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let entity = bp
            .entities
            .iter_mut()
            .find(|e| e.id == *entity_id)
            .ok_or_else(|| ForgeError::EntityNotFound(entity_id.to_string()))?;
        entity.validation_rules.push(rule);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    // File operations

    pub fn add_file(&mut self, bp_id: &BlueprintId, file: FileBlueprint) -> ForgeResult<FileId> {
        let bp = self.engine.store.load_mut(bp_id)?;
        if bp.files.len() >= MAX_FILES {
            return Err(ForgeError::capacity("files", MAX_FILES));
        }
        let id = file.id;
        bp.files.push(file);
        bp.touch();
        self.engine.dirty = true;
        Ok(id)
    }

    pub fn remove_file(
        &mut self,
        bp_id: &BlueprintId,
        file_id: &FileId,
    ) -> ForgeResult<FileBlueprint> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let pos = bp
            .files
            .iter()
            .position(|f| f.id == *file_id)
            .ok_or_else(|| ForgeError::FileNotFound(file_id.to_string()))?;
        let file = bp.files.remove(pos);
        bp.touch();
        self.engine.dirty = true;
        Ok(file)
    }

    pub fn update_file_imports(
        &mut self,
        bp_id: &BlueprintId,
        file_id: &FileId,
        imports: Vec<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let file = bp
            .files
            .iter_mut()
            .find(|f| f.id == *file_id)
            .ok_or_else(|| ForgeError::FileNotFound(file_id.to_string()))?;
        file.imports = imports;
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn update_file_exports(
        &mut self,
        bp_id: &BlueprintId,
        file_id: &FileId,
        exports: Vec<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let file = bp
            .files
            .iter_mut()
            .find(|f| f.id == *file_id)
            .ok_or_else(|| ForgeError::FileNotFound(file_id.to_string()))?;
        file.exports = exports;
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    // Dependency operations

    pub fn add_dependency(
        &mut self,
        bp_id: &BlueprintId,
        dep: Dependency,
    ) -> ForgeResult<DependencyId> {
        let bp = self.engine.store.load_mut(bp_id)?;
        if bp.dependencies.len() >= MAX_DEPENDENCIES {
            return Err(ForgeError::capacity("dependencies", MAX_DEPENDENCIES));
        }
        if bp.dependencies.iter().any(|d| d.name == dep.name) {
            return Err(ForgeError::DuplicateDependency(dep.name.clone()));
        }
        let id = dep.id;
        bp.dependencies.push(dep);
        bp.touch();
        self.engine.dirty = true;
        Ok(id)
    }

    pub fn remove_dependency(
        &mut self,
        bp_id: &BlueprintId,
        dep_id: &DependencyId,
    ) -> ForgeResult<Dependency> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let pos = bp
            .dependencies
            .iter()
            .position(|d| d.id == *dep_id)
            .ok_or_else(|| ForgeError::DependencyNotFound(dep_id.to_string()))?;
        let dep = bp.dependencies.remove(pos);
        bp.touch();
        self.engine.dirty = true;
        Ok(dep)
    }

    pub fn update_dependency_version(
        &mut self,
        bp_id: &BlueprintId,
        dep_id: &DependencyId,
        version: impl Into<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let dep = bp
            .dependencies
            .iter_mut()
            .find(|d| d.id == *dep_id)
            .ok_or_else(|| ForgeError::DependencyNotFound(dep_id.to_string()))?;
        dep.version = version.into();
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    // Test operations

    pub fn add_test_case(&mut self, bp_id: &BlueprintId, tc: TestCase) -> ForgeResult<TestCaseId> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let id = tc.id;
        bp.test_cases.push(tc);
        bp.touch();
        self.engine.dirty = true;
        Ok(id)
    }

    pub fn remove_test_case(
        &mut self,
        bp_id: &BlueprintId,
        tc_id: &TestCaseId,
    ) -> ForgeResult<TestCase> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let pos = bp
            .test_cases
            .iter()
            .position(|t| t.id == *tc_id)
            .ok_or_else(|| ForgeError::TestCaseNotFound(tc_id.to_string()))?;
        let tc = bp.test_cases.remove(pos);
        bp.touch();
        self.engine.dirty = true;
        Ok(tc)
    }

    // Type definition operations

    pub fn add_type_definition(
        &mut self,
        bp_id: &BlueprintId,
        td: TypeDefinition,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.type_definitions.push(td);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn remove_type_definition(&mut self, bp_id: &BlueprintId, name: &str) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        let pos = bp
            .type_definitions
            .iter()
            .position(|t| t.name == name)
            .ok_or_else(|| ForgeError::MissingField(name.to_string()))?;
        bp.type_definitions.remove(pos);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    // Function blueprint operations

    pub fn add_function_blueprint(
        &mut self,
        bp_id: &BlueprintId,
        fb: FunctionBlueprint,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.function_blueprints.push(fb);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    // Architecture operations

    pub fn add_layer(&mut self, bp_id: &BlueprintId, layer: ArchitectureLayer) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.layers.push(layer);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn add_concern(
        &mut self,
        bp_id: &BlueprintId,
        concern: CrossCuttingConcern,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.concerns.push(concern);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    // Wiring operations

    pub fn add_wiring(&mut self, bp_id: &BlueprintId, wiring: ComponentWiring) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.wiring.push(wiring);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn add_data_flow(&mut self, bp_id: &BlueprintId, flow: DataFlow) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.data_flows.push(flow);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn add_import_edge(&mut self, bp_id: &BlueprintId, edge: ImportEdge) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.import_graph.push(edge);
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }

    pub fn set_generation_order(
        &mut self,
        bp_id: &BlueprintId,
        order: Vec<String>,
    ) -> ForgeResult<()> {
        let bp = self.engine.store.load_mut(bp_id)?;
        bp.generation_order = order;
        bp.touch();
        self.engine.dirty = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ForgeEngine;

    fn setup() -> (ForgeEngine, BlueprintId) {
        let mut engine = ForgeEngine::new();
        let id = engine
            .create_blueprint("Test", "Test blueprint", Domain::Api)
            .unwrap();
        (engine, id)
    }

    #[test]
    fn test_rename_blueprint() {
        let (mut engine, id) = setup();
        engine.writer().rename_blueprint(&id, "Renamed").unwrap();
        assert_eq!(engine.store.load(&id).unwrap().name, "Renamed");
    }

    #[test]
    fn test_set_description() {
        let (mut engine, id) = setup();
        engine.writer().set_description(&id, "New desc").unwrap();
        assert_eq!(engine.store.load(&id).unwrap().description, "New desc");
    }

    #[test]
    fn test_set_status() {
        let (mut engine, id) = setup();
        engine
            .writer()
            .set_status(&id, BlueprintStatus::Complete)
            .unwrap();
        assert_eq!(
            engine.store.load(&id).unwrap().status,
            BlueprintStatus::Complete
        );
    }

    #[test]
    fn test_set_version() {
        let (mut engine, id) = setup();
        engine.writer().set_version(&id, "1.0.0").unwrap();
        assert_eq!(engine.store.load(&id).unwrap().version, "1.0.0");
    }

    #[test]
    fn test_set_metadata() {
        let (mut engine, id) = setup();
        engine.writer().set_metadata(&id, "key", "value").unwrap();
        assert_eq!(
            engine.store.load(&id).unwrap().metadata.get("key").unwrap(),
            "value"
        );
    }

    #[test]
    fn test_add_entity() {
        let (mut engine, id) = setup();
        let entity = Entity::new("User", "A user");
        let eid = engine.writer().add_entity(&id, entity).unwrap();
        let bp = engine.store.load(&id).unwrap();
        assert_eq!(bp.entity_count(), 1);
        assert!(bp.find_entity_by_id(&eid).is_some());
    }

    #[test]
    fn test_add_duplicate_entity() {
        let (mut engine, id) = setup();
        engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        let result = engine.writer().add_entity(&id, Entity::new("User", "B"));
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_entity() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        engine.writer().remove_entity(&id, &eid).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().entity_count(), 0);
    }

    #[test]
    fn test_update_entity_name() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        engine
            .writer()
            .update_entity_name(&id, &eid, "Account")
            .unwrap();
        assert_eq!(
            engine
                .store
                .load(&id)
                .unwrap()
                .find_entity_by_id(&eid)
                .unwrap()
                .name,
            "Account"
        );
    }

    #[test]
    fn test_add_field_to_entity() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        let field = EntityField::new("name", FieldType::String);
        engine
            .writer()
            .add_field_to_entity(&id, &eid, field)
            .unwrap();
        let bp = engine.store.load(&id).unwrap();
        assert_eq!(bp.find_entity_by_id(&eid).unwrap().fields.len(), 1);
    }

    #[test]
    fn test_remove_field_from_entity() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        let field = EntityField::new("name", FieldType::String);
        engine
            .writer()
            .add_field_to_entity(&id, &eid, field)
            .unwrap();
        engine
            .writer()
            .remove_field_from_entity(&id, &eid, "name")
            .unwrap();
        assert_eq!(
            engine
                .store
                .load(&id)
                .unwrap()
                .find_entity_by_id(&eid)
                .unwrap()
                .fields
                .len(),
            0
        );
    }

    #[test]
    fn test_add_operation_to_entity() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        let op = EntityOperation::new("create", OperationType::Create);
        engine
            .writer()
            .add_operation_to_entity(&id, &eid, op)
            .unwrap();
        assert_eq!(
            engine
                .store
                .load(&id)
                .unwrap()
                .find_entity_by_id(&eid)
                .unwrap()
                .operations
                .len(),
            1
        );
    }

    #[test]
    fn test_add_file() {
        let (mut engine, id) = setup();
        let file = FileBlueprint::new("src/main.rs", FileType::Source);
        engine.writer().add_file(&id, file).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().file_count(), 1);
    }

    #[test]
    fn test_remove_file() {
        let (mut engine, id) = setup();
        let file = FileBlueprint::new("src/main.rs", FileType::Source);
        let fid = engine.writer().add_file(&id, file).unwrap();
        engine.writer().remove_file(&id, &fid).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().file_count(), 0);
    }

    #[test]
    fn test_add_dependency() {
        let (mut engine, id) = setup();
        let dep = Dependency::new("serde", "1.0");
        engine.writer().add_dependency(&id, dep).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().dependency_count(), 1);
    }

    #[test]
    fn test_add_duplicate_dependency() {
        let (mut engine, id) = setup();
        engine
            .writer()
            .add_dependency(&id, Dependency::new("serde", "1.0"))
            .unwrap();
        let result = engine
            .writer()
            .add_dependency(&id, Dependency::new("serde", "2.0"));
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_dependency() {
        let (mut engine, id) = setup();
        let did = engine
            .writer()
            .add_dependency(&id, Dependency::new("serde", "1.0"))
            .unwrap();
        engine.writer().remove_dependency(&id, &did).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().dependency_count(), 0);
    }

    #[test]
    fn test_update_dependency_version() {
        let (mut engine, id) = setup();
        let did = engine
            .writer()
            .add_dependency(&id, Dependency::new("serde", "1.0"))
            .unwrap();
        engine
            .writer()
            .update_dependency_version(&id, &did, "2.0")
            .unwrap();
        assert_eq!(
            engine
                .store
                .load(&id)
                .unwrap()
                .find_dependency("serde")
                .unwrap()
                .version,
            "2.0"
        );
    }

    #[test]
    fn test_add_test_case() {
        let (mut engine, id) = setup();
        let tc = TestCase::new("test_create", TestType::Unit, "User::create");
        engine.writer().add_test_case(&id, tc).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().test_count(), 1);
    }

    #[test]
    fn test_add_type_definition() {
        let (mut engine, id) = setup();
        let td = TypeDefinition::new("User", TypeKind::Struct);
        engine.writer().add_type_definition(&id, td).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().type_definitions.len(), 1);
    }

    #[test]
    fn test_add_function_blueprint() {
        let (mut engine, id) = setup();
        let fb = FunctionBlueprint::new("create_user");
        engine.writer().add_function_blueprint(&id, fb).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().function_blueprints.len(), 1);
    }

    #[test]
    fn test_add_layer() {
        let (mut engine, id) = setup();
        let layer = ArchitectureLayer {
            name: "domain".into(),
            description: "Domain layer".into(),
            modules: vec!["models".into()],
            allowed_dependencies: vec![],
        };
        engine.writer().add_layer(&id, layer).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().layers.len(), 1);
    }

    #[test]
    fn test_add_concern() {
        let (mut engine, id) = setup();
        let concern = CrossCuttingConcern {
            name: "logging".into(),
            concern_type: ConcernType::Logging,
            affected_layers: vec!["all".into()],
            implementation_strategy: "tracing".into(),
        };
        engine.writer().add_concern(&id, concern).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().concerns.len(), 1);
    }

    #[test]
    fn test_add_wiring() {
        let (mut engine, id) = setup();
        let wiring = ComponentWiring {
            source: "UserService".into(),
            target: "UserRepository".into(),
            wiring_type: WiringType::DependencyInjection,
            description: "Service depends on repo".into(),
        };
        engine.writer().add_wiring(&id, wiring).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().wiring.len(), 1);
    }

    #[test]
    fn test_add_data_flow() {
        let (mut engine, id) = setup();
        let flow = DataFlow {
            source: "API".into(),
            target: "Database".into(),
            data_type: "User".into(),
            direction: FlowDirection::Unidirectional,
            is_async: true,
        };
        engine.writer().add_data_flow(&id, flow).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().data_flows.len(), 1);
    }

    #[test]
    fn test_add_import_edge() {
        let (mut engine, id) = setup();
        let edge = ImportEdge {
            from_file: "src/main.rs".into(),
            to_file: "src/lib.rs".into(),
            imported_symbols: vec!["App".into()],
        };
        engine.writer().add_import_edge(&id, edge).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().import_graph.len(), 1);
    }

    #[test]
    fn test_set_generation_order() {
        let (mut engine, id) = setup();
        let order = vec!["types.rs".into(), "models.rs".into(), "main.rs".into()];
        engine.writer().set_generation_order(&id, order).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().generation_order.len(), 3);
    }

    #[test]
    fn test_delete_blueprint() {
        let (mut engine, id) = setup();
        engine.writer().delete_blueprint(&id).unwrap();
        assert_eq!(engine.blueprint_count(), 0);
    }

    #[test]
    fn test_add_entity_from_spec() {
        let (mut engine, id) = setup();
        let spec = EntitySpec::new("User", "A user")
            .with_field(FieldSpec::new("name", FieldType::String))
            .with_operation(OperationSpec::new("create", OperationType::Create).async_op());
        engine.writer().add_entity_from_spec(&id, &spec).unwrap();
        let bp = engine.store.load(&id).unwrap();
        let entity = bp.find_entity("User").unwrap();
        assert_eq!(entity.fields.len(), 1);
        assert_eq!(entity.operations.len(), 1);
        assert!(entity.operations[0].is_async);
    }

    #[test]
    fn test_add_relationship() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        let rel = Relationship {
            target_entity: "Post".into(),
            relationship_type: RelationshipType::HasMany,
            cardinality: Cardinality::OneToMany,
            description: "User has many posts".into(),
        };
        engine.writer().add_relationship(&id, &eid, rel).unwrap();
        let entity = engine
            .store
            .load(&id)
            .unwrap()
            .find_entity_by_id(&eid)
            .unwrap();
        assert_eq!(entity.relationships.len(), 1);
    }

    #[test]
    fn test_add_validation_rule() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        let rule = ValidationRule {
            field: "email".into(),
            rule_type: "format".into(),
            parameters: std::collections::HashMap::new(),
            message: "Invalid email".into(),
        };
        engine
            .writer()
            .add_validation_rule(&id, &eid, rule)
            .unwrap();
    }

    #[test]
    fn test_update_file_imports() {
        let (mut engine, id) = setup();
        let fid = engine
            .writer()
            .add_file(&id, FileBlueprint::new("src/main.rs", FileType::Source))
            .unwrap();
        engine
            .writer()
            .update_file_imports(&id, &fid, vec!["std::io".into()])
            .unwrap();
        let file = engine
            .store
            .load(&id)
            .unwrap()
            .files
            .iter()
            .find(|f| f.id == fid)
            .unwrap();
        assert_eq!(file.imports.len(), 1);
    }

    #[test]
    fn test_update_file_exports() {
        let (mut engine, id) = setup();
        let fid = engine
            .writer()
            .add_file(&id, FileBlueprint::new("src/lib.rs", FileType::Source))
            .unwrap();
        engine
            .writer()
            .update_file_exports(&id, &fid, vec!["App".into()])
            .unwrap();
        let file = engine
            .store
            .load(&id)
            .unwrap()
            .files
            .iter()
            .find(|f| f.id == fid)
            .unwrap();
        assert_eq!(file.exports.len(), 1);
    }

    #[test]
    fn test_remove_operation_from_entity() {
        let (mut engine, id) = setup();
        let eid = engine
            .writer()
            .add_entity(&id, Entity::new("User", "A"))
            .unwrap();
        let op = EntityOperation::new("create", OperationType::Create);
        let oid = engine
            .writer()
            .add_operation_to_entity(&id, &eid, op)
            .unwrap();
        engine
            .writer()
            .remove_operation_from_entity(&id, &eid, &oid)
            .unwrap();
        assert_eq!(
            engine
                .store
                .load(&id)
                .unwrap()
                .find_entity_by_id(&eid)
                .unwrap()
                .operations
                .len(),
            0
        );
    }

    #[test]
    fn test_remove_type_definition() {
        let (mut engine, id) = setup();
        engine
            .writer()
            .add_type_definition(&id, TypeDefinition::new("User", TypeKind::Struct))
            .unwrap();
        engine.writer().remove_type_definition(&id, "User").unwrap();
        assert_eq!(engine.store.load(&id).unwrap().type_definitions.len(), 0);
    }

    #[test]
    fn test_remove_test_case() {
        let (mut engine, id) = setup();
        let tcid = engine
            .writer()
            .add_test_case(&id, TestCase::new("test_a", TestType::Unit, "A"))
            .unwrap();
        engine.writer().remove_test_case(&id, &tcid).unwrap();
        assert_eq!(engine.store.load(&id).unwrap().test_count(), 0);
    }
}

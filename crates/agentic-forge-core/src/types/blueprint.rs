//! Blueprint types — the core output of the Forge engine.

use crate::types::ids::*;
use crate::types::intent::{Domain, FieldType, OperationType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub id: BlueprintId,
    pub name: String,
    pub description: String,
    pub domain: Domain,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub entities: Vec<Entity>,
    pub files: Vec<FileBlueprint>,
    pub dependencies: Vec<Dependency>,
    pub test_cases: Vec<TestCase>,
    pub type_definitions: Vec<TypeDefinition>,
    pub function_blueprints: Vec<FunctionBlueprint>,
    pub metadata: HashMap<String, String>,
    pub layers: Vec<ArchitectureLayer>,
    pub concerns: Vec<CrossCuttingConcern>,
    pub generation_order: Vec<String>,
    pub wiring: Vec<ComponentWiring>,
    pub data_flows: Vec<DataFlow>,
    pub import_graph: Vec<ImportEdge>,
    pub status: BlueprintStatus,
}

impl Blueprint {
    pub fn new(name: impl Into<String>, description: impl Into<String>, domain: Domain) -> Self {
        let now = Utc::now();
        Self {
            id: BlueprintId::new(),
            name: name.into(),
            description: description.into(),
            domain,
            version: "0.1.0".into(),
            created_at: now,
            updated_at: now,
            entities: Vec::new(),
            files: Vec::new(),
            dependencies: Vec::new(),
            test_cases: Vec::new(),
            type_definitions: Vec::new(),
            function_blueprints: Vec::new(),
            metadata: HashMap::new(),
            layers: Vec::new(),
            concerns: Vec::new(),
            generation_order: Vec::new(),
            wiring: Vec::new(),
            data_flows: Vec::new(),
            import_graph: Vec::new(),
            status: BlueprintStatus::Draft,
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    pub fn test_count(&self) -> usize {
        self.test_cases.len()
    }

    pub fn find_entity(&self, name: &str) -> Option<&Entity> {
        self.entities.iter().find(|e| e.name == name)
    }

    pub fn find_entity_by_id(&self, id: &EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == *id)
    }

    pub fn find_file(&self, path: &str) -> Option<&FileBlueprint> {
        self.files.iter().find(|f| f.path == path)
    }

    pub fn find_dependency(&self, name: &str) -> Option<&Dependency> {
        self.dependencies.iter().find(|d| d.name == name)
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.status != BlueprintStatus::Invalid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlueprintStatus {
    Draft,
    InProgress,
    Complete,
    Validated,
    Exported,
    Invalid,
}

impl BlueprintStatus {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InProgress => "in_progress",
            Self::Complete => "complete",
            Self::Validated => "validated",
            Self::Exported => "exported",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub description: String,
    pub fields: Vec<EntityField>,
    pub operations: Vec<EntityOperation>,
    pub relationships: Vec<Relationship>,
    pub validation_rules: Vec<ValidationRule>,
    pub is_aggregate_root: bool,
    pub layer: Option<String>,
}

impl Entity {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: EntityId::new(),
            name: name.into(),
            description: description.into(),
            fields: Vec::new(),
            operations: Vec::new(),
            relationships: Vec::new(),
            validation_rules: Vec::new(),
            is_aggregate_root: false,
            layer: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
}

impl EntityField {
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            required: true,
            default_value: None,
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityOperation {
    pub id: OperationId,
    pub name: String,
    pub description: String,
    pub operation_type: OperationType,
    pub parameters: Vec<OperationParameter>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub error_types: Vec<String>,
}

impl EntityOperation {
    pub fn new(name: impl Into<String>, op_type: OperationType) -> Self {
        Self {
            id: OperationId::new(),
            name: name.into(),
            description: String::new(),
            operation_type: op_type,
            parameters: Vec::new(),
            return_type: None,
            is_async: false,
            error_types: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target_entity: String,
    pub relationship_type: RelationshipType,
    pub cardinality: Cardinality,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    HasOne,
    HasMany,
    BelongsTo,
    ManyToMany,
    References,
    Contains,
    DependsOn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule_type: String,
    pub parameters: HashMap<String, String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBlueprint {
    pub id: FileId,
    pub path: String,
    pub file_type: FileType,
    pub module: String,
    pub description: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub type_definitions: Vec<String>,
    pub function_signatures: Vec<String>,
    pub estimated_lines: usize,
}

impl FileBlueprint {
    pub fn new(path: impl Into<String>, file_type: FileType) -> Self {
        Self {
            id: FileId::new(),
            path: path.into(),
            file_type,
            module: String::new(),
            description: String::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            type_definitions: Vec::new(),
            function_signatures: Vec::new(),
            estimated_lines: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Source,
    Test,
    Config,
    Migration,
    Schema,
    Documentation,
    Build,
    Asset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: TypeKind,
    pub fields: Vec<TypeField>,
    pub derives: Vec<String>,
    pub visibility: Visibility,
    pub doc_comment: String,
}

impl TypeDefinition {
    pub fn new(name: impl Into<String>, kind: TypeKind) -> Self {
        Self {
            name: name.into(),
            kind,
            fields: Vec::new(),
            derives: Vec::new(),
            visibility: Visibility::Public,
            doc_comment: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Newtype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeField {
    pub name: String,
    pub field_type: String,
    pub visibility: Visibility,
    pub doc_comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBlueprint {
    pub name: String,
    pub parameters: Vec<FunctionParam>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub is_pub: bool,
    pub doc_comment: String,
    pub body_hint: String,
    pub error_handling: ErrorHandling,
}

impl FunctionBlueprint {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parameters: Vec::new(),
            return_type: None,
            is_async: false,
            is_pub: true,
            doc_comment: String::new(),
            body_hint: String::new(),
            error_handling: ErrorHandling::Result,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: String,
    pub is_reference: bool,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorHandling {
    Result,
    Option,
    Panic,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: DependencyId,
    pub name: String,
    pub version: String,
    pub dep_type: DependencyType,
    pub features: Vec<String>,
    pub optional: bool,
    pub source: DependencySource,
}

impl Dependency {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: DependencyId::new(),
            name: name.into(),
            version: version.into(),
            dep_type: DependencyType::Runtime,
            features: Vec::new(),
            optional: false,
            source: DependencySource::Registry,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    Runtime,
    Build,
    Dev,
    Optional,
    Peer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    Registry,
    Git(String),
    Path(String),
    Url(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: TestCaseId,
    pub name: String,
    pub description: String,
    pub test_type: TestType,
    pub target: String,
    pub assertions: Vec<Assertion>,
    pub setup: Vec<String>,
    pub teardown: Vec<String>,
    pub tags: Vec<String>,
}

impl TestCase {
    pub fn new(name: impl Into<String>, test_type: TestType, target: impl Into<String>) -> Self {
        Self {
            id: TestCaseId::new(),
            name: name.into(),
            description: String::new(),
            test_type,
            target: target.into(),
            assertions: Vec::new(),
            setup: Vec::new(),
            teardown: Vec::new(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    EndToEnd,
    Property,
    Snapshot,
    Performance,
    Fuzz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub description: String,
    pub assertion_type: AssertionType,
    pub expected: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertionType {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    IsTrue,
    IsFalse,
    IsNone,
    IsSome,
    IsOk,
    IsErr,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureLayer {
    pub name: String,
    pub description: String,
    pub modules: Vec<String>,
    pub allowed_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCuttingConcern {
    pub name: String,
    pub concern_type: ConcernType,
    pub affected_layers: Vec<String>,
    pub implementation_strategy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcernType {
    Logging,
    Authentication,
    Authorization,
    Caching,
    ErrorHandling,
    Validation,
    Monitoring,
    Serialization,
    Configuration,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentWiring {
    pub source: String,
    pub target: String,
    pub wiring_type: WiringType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WiringType {
    DirectCall,
    EventDriven,
    MessageQueue,
    SharedState,
    DependencyInjection,
    Callback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub source: String,
    pub target: String,
    pub data_type: String,
    pub direction: FlowDirection,
    pub is_async: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowDirection {
    Unidirectional,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEdge {
    pub from_file: String,
    pub to_file: String,
    pub imported_symbols: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blueprint_creation() {
        let bp = Blueprint::new("TestProject", "A test blueprint", Domain::Api);
        assert_eq!(bp.name, "TestProject");
        assert_eq!(bp.domain, Domain::Api);
        assert_eq!(bp.status, BlueprintStatus::Draft);
        assert!(bp.is_valid());
    }

    #[test]
    fn test_blueprint_entity_management() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Web);
        bp.entities.push(Entity::new("User", "A user"));
        bp.entities.push(Entity::new("Post", "A post"));
        assert_eq!(bp.entity_count(), 2);
        assert!(bp.find_entity("User").is_some());
        assert!(bp.find_entity("Missing").is_none());
    }

    #[test]
    fn test_blueprint_file_management() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Cli);
        bp.files.push(FileBlueprint::new("src/main.rs", FileType::Source));
        assert_eq!(bp.file_count(), 1);
        assert!(bp.find_file("src/main.rs").is_some());
    }

    #[test]
    fn test_blueprint_dependency_management() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Library);
        bp.dependencies.push(Dependency::new("serde", "1.0"));
        assert_eq!(bp.dependency_count(), 1);
        assert!(bp.find_dependency("serde").is_some());
    }

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("User", "A user entity");
        assert_eq!(entity.name, "User");
        assert!(!entity.is_aggregate_root);
    }

    #[test]
    fn test_file_blueprint_creation() {
        let file = FileBlueprint::new("src/lib.rs", FileType::Source);
        assert_eq!(file.path, "src/lib.rs");
        assert_eq!(file.file_type, FileType::Source);
    }

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new("tokio", "1.35");
        assert_eq!(dep.name, "tokio");
        assert_eq!(dep.dep_type, DependencyType::Runtime);
        assert!(!dep.optional);
    }

    #[test]
    fn test_test_case_creation() {
        let tc = TestCase::new("test_user_create", TestType::Unit, "User::create");
        assert_eq!(tc.name, "test_user_create");
        assert_eq!(tc.test_type, TestType::Unit);
    }

    #[test]
    fn test_blueprint_status_name() {
        assert_eq!(BlueprintStatus::Draft.name(), "draft");
        assert_eq!(BlueprintStatus::Validated.name(), "validated");
        assert_eq!(BlueprintStatus::Exported.name(), "exported");
    }

    #[test]
    fn test_type_definition_creation() {
        let td = TypeDefinition::new("User", TypeKind::Struct);
        assert_eq!(td.name, "User");
        assert_eq!(td.kind, TypeKind::Struct);
        assert_eq!(td.visibility, Visibility::Public);
    }

    #[test]
    fn test_function_blueprint_creation() {
        let fb = FunctionBlueprint::new("create_user");
        assert_eq!(fb.name, "create_user");
        assert!(fb.is_pub);
        assert!(!fb.is_async);
    }

    #[test]
    fn test_blueprint_serialization() {
        let bp = Blueprint::new("Test", "A test", Domain::Api);
        let json = serde_json::to_string(&bp).unwrap();
        let parsed: Blueprint = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test");
        assert_eq!(parsed.id, bp.id);
    }

    #[test]
    fn test_blueprint_touch() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Web);
        let orig = bp.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        bp.touch();
        assert!(bp.updated_at >= orig);
    }

    #[test]
    fn test_invalid_blueprint() {
        let mut bp = Blueprint::new("", "Empty name", Domain::Web);
        assert!(!bp.is_valid());
        bp.name = "Valid".into();
        assert!(bp.is_valid());
        bp.status = BlueprintStatus::Invalid;
        assert!(!bp.is_valid());
    }
}

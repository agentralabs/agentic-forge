//! Intent specification types for blueprint creation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSpec {
    pub name: String,
    pub description: String,
    pub domain: Domain,
    pub entities: Vec<EntitySpec>,
    pub constraints: Vec<Constraint>,
    pub target_language: String,
    pub target_framework: Option<String>,
}

impl IntentSpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>, domain: Domain) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            domain,
            entities: Vec::new(),
            constraints: Vec::new(),
            target_language: "rust".into(),
            target_framework: None,
        }
    }

    pub fn with_entity(mut self, entity: EntitySpec) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.target_language = lang.into();
        self
    }

    pub fn with_framework(mut self, framework: impl Into<String>) -> Self {
        self.target_framework = Some(framework.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Domain {
    Web = 0,
    Api = 1,
    Cli = 2,
    Library = 3,
    Service = 4,
    Database = 5,
    Embedded = 6,
    Mobile = 7,
    Desktop = 8,
    Plugin = 9,
    Custom = 255,
}

impl Domain {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Web),
            1 => Some(Self::Api),
            2 => Some(Self::Cli),
            3 => Some(Self::Library),
            4 => Some(Self::Service),
            5 => Some(Self::Database),
            6 => Some(Self::Embedded),
            7 => Some(Self::Mobile),
            8 => Some(Self::Desktop),
            9 => Some(Self::Plugin),
            255 => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Api => "api",
            Self::Cli => "cli",
            Self::Library => "library",
            Self::Service => "service",
            Self::Database => "database",
            Self::Embedded => "embedded",
            Self::Mobile => "mobile",
            Self::Desktop => "desktop",
            Self::Plugin => "plugin",
            Self::Custom => "custom",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "web" => Some(Self::Web),
            "api" => Some(Self::Api),
            "cli" => Some(Self::Cli),
            "library" | "lib" => Some(Self::Library),
            "service" | "svc" => Some(Self::Service),
            "database" | "db" => Some(Self::Database),
            "embedded" => Some(Self::Embedded),
            "mobile" => Some(Self::Mobile),
            "desktop" => Some(Self::Desktop),
            "plugin" => Some(Self::Plugin),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySpec {
    pub name: String,
    pub description: String,
    pub fields: Vec<FieldSpec>,
    pub operations: Vec<OperationSpec>,
    pub is_aggregate_root: bool,
}

impl EntitySpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            fields: Vec::new(),
            operations: Vec::new(),
            is_aggregate_root: false,
        }
    }

    pub fn with_field(mut self, field: FieldSpec) -> Self {
        self.fields.push(field);
        self
    }

    pub fn with_operation(mut self, op: OperationSpec) -> Self {
        self.operations.push(op);
        self
    }

    pub fn as_aggregate_root(mut self) -> Self {
        self.is_aggregate_root = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSpec {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: Option<String>,
    pub constraints: Vec<FieldConstraint>,
}

impl FieldSpec {
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            required: true,
            default_value: None,
            constraints: Vec::new(),
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }

    pub fn with_constraint(mut self, constraint: FieldConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Uuid,
    Binary,
    Json,
    Array(Box<FieldType>),
    Optional(Box<FieldType>),
    Reference(String),
    Enum(Vec<String>),
    Custom(String),
}

impl FieldType {
    pub fn name(&self) -> String {
        match self {
            Self::String => "String".into(),
            Self::Integer => "i64".into(),
            Self::Float => "f64".into(),
            Self::Boolean => "bool".into(),
            Self::DateTime => "DateTime".into(),
            Self::Uuid => "Uuid".into(),
            Self::Binary => "Vec<u8>".into(),
            Self::Json => "Value".into(),
            Self::Array(inner) => format!("Vec<{}>", inner.name()),
            Self::Optional(inner) => format!("Option<{}>", inner.name()),
            Self::Reference(r) => r.clone(),
            Self::Enum(variants) => format!("Enum({})", variants.join("|")),
            Self::Custom(c) => c.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldConstraint {
    MinLength(usize),
    MaxLength(usize),
    Min(f64),
    Max(f64),
    Pattern(String),
    Unique,
    NotNull,
    ForeignKey(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSpec {
    pub name: String,
    pub description: String,
    pub operation_type: OperationType,
    pub parameters: Vec<ParameterSpec>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub error_cases: Vec<String>,
}

impl OperationSpec {
    pub fn new(name: impl Into<String>, op_type: OperationType) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            operation_type: op_type,
            parameters: Vec::new(),
            return_type: None,
            is_async: false,
            error_cases: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_param(mut self, param: ParameterSpec) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn with_return(mut self, return_type: impl Into<String>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    pub fn async_op(mut self) -> Self {
        self.is_async = true;
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_cases.push(error.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Create,
    Read,
    Update,
    Delete,
    Query,
    Command,
    Event,
    Validation,
    Transform,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

impl ParameterSpec {
    pub fn new(name: impl Into<String>, param_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            required: true,
            description: String::new(),
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub description: String,
}

impl Constraint {
    pub fn new(name: impl Into<String>, ct: ConstraintType, desc: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constraint_type: ct,
            description: desc.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Performance(String),
    Security(String),
    Compatibility(String),
    Concurrency(String),
    DataIntegrity(String),
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_spec_builder() {
        let intent = IntentSpec::new("MyProject", "A test project", Domain::Api)
            .with_language("rust")
            .with_framework("axum");
        assert_eq!(intent.name, "MyProject");
        assert_eq!(intent.domain, Domain::Api);
        assert_eq!(intent.target_language, "rust");
        assert_eq!(intent.target_framework.as_deref(), Some("axum"));
    }

    #[test]
    fn test_domain_roundtrip() {
        for v in [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255] {
            let d = Domain::from_u8(v).unwrap();
            let name = d.name();
            let parsed = Domain::from_name(name).unwrap();
            assert_eq!(d, parsed);
        }
    }

    #[test]
    fn test_domain_from_name_aliases() {
        assert_eq!(Domain::from_name("lib"), Some(Domain::Library));
        assert_eq!(Domain::from_name("svc"), Some(Domain::Service));
        assert_eq!(Domain::from_name("db"), Some(Domain::Database));
    }

    #[test]
    fn test_entity_spec_builder() {
        let entity = EntitySpec::new("User", "A user entity")
            .with_field(FieldSpec::new("name", FieldType::String))
            .with_field(
                FieldSpec::new("email", FieldType::String).with_constraint(FieldConstraint::Unique),
            )
            .as_aggregate_root();
        assert_eq!(entity.name, "User");
        assert!(entity.is_aggregate_root);
        assert_eq!(entity.fields.len(), 2);
    }

    #[test]
    fn test_field_type_name() {
        assert_eq!(FieldType::String.name(), "String");
        assert_eq!(FieldType::Integer.name(), "i64");
        assert_eq!(
            FieldType::Array(Box::new(FieldType::String)).name(),
            "Vec<String>"
        );
        assert_eq!(
            FieldType::Optional(Box::new(FieldType::Integer)).name(),
            "Option<i64>"
        );
    }

    #[test]
    fn test_operation_spec_builder() {
        let op = OperationSpec::new("create_user", OperationType::Create)
            .with_description("Create a new user")
            .with_param(ParameterSpec::new("name", "String"))
            .with_return("User")
            .async_op()
            .with_error("DuplicateEmail");
        assert_eq!(op.name, "create_user");
        assert!(op.is_async);
        assert_eq!(op.error_cases.len(), 1);
    }

    #[test]
    fn test_constraint_creation() {
        let c = Constraint::new(
            "max_latency",
            ConstraintType::Performance("< 100ms".into()),
            "API must respond under 100ms",
        );
        assert_eq!(c.name, "max_latency");
    }

    #[test]
    fn test_intent_serialization() {
        let intent = IntentSpec::new("Test", "Test project", Domain::Web);
        let json = serde_json::to_string(&intent).unwrap();
        let parsed: IntentSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test");
    }
}

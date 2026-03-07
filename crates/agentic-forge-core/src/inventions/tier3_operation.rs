//! Tier 3: Operation inventions.
//! OperationInferrer, SignatureGenerator, ErrorFlowDesigner, AsyncAnalyzer

use crate::types::blueprint::*;
use crate::types::intent::*;

pub struct OperationInferrer;

impl OperationInferrer {
    pub fn infer_operations(entity: &Entity) -> Vec<EntityOperation> {
        let mut ops = Vec::new();
        ops.push(EntityOperation::new(
            format!("create_{}", entity.name.to_lowercase()),
            OperationType::Create,
        ));
        ops.push(EntityOperation::new(
            format!("get_{}", entity.name.to_lowercase()),
            OperationType::Read,
        ));
        ops.push(EntityOperation::new(
            format!("update_{}", entity.name.to_lowercase()),
            OperationType::Update,
        ));
        ops.push(EntityOperation::new(
            format!("delete_{}", entity.name.to_lowercase()),
            OperationType::Delete,
        ));
        ops.push(EntityOperation::new(
            format!("list_{}s", entity.name.to_lowercase()),
            OperationType::Query,
        ));

        for field in &entity.fields {
            if field.name != "id" && field.name != "created_at" && field.name != "updated_at" {
                ops.push(EntityOperation::new(
                    format!("find_{}_by_{}", entity.name.to_lowercase(), field.name),
                    OperationType::Query,
                ));
            }
        }
        ops
    }

    pub fn name() -> &'static str {
        "OperationInferrer"
    }
    pub fn tier() -> u8 {
        3
    }
}

pub struct SignatureGenerator;

impl SignatureGenerator {
    pub fn generate(op: &EntityOperation, entity_name: &str) -> FunctionBlueprint {
        let mut fb = FunctionBlueprint::new(&op.name);
        fb.is_async = op.is_async;
        fb.is_pub = true;

        match op.operation_type {
            OperationType::Create => {
                fb.parameters.push(FunctionParam {
                    name: "input".into(),
                    param_type: format!("Create{}Input", entity_name),
                    is_reference: true,
                    is_mutable: false,
                });
                fb.return_type = Some(format!("ForgeResult<{}>", entity_name));
            }
            OperationType::Read => {
                fb.parameters.push(FunctionParam {
                    name: "id".into(),
                    param_type: "Uuid".into(),
                    is_reference: true,
                    is_mutable: false,
                });
                fb.return_type = Some(format!("ForgeResult<{}>", entity_name));
            }
            OperationType::Update => {
                fb.parameters.push(FunctionParam {
                    name: "id".into(),
                    param_type: "Uuid".into(),
                    is_reference: true,
                    is_mutable: false,
                });
                fb.parameters.push(FunctionParam {
                    name: "input".into(),
                    param_type: format!("Update{}Input", entity_name),
                    is_reference: true,
                    is_mutable: false,
                });
                fb.return_type = Some(format!("ForgeResult<{}>", entity_name));
            }
            OperationType::Delete => {
                fb.parameters.push(FunctionParam {
                    name: "id".into(),
                    param_type: "Uuid".into(),
                    is_reference: true,
                    is_mutable: false,
                });
                fb.return_type = Some("ForgeResult<()>".into());
            }
            OperationType::Query => {
                fb.parameters.push(FunctionParam {
                    name: "query".into(),
                    param_type: format!("{}Query", entity_name),
                    is_reference: true,
                    is_mutable: false,
                });
                fb.return_type = Some(format!("ForgeResult<Vec<{}>>", entity_name));
            }
            _ => {
                fb.return_type = Some("ForgeResult<()>".into());
            }
        }

        fb.error_handling = ErrorHandling::Result;
        fb
    }

    pub fn name() -> &'static str {
        "SignatureGenerator"
    }
    pub fn tier() -> u8 {
        3
    }
}

pub struct ErrorFlowDesigner;

impl ErrorFlowDesigner {
    pub fn design_error_types(entities: &[Entity]) -> Vec<TypeDefinition> {
        let mut types = Vec::new();
        let mut error_td = TypeDefinition::new("ForgeError", TypeKind::Enum);
        error_td.derives = vec!["Error".into(), "Debug".into()];

        for entity in entities {
            error_td.fields.push(TypeField {
                name: format!("{}NotFound", entity.name),
                field_type: "String".into(),
                visibility: Visibility::Public,
                doc_comment: format!("{} not found", entity.name),
            });
            error_td.fields.push(TypeField {
                name: format!("Invalid{}", entity.name),
                field_type: "String".into(),
                visibility: Visibility::Public,
                doc_comment: format!("Invalid {} data", entity.name),
            });
            error_td.fields.push(TypeField {
                name: format!("Duplicate{}", entity.name),
                field_type: "String".into(),
                visibility: Visibility::Public,
                doc_comment: format!("Duplicate {}", entity.name),
            });
        }

        error_td.fields.push(TypeField {
            name: "Internal".into(),
            field_type: "String".into(),
            visibility: Visibility::Public,
            doc_comment: "Internal error".into(),
        });
        error_td.fields.push(TypeField {
            name: "Io".into(),
            field_type: "std::io::Error".into(),
            visibility: Visibility::Public,
            doc_comment: "IO error".into(),
        });

        types.push(error_td);
        types
    }

    pub fn name() -> &'static str {
        "ErrorFlowDesigner"
    }
    pub fn tier() -> u8 {
        3
    }
}

pub struct AsyncAnalyzer;

impl AsyncAnalyzer {
    pub fn should_be_async(op: &EntityOperation, domain: Domain) -> bool {
        if matches!(domain, Domain::Web | Domain::Api | Domain::Service) {
            return true;
        }
        if op.is_async {
            return true;
        }
        matches!(
            op.operation_type,
            OperationType::Create
                | OperationType::Update
                | OperationType::Delete
                | OperationType::Query
        )
    }

    pub fn analyze_concurrency_needs(entities: &[Entity], domain: Domain) -> ConcurrencyAnalysis {
        let needs_mutex = entities.iter().any(|e| e.is_aggregate_root);
        let needs_rwlock = entities.len() > 3;
        ConcurrencyAnalysis {
            needs_async: matches!(domain, Domain::Web | Domain::Api | Domain::Service),
            needs_mutex,
            needs_rwlock,
            recommended_runtime: if matches!(domain, Domain::Web | Domain::Api) {
                "tokio".into()
            } else {
                "none".into()
            },
            thread_safety: if needs_mutex || needs_rwlock {
                "Arc<RwLock<T>>".into()
            } else {
                "single-threaded".into()
            },
        }
    }

    pub fn name() -> &'static str {
        "AsyncAnalyzer"
    }
    pub fn tier() -> u8 {
        3
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConcurrencyAnalysis {
    pub needs_async: bool,
    pub needs_mutex: bool,
    pub needs_rwlock: bool,
    pub recommended_runtime: String,
    pub thread_safety: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_inferrer() {
        let entity = Entity::new("User", "A user");
        let ops = OperationInferrer::infer_operations(&entity);
        assert!(ops.len() >= 5);
        assert!(ops.iter().any(|o| o.name == "create_user"));
        assert!(ops.iter().any(|o| o.name == "get_user"));
        assert!(ops.iter().any(|o| o.name == "delete_user"));
    }

    #[test]
    fn test_operation_inferrer_with_fields() {
        let mut entity = Entity::new("User", "A user");
        entity
            .fields
            .push(EntityField::new("email", FieldType::String));
        let ops = OperationInferrer::infer_operations(&entity);
        assert!(ops.iter().any(|o| o.name == "find_user_by_email"));
    }

    #[test]
    fn test_signature_generator_create() {
        let op = EntityOperation::new("create_user", OperationType::Create);
        let sig = SignatureGenerator::generate(&op, "User");
        assert_eq!(sig.name, "create_user");
        assert!(sig.return_type.as_deref().unwrap().contains("User"));
    }

    #[test]
    fn test_signature_generator_read() {
        let op = EntityOperation::new("get_user", OperationType::Read);
        let sig = SignatureGenerator::generate(&op, "User");
        assert!(sig.parameters.iter().any(|p| p.name == "id"));
    }

    #[test]
    fn test_signature_generator_delete() {
        let op = EntityOperation::new("delete_user", OperationType::Delete);
        let sig = SignatureGenerator::generate(&op, "User");
        assert!(sig.return_type.as_deref().unwrap().contains("()"));
    }

    #[test]
    fn test_error_flow_designer() {
        let entities = vec![Entity::new("User", "A user"), Entity::new("Post", "A post")];
        let types = ErrorFlowDesigner::design_error_types(&entities);
        assert_eq!(types.len(), 1);
        assert!(types[0].fields.iter().any(|f| f.name == "UserNotFound"));
        assert!(types[0].fields.iter().any(|f| f.name == "PostNotFound"));
    }

    #[test]
    fn test_async_analyzer_web() {
        let op = EntityOperation::new("create", OperationType::Create);
        assert!(AsyncAnalyzer::should_be_async(&op, Domain::Web));
    }

    #[test]
    fn test_async_analyzer_library() {
        let op = EntityOperation::new("create", OperationType::Create);
        assert!(AsyncAnalyzer::should_be_async(&op, Domain::Library));
    }

    #[test]
    fn test_concurrency_analysis() {
        let entities = vec![Entity::new("User", "A user")];
        let analysis = AsyncAnalyzer::analyze_concurrency_needs(&entities, Domain::Api);
        assert!(analysis.needs_async);
        assert_eq!(analysis.recommended_runtime, "tokio");
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(OperationInferrer::name(), "OperationInferrer");
        assert_eq!(OperationInferrer::tier(), 3);
        assert_eq!(SignatureGenerator::name(), "SignatureGenerator");
        assert_eq!(ErrorFlowDesigner::name(), "ErrorFlowDesigner");
        assert_eq!(AsyncAnalyzer::name(), "AsyncAnalyzer");
    }
}

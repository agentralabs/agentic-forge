//! Tier 6: Blueprint inventions.
//! SkeletonGenerator, TypeFirstMaterializer, ContractSpecifier, GenerationPlanner

use crate::types::blueprint::*;

pub struct SkeletonGenerator;

impl SkeletonGenerator {
    pub fn generate(entity: &Entity) -> String {
        let mut skeleton = String::new();
        skeleton.push_str(&format!("/// {}\n", entity.description));
        skeleton.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        skeleton.push_str(&format!("pub struct {} {{\n", entity.name));

        for field in &entity.fields {
            let type_str = field.field_type.name();
            if field.required {
                skeleton.push_str(&format!("    pub {}: {},\n", field.name, type_str));
            } else {
                skeleton.push_str(&format!("    pub {}: Option<{}>,\n", field.name, type_str));
            }
        }

        skeleton.push_str("}\n\n");

        skeleton.push_str(&format!("impl {} {{\n", entity.name));
        for op in &entity.operations {
            let async_kw = if op.is_async { "async " } else { "" };
            let ret = op.return_type.as_deref().unwrap_or("()");
            skeleton.push_str(&format!(
                "    pub {}fn {}(&self) -> Result<{}, ForgeError> {{\n",
                async_kw, op.name, ret
            ));
            skeleton.push_str("        todo!()\n");
            skeleton.push_str("    }\n\n");
        }
        skeleton.push_str("}\n");

        skeleton
    }

    pub fn name() -> &'static str {
        "SkeletonGenerator"
    }
    pub fn tier() -> u8 {
        6
    }
}

pub struct TypeFirstMaterializer;

impl TypeFirstMaterializer {
    pub fn materialize(entities: &[Entity]) -> Vec<TypeDefinition> {
        let mut types = Vec::new();

        for entity in entities {
            let mut td = TypeDefinition::new(&entity.name, TypeKind::Struct);
            td.derives = vec![
                "Debug".into(),
                "Clone".into(),
                "Serialize".into(),
                "Deserialize".into(),
            ];
            td.doc_comment = entity.description.clone();

            for field in &entity.fields {
                td.fields.push(TypeField {
                    name: field.name.clone(),
                    field_type: field.field_type.name(),
                    visibility: Visibility::Public,
                    doc_comment: field.description.clone(),
                });
            }
            types.push(td);

            // Create input type
            let mut create_td =
                TypeDefinition::new(format!("Create{}Input", entity.name), TypeKind::Struct);
            create_td.derives = vec!["Debug".into(), "Deserialize".into()];
            for field in &entity.fields {
                if field.name != "id" && field.name != "created_at" && field.name != "updated_at" {
                    create_td.fields.push(TypeField {
                        name: field.name.clone(),
                        field_type: if field.required {
                            field.field_type.name()
                        } else {
                            format!("Option<{}>", field.field_type.name())
                        },
                        visibility: Visibility::Public,
                        doc_comment: String::new(),
                    });
                }
            }
            types.push(create_td);

            // Update input type
            let mut update_td =
                TypeDefinition::new(format!("Update{}Input", entity.name), TypeKind::Struct);
            update_td.derives = vec!["Debug".into(), "Deserialize".into()];
            for field in &entity.fields {
                if field.name != "id" && field.name != "created_at" && field.name != "updated_at" {
                    update_td.fields.push(TypeField {
                        name: field.name.clone(),
                        field_type: format!("Option<{}>", field.field_type.name()),
                        visibility: Visibility::Public,
                        doc_comment: String::new(),
                    });
                }
            }
            types.push(update_td);
        }

        types
    }

    pub fn name() -> &'static str {
        "TypeFirstMaterializer"
    }
    pub fn tier() -> u8 {
        6
    }
}

pub struct ContractSpecifier;

impl ContractSpecifier {
    pub fn specify(entities: &[Entity]) -> Vec<ContractSpec> {
        let mut contracts = Vec::new();

        for entity in entities {
            contracts.push(ContractSpec {
                name: format!("{}Repository", entity.name),
                contract_type: ContractType::Trait,
                methods: vec![
                    format!(
                        "fn find_by_id(&self, id: Uuid) -> Result<Option<{}>, Error>",
                        entity.name
                    ),
                    format!(
                        "fn save(&self, entity: &{}) -> Result<(), Error>",
                        entity.name
                    ),
                    format!("fn delete(&self, id: Uuid) -> Result<(), Error>"),
                    format!("fn list(&self) -> Result<Vec<{}>, Error>", entity.name),
                ],
                invariants: vec![
                    format!("{} ID must be unique", entity.name),
                    format!("{} must pass validation before save", entity.name),
                ],
            });
        }

        contracts
    }

    pub fn name() -> &'static str {
        "ContractSpecifier"
    }
    pub fn tier() -> u8 {
        6
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractSpec {
    pub name: String,
    pub contract_type: ContractType,
    pub methods: Vec<String>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContractType {
    Trait,
    Interface,
    Protocol,
}

pub struct GenerationPlanner;

impl GenerationPlanner {
    pub fn plan(blueprint: &Blueprint) -> Vec<GenerationStep> {
        let mut steps = Vec::new();
        let mut order = 0;

        steps.push(GenerationStep {
            order: {
                order += 1;
                order
            },
            name: "types".into(),
            description: "Generate type definitions".into(),
            dependencies: vec![],
        });
        steps.push(GenerationStep {
            order: {
                order += 1;
                order
            },
            name: "error_types".into(),
            description: "Generate error types".into(),
            dependencies: vec!["types".into()],
        });
        steps.push(GenerationStep {
            order: {
                order += 1;
                order
            },
            name: "models".into(),
            description: "Generate entity models".into(),
            dependencies: vec!["types".into()],
        });
        steps.push(GenerationStep {
            order: {
                order += 1;
                order
            },
            name: "contracts".into(),
            description: "Generate trait contracts".into(),
            dependencies: vec!["models".into()],
        });
        steps.push(GenerationStep {
            order: {
                order += 1;
                order
            },
            name: "repositories".into(),
            description: "Generate repository implementations".into(),
            dependencies: vec!["contracts".into()],
        });

        if !blueprint
            .dependencies
            .iter()
            .any(|d| d.name == "axum" || d.name == "actix-web")
        {
            // No web framework
        } else {
            steps.push(GenerationStep {
                order: {
                    order += 1;
                    order
                },
                name: "handlers".into(),
                description: "Generate HTTP handlers".into(),
                dependencies: vec!["repositories".into()],
            });
            steps.push(GenerationStep {
                order: {
                    order += 1;
                    order
                },
                name: "routes".into(),
                description: "Generate route configuration".into(),
                dependencies: vec!["handlers".into()],
            });
        }

        steps.push(GenerationStep {
            order: {
                order += 1;
                order
            },
            name: "tests".into(),
            description: "Generate test files".into(),
            dependencies: vec!["models".into(), "repositories".into()],
        });
        steps.push(GenerationStep {
            order: {
                order += 1;
                order
            },
            name: "config".into(),
            description: "Generate configuration".into(),
            dependencies: vec![],
        });
        let _ = order;

        steps
    }

    pub fn name() -> &'static str {
        "GenerationPlanner"
    }
    pub fn tier() -> u8 {
        6
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationStep {
    pub order: usize,
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::intent::{Domain, FieldType, OperationType};

    #[test]
    fn test_skeleton_generator() {
        let mut entity = Entity::new("User", "A user entity");
        entity
            .fields
            .push(EntityField::new("name", FieldType::String));
        entity
            .fields
            .push(EntityField::new("email", FieldType::String));
        let skeleton = SkeletonGenerator::generate(&entity);
        assert!(skeleton.contains("pub struct User"));
        assert!(skeleton.contains("pub name: String"));
    }

    #[test]
    fn test_skeleton_with_operations() {
        let mut entity = Entity::new("User", "A user entity");
        let op = EntityOperation::new("create", OperationType::Create);
        entity.operations.push(op);
        let skeleton = SkeletonGenerator::generate(&entity);
        assert!(skeleton.contains("fn create"));
        assert!(skeleton.contains("todo!()"));
    }

    #[test]
    fn test_type_first_materializer() {
        let mut entity = Entity::new("User", "A user");
        entity.fields.push(EntityField::new("id", FieldType::Uuid));
        entity
            .fields
            .push(EntityField::new("name", FieldType::String));
        entity
            .fields
            .push(EntityField::new("created_at", FieldType::DateTime));
        let types = TypeFirstMaterializer::materialize(&[entity]);
        assert_eq!(types.len(), 3); // User, CreateUserInput, UpdateUserInput
        assert_eq!(types[0].name, "User");
        assert_eq!(types[1].name, "CreateUserInput");
        assert_eq!(types[2].name, "UpdateUserInput");
    }

    #[test]
    fn test_contract_specifier() {
        let entity = Entity::new("User", "A user");
        let contracts = ContractSpecifier::specify(&[entity]);
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].name, "UserRepository");
        assert!(contracts[0].methods.len() >= 4);
    }

    #[test]
    fn test_generation_planner() {
        let bp = Blueprint::new("Test", "Test", Domain::Api);
        let steps = GenerationPlanner::plan(&bp);
        assert!(!steps.is_empty());
        assert_eq!(steps[0].name, "types");
        assert!(steps.iter().any(|s| s.name == "tests"));
    }

    #[test]
    fn test_generation_planner_with_web() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Api);
        bp.dependencies.push(Dependency::new("axum", "0.7"));
        let steps = GenerationPlanner::plan(&bp);
        assert!(steps.iter().any(|s| s.name == "handlers"));
        assert!(steps.iter().any(|s| s.name == "routes"));
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(SkeletonGenerator::name(), "SkeletonGenerator");
        assert_eq!(TypeFirstMaterializer::name(), "TypeFirstMaterializer");
        assert_eq!(ContractSpecifier::name(), "ContractSpecifier");
        assert_eq!(GenerationPlanner::name(), "GenerationPlanner");
    }
}

//! Tier 2: Entity inventions.
//! EntityInferrer, RelationshipMapper, FieldDeriver, ValidationRuleGenerator

use crate::types::blueprint::*;
use crate::types::intent::*;

pub struct EntityInferrer;

impl EntityInferrer {
    pub fn infer(description: &str) -> Vec<EntitySpec> {
        let mut entities = Vec::new();
        let words: Vec<&str> = description.split_whitespace().collect();
        let nouns = [
            "user",
            "account",
            "post",
            "comment",
            "product",
            "order",
            "item",
            "category",
            "tag",
            "role",
            "permission",
            "session",
            "token",
            "file",
            "image",
            "message",
            "notification",
            "event",
            "log",
            "config",
            "setting",
            "profile",
            "address",
            "payment",
        ];

        for noun in &nouns {
            if words.iter().any(|w| w.to_lowercase().contains(noun)) {
                let name = capitalize(noun);
                entities.push(EntitySpec::new(&name, format!("Inferred entity: {}", name)));
            }
        }

        if entities.is_empty() {
            entities.push(EntitySpec::new("Resource", "Default inferred entity"));
        }

        entities
    }

    pub fn name() -> &'static str {
        "EntityInferrer"
    }
    pub fn tier() -> u8 {
        2
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

pub struct RelationshipMapper;

impl RelationshipMapper {
    pub fn map_relationships(entities: &[Entity]) -> Vec<(String, Relationship)> {
        let mut relationships = Vec::new();
        let entity_names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();

        for entity in entities {
            for field in &entity.fields {
                if let FieldType::Reference(ref target) = field.field_type {
                    if entity_names.contains(&target.as_str()) {
                        relationships.push((
                            entity.name.clone(),
                            Relationship {
                                target_entity: target.clone(),
                                relationship_type: RelationshipType::BelongsTo,
                                cardinality: Cardinality::ManyToOne,
                                description: format!("{} belongs to {}", entity.name, target),
                            },
                        ));
                    }
                }
                if let FieldType::Array(ref inner) = field.field_type {
                    if let FieldType::Reference(ref target) = **inner {
                        if entity_names.contains(&target.as_str()) {
                            relationships.push((
                                entity.name.clone(),
                                Relationship {
                                    target_entity: target.clone(),
                                    relationship_type: RelationshipType::HasMany,
                                    cardinality: Cardinality::OneToMany,
                                    description: format!("{} has many {}", entity.name, target),
                                },
                            ));
                        }
                    }
                }
            }
        }

        relationships
    }

    pub fn name() -> &'static str {
        "RelationshipMapper"
    }
    pub fn tier() -> u8 {
        2
    }
}

pub struct FieldDeriver;

impl FieldDeriver {
    pub fn derive_fields(entity_name: &str, domain: Domain) -> Vec<EntityField> {
        let mut fields = vec![
            EntityField::new("id", FieldType::Uuid),
            EntityField {
                name: "created_at".into(),
                field_type: FieldType::DateTime,
                required: true,
                default_value: Some("now()".into()),
                description: "Creation timestamp".into(),
            },
            EntityField {
                name: "updated_at".into(),
                field_type: FieldType::DateTime,
                required: true,
                default_value: Some("now()".into()),
                description: "Last update timestamp".into(),
            },
        ];

        let name_lower = entity_name.to_lowercase();
        match name_lower.as_str() {
            "user" | "account" | "profile" => {
                fields.push(EntityField::new("name", FieldType::String));
                fields.push(EntityField::new("email", FieldType::String));
                if matches!(domain, Domain::Web | Domain::Api) {
                    fields.push(EntityField::new("password_hash", FieldType::String));
                }
            }
            "post" | "article" | "comment" => {
                fields.push(EntityField::new("title", FieldType::String));
                fields.push(EntityField::new("content", FieldType::String));
                fields.push(EntityField::new("author_id", FieldType::Uuid));
            }
            "product" | "item" => {
                fields.push(EntityField::new("name", FieldType::String));
                fields.push(EntityField::new("description", FieldType::String));
                fields.push(EntityField::new("price", FieldType::Float));
            }
            "order" => {
                fields.push(EntityField::new("status", FieldType::String));
                fields.push(EntityField::new("total", FieldType::Float));
                fields.push(EntityField::new("user_id", FieldType::Uuid));
            }
            _ => {
                fields.push(EntityField::new("name", FieldType::String));
            }
        }

        fields
    }

    pub fn name() -> &'static str {
        "FieldDeriver"
    }
    pub fn tier() -> u8 {
        2
    }
}

pub struct ValidationRuleGenerator;

impl ValidationRuleGenerator {
    pub fn generate(entity: &Entity) -> Vec<ValidationRule> {
        let mut rules = Vec::new();

        for field in &entity.fields {
            if field.required {
                rules.push(ValidationRule {
                    field: field.name.clone(),
                    rule_type: "required".into(),
                    parameters: std::collections::HashMap::new(),
                    message: format!("{} is required", field.name),
                });
            }

            match &field.field_type {
                FieldType::String => {
                    if field.name.contains("email") {
                        rules.push(ValidationRule {
                            field: field.name.clone(),
                            rule_type: "email_format".into(),
                            parameters: std::collections::HashMap::new(),
                            message: format!("{} must be a valid email", field.name),
                        });
                    }
                    let mut params = std::collections::HashMap::new();
                    params.insert("max".into(), "10000".into());
                    rules.push(ValidationRule {
                        field: field.name.clone(),
                        rule_type: "max_length".into(),
                        parameters: params,
                        message: format!("{} exceeds maximum length", field.name),
                    });
                }
                FieldType::Float | FieldType::Integer => {
                    if field.name.contains("price") || field.name.contains("total") {
                        let mut params = std::collections::HashMap::new();
                        params.insert("min".into(), "0".into());
                        rules.push(ValidationRule {
                            field: field.name.clone(),
                            rule_type: "min_value".into(),
                            parameters: params,
                            message: format!("{} must be non-negative", field.name),
                        });
                    }
                }
                _ => {}
            }
        }

        rules
    }

    pub fn name() -> &'static str {
        "ValidationRuleGenerator"
    }
    pub fn tier() -> u8 {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_inferrer_users() {
        let entities = EntityInferrer::infer("A system with users and posts");
        assert!(entities.iter().any(|e| e.name == "User"));
        assert!(entities.iter().any(|e| e.name == "Post"));
    }

    #[test]
    fn test_entity_inferrer_empty() {
        let entities = EntityInferrer::infer("Nothing relevant here xyz");
        assert!(!entities.is_empty()); // Falls back to default
    }

    #[test]
    fn test_entity_inferrer_products() {
        let entities = EntityInferrer::infer("E-commerce with products and orders");
        assert!(entities.iter().any(|e| e.name == "Product"));
        assert!(entities.iter().any(|e| e.name == "Order"));
    }

    #[test]
    fn test_relationship_mapper() {
        let mut user = Entity::new("User", "A user");
        user.fields.push(EntityField::new("id", FieldType::Uuid));
        let mut post = Entity::new("Post", "A post");
        post.fields.push(EntityField::new(
            "author_id",
            FieldType::Reference("User".into()),
        ));
        let rels = RelationshipMapper::map_relationships(&[user, post]);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].1.target_entity, "User");
    }

    #[test]
    fn test_relationship_mapper_has_many() {
        let mut user = Entity::new("User", "A user");
        user.fields.push(EntityField::new(
            "posts",
            FieldType::Array(Box::new(FieldType::Reference("Post".into()))),
        ));
        let post = Entity::new("Post", "A post");
        let rels = RelationshipMapper::map_relationships(&[user, post]);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].1.relationship_type, RelationshipType::HasMany);
    }

    #[test]
    fn test_field_deriver_user() {
        let fields = FieldDeriver::derive_fields("User", Domain::Api);
        assert!(fields.iter().any(|f| f.name == "email"));
        assert!(fields.iter().any(|f| f.name == "password_hash"));
        assert!(fields.iter().any(|f| f.name == "id"));
    }

    #[test]
    fn test_field_deriver_post() {
        let fields = FieldDeriver::derive_fields("Post", Domain::Web);
        assert!(fields.iter().any(|f| f.name == "title"));
        assert!(fields.iter().any(|f| f.name == "content"));
    }

    #[test]
    fn test_field_deriver_product() {
        let fields = FieldDeriver::derive_fields("Product", Domain::Web);
        assert!(fields.iter().any(|f| f.name == "price"));
    }

    #[test]
    fn test_field_deriver_unknown() {
        let fields = FieldDeriver::derive_fields("Widget", Domain::Library);
        assert!(fields.iter().any(|f| f.name == "name"));
    }

    #[test]
    fn test_validation_rule_generator() {
        let mut entity = Entity::new("User", "A user");
        entity
            .fields
            .push(EntityField::new("email", FieldType::String));
        entity
            .fields
            .push(EntityField::new("name", FieldType::String));
        let rules = ValidationRuleGenerator::generate(&entity);
        assert!(!rules.is_empty());
        assert!(rules.iter().any(|r| r.rule_type == "email_format"));
    }

    #[test]
    fn test_validation_rule_price() {
        let mut entity = Entity::new("Product", "A product");
        entity
            .fields
            .push(EntityField::new("price", FieldType::Float));
        let rules = ValidationRuleGenerator::generate(&entity);
        assert!(rules.iter().any(|r| r.rule_type == "min_value"));
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(EntityInferrer::name(), "EntityInferrer");
        assert_eq!(EntityInferrer::tier(), 2);
        assert_eq!(RelationshipMapper::name(), "RelationshipMapper");
        assert_eq!(FieldDeriver::name(), "FieldDeriver");
        assert_eq!(ValidationRuleGenerator::name(), "ValidationRuleGenerator");
    }
}

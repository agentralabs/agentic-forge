//! BlueprintValidator — comprehensive blueprint validation.

use crate::types::blueprint::*;
use crate::types::{ForgeResult, MAX_DEPENDENCIES, MAX_ENTITIES, MAX_FILES};
use std::collections::HashSet;

pub struct BlueprintValidator;

impl BlueprintValidator {
    pub fn validate(blueprint: &Blueprint) -> ForgeResult<ValidationReport> {
        let mut report = ValidationReport::new();

        Self::validate_structure(blueprint, &mut report);
        Self::validate_entities(blueprint, &mut report);
        Self::validate_files(blueprint, &mut report);
        Self::validate_dependencies(blueprint, &mut report);
        Self::validate_tests(blueprint, &mut report);
        Self::validate_wiring(blueprint, &mut report);

        Ok(report)
    }

    fn validate_structure(bp: &Blueprint, report: &mut ValidationReport) {
        if bp.name.is_empty() {
            report.add_error("Blueprint name is empty");
        }
        if bp.name.len() > 256 {
            report.add_error("Blueprint name exceeds 256 characters");
        }
        if bp.description.is_empty() {
            report.add_warning("Blueprint description is empty");
        }
        if bp.entities.len() > MAX_ENTITIES {
            report.add_error(&format!(
                "Entity count {} exceeds maximum {}",
                bp.entities.len(),
                MAX_ENTITIES
            ));
        }
        if bp.files.len() > MAX_FILES {
            report.add_error(&format!(
                "File count {} exceeds maximum {}",
                bp.files.len(),
                MAX_FILES
            ));
        }
        if bp.dependencies.len() > MAX_DEPENDENCIES {
            report.add_error(&format!(
                "Dependency count {} exceeds maximum {}",
                bp.dependencies.len(),
                MAX_DEPENDENCIES
            ));
        }
        if bp.entities.is_empty() {
            report.add_warning("Blueprint has no entities");
        }
        if bp.files.is_empty() {
            report.add_warning("Blueprint has no files");
        }
    }

    fn validate_entities(bp: &Blueprint, report: &mut ValidationReport) {
        let mut names = HashSet::new();
        for entity in &bp.entities {
            if entity.name.is_empty() {
                report.add_error("Entity has empty name");
            }
            if !names.insert(&entity.name) {
                report.add_error(&format!("Duplicate entity name: {}", entity.name));
            }
            if entity.fields.is_empty() {
                report.add_warning(&format!("Entity '{}' has no fields", entity.name));
            }

            let mut field_names = HashSet::new();
            for field in &entity.fields {
                if field.name.is_empty() {
                    report.add_error(&format!(
                        "Entity '{}' has field with empty name",
                        entity.name
                    ));
                }
                if !field_names.insert(&field.name) {
                    report.add_error(&format!(
                        "Entity '{}' has duplicate field: {}",
                        entity.name, field.name
                    ));
                }
            }

            for rel in &entity.relationships {
                if rel.target_entity.is_empty() {
                    report.add_error(&format!(
                        "Entity '{}' has relationship with empty target",
                        entity.name
                    ));
                }
                if !bp.entities.iter().any(|e| e.name == rel.target_entity) {
                    report.add_warning(&format!(
                        "Entity '{}' references unknown entity '{}'",
                        entity.name, rel.target_entity
                    ));
                }
            }
        }
    }

    fn validate_files(bp: &Blueprint, report: &mut ValidationReport) {
        let mut paths = HashSet::new();
        for file in &bp.files {
            if file.path.is_empty() {
                report.add_error("File has empty path");
            }
            if !paths.insert(&file.path) {
                report.add_error(&format!("Duplicate file path: {}", file.path));
            }
        }
    }

    fn validate_dependencies(bp: &Blueprint, report: &mut ValidationReport) {
        let mut names = HashSet::new();
        for dep in &bp.dependencies {
            if dep.name.is_empty() {
                report.add_error("Dependency has empty name");
            }
            if dep.version.is_empty() {
                report.add_warning(&format!("Dependency '{}' has empty version", dep.name));
            }
            if !names.insert(&dep.name) {
                report.add_error(&format!("Duplicate dependency: {}", dep.name));
            }
        }
    }

    fn validate_tests(bp: &Blueprint, report: &mut ValidationReport) {
        if bp.test_cases.is_empty() && !bp.entities.is_empty() {
            report.add_warning("Blueprint has entities but no test cases");
        }
        for tc in &bp.test_cases {
            if tc.name.is_empty() {
                report.add_error("Test case has empty name");
            }
            if tc.target.is_empty() {
                report.add_warning(&format!("Test case '{}' has empty target", tc.name));
            }
        }
    }

    fn validate_wiring(bp: &Blueprint, report: &mut ValidationReport) {
        for w in &bp.wiring {
            if w.source.is_empty() || w.target.is_empty() {
                report.add_error("Component wiring has empty source or target");
            }
        }
        for flow in &bp.data_flows {
            if flow.source.is_empty() || flow.target.is_empty() {
                report.add_error("Data flow has empty source or target");
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub is_valid: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }

    pub fn add_error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, msg: &str) {
        self.warnings.push(msg.to_string());
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::intent::Domain;

    #[test]
    fn test_validate_empty_blueprint() {
        let bp = Blueprint::new("Test", "A test", Domain::Api);
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(!report.is_valid || report.warning_count() > 0);
    }

    #[test]
    fn test_validate_empty_name() {
        let bp = Blueprint::new("", "A test", Domain::Api);
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("name is empty")));
    }

    #[test]
    fn test_validate_duplicate_entities() {
        let mut bp = Blueprint::new("Test", "A test", Domain::Api);
        bp.entities.push(Entity::new("User", "A"));
        bp.entities.push(Entity::new("User", "B"));
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("Duplicate entity")));
    }

    #[test]
    fn test_validate_duplicate_files() {
        let mut bp = Blueprint::new("Test", "A test", Domain::Api);
        bp.files
            .push(FileBlueprint::new("src/main.rs", FileType::Source));
        bp.files
            .push(FileBlueprint::new("src/main.rs", FileType::Source));
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(!report.is_valid);
    }

    #[test]
    fn test_validate_duplicate_dependencies() {
        let mut bp = Blueprint::new("Test", "A test", Domain::Api);
        bp.dependencies.push(Dependency::new("serde", "1.0"));
        bp.dependencies.push(Dependency::new("serde", "2.0"));
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(!report.is_valid);
    }

    #[test]
    fn test_validate_missing_relationship_target() {
        let mut bp = Blueprint::new("Test", "A test", Domain::Api);
        let mut entity = Entity::new("User", "A");
        entity.relationships.push(Relationship {
            target_entity: "Nonexistent".into(),
            relationship_type: RelationshipType::HasMany,
            cardinality: Cardinality::OneToMany,
            description: "".into(),
        });
        bp.entities.push(entity);
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(report.warnings.iter().any(|w| w.contains("unknown entity")));
    }

    #[test]
    fn test_validate_no_tests_warning() {
        let mut bp = Blueprint::new("Test", "A test", Domain::Api);
        bp.entities.push(Entity::new("User", "A"));
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(report.warnings.iter().any(|w| w.contains("no test cases")));
    }

    #[test]
    fn test_validation_report_counts() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid);
        report.add_error("error 1");
        report.add_warning("warning 1");
        assert!(!report.is_valid);
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 1);
        assert_eq!(report.total_issues(), 2);
    }

    #[test]
    fn test_validate_entity_empty_field_name() {
        let mut bp = Blueprint::new("Test", "A test", Domain::Api);
        let mut entity = Entity::new("User", "A");
        entity.fields.push(EntityField::new(
            "",
            crate::types::intent::FieldType::String,
        ));
        bp.entities.push(entity);
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(report.errors.iter().any(|e| e.contains("empty name")));
    }

    #[test]
    fn test_validate_wiring_empty_source() {
        let mut bp = Blueprint::new("Test", "A test", Domain::Api);
        bp.wiring.push(ComponentWiring {
            source: "".into(),
            target: "B".into(),
            wiring_type: WiringType::DirectCall,
            description: "".into(),
        });
        let report = BlueprintValidator::validate(&bp).unwrap();
        assert!(report.errors.iter().any(|e| e.contains("empty source")));
    }
}

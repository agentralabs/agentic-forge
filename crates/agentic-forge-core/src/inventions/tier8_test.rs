//! Tier 8: Test inventions.
//! TestCaseGenerator, TestFixtureDesigner, IntegrationTestPlanner, MockSpecifier

use crate::types::blueprint::*;

pub struct TestCaseGenerator;

impl TestCaseGenerator {
    pub fn generate(entity: &Entity) -> Vec<TestCase> {
        let mut tests = Vec::new();
        let name = &entity.name;
        let lower = name.to_lowercase();

        tests.push(TestCase::new(&format!("test_create_{}", lower), TestType::Unit, &format!("{}::create", name)));
        tests.push(TestCase::new(&format!("test_read_{}", lower), TestType::Unit, &format!("{}::read", name)));
        tests.push(TestCase::new(&format!("test_update_{}", lower), TestType::Unit, &format!("{}::update", name)));
        tests.push(TestCase::new(&format!("test_delete_{}", lower), TestType::Unit, &format!("{}::delete", name)));
        tests.push(TestCase::new(&format!("test_list_{}s", lower), TestType::Unit, &format!("{}::list", name)));

        for field in &entity.fields {
            if field.required {
                tests.push(TestCase::new(
                    &format!("test_{}_requires_{}", lower, field.name),
                    TestType::Unit,
                    &format!("{}::validation", name),
                ));
            }
        }

        tests.push(TestCase::new(&format!("test_{}_not_found", lower), TestType::Unit, &format!("{}::error", name)));
        tests.push(TestCase::new(&format!("test_{}_serialization", lower), TestType::Unit, &format!("{}::serde", name)));

        tests
    }

    pub fn name() -> &'static str { "TestCaseGenerator" }
    pub fn tier() -> u8 { 8 }
}

pub struct TestFixtureDesigner;

impl TestFixtureDesigner {
    pub fn design(entity: &Entity) -> Vec<TestFixture> {
        let name = &entity.name;
        let lower = name.to_lowercase();
        let mut fixtures = Vec::new();

        fixtures.push(TestFixture {
            name: format!("make_{}", lower),
            fixture_type: FixtureType::Builder,
            description: format!("Create a test {} with defaults", lower),
            setup_code: format!("{}::new(/* test defaults */)", name),
        });

        fixtures.push(TestFixture {
            name: format!("make_{}_list", lower),
            fixture_type: FixtureType::Collection,
            description: format!("Create a list of test {}s", lower),
            setup_code: format!("(0..10).map(|_| make_{}()).collect()", lower),
        });

        if entity.relationships.iter().any(|r| matches!(r.relationship_type, RelationshipType::HasMany)) {
            fixtures.push(TestFixture {
                name: format!("make_{}_with_children", lower),
                fixture_type: FixtureType::Graph,
                description: format!("Create a {} with related entities", lower),
                setup_code: format!("make_{}() + add_children()", lower),
            });
        }

        fixtures
    }

    pub fn name() -> &'static str { "TestFixtureDesigner" }
    pub fn tier() -> u8 { 8 }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestFixture {
    pub name: String,
    pub fixture_type: FixtureType,
    pub description: String,
    pub setup_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FixtureType {
    Builder,
    Collection,
    Graph,
    Mock,
}

pub struct IntegrationTestPlanner;

impl IntegrationTestPlanner {
    pub fn plan(entities: &[Entity]) -> Vec<IntegrationTestPlan> {
        let mut plans = Vec::new();

        for entity in entities {
            plans.push(IntegrationTestPlan {
                name: format!("{}_crud_flow", entity.name.to_lowercase()),
                description: format!("Full CRUD lifecycle for {}", entity.name),
                steps: vec![
                    format!("Create {} with valid data", entity.name),
                    format!("Read {} by ID", entity.name),
                    format!("Update {} fields", entity.name),
                    format!("Delete {}", entity.name),
                    format!("Verify {} is deleted", entity.name),
                ],
                entities_involved: vec![entity.name.clone()],
            });
        }

        // Cross-entity tests
        for rel in entities.iter().flat_map(|e| e.relationships.iter().map(move |r| (e, r))) {
            let (entity, relationship) = rel;
            plans.push(IntegrationTestPlan {
                name: format!("{}_{}_relationship", entity.name.to_lowercase(), relationship.target_entity.to_lowercase()),
                description: format!("{} to {} relationship test", entity.name, relationship.target_entity),
                steps: vec![
                    format!("Create {}", relationship.target_entity),
                    format!("Create {} linked to {}", entity.name, relationship.target_entity),
                    format!("Verify relationship exists"),
                    format!("Delete {} and verify cascade", relationship.target_entity),
                ],
                entities_involved: vec![entity.name.clone(), relationship.target_entity.clone()],
            });
        }

        plans
    }

    pub fn name() -> &'static str { "IntegrationTestPlanner" }
    pub fn tier() -> u8 { 8 }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegrationTestPlan {
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
    pub entities_involved: Vec<String>,
}

pub struct MockSpecifier;

impl MockSpecifier {
    pub fn specify(entities: &[Entity]) -> Vec<MockSpec> {
        let mut mocks = Vec::new();

        for entity in entities {
            mocks.push(MockSpec {
                name: format!("Mock{}Repository", entity.name),
                target_trait: format!("{}Repository", entity.name),
                methods: vec![
                    MockMethod { name: "find_by_id".into(), return_behavior: format!("Returns Some({}) or None", entity.name) },
                    MockMethod { name: "save".into(), return_behavior: "Returns Ok(())".into() },
                    MockMethod { name: "delete".into(), return_behavior: "Returns Ok(())".into() },
                    MockMethod { name: "list".into(), return_behavior: format!("Returns Vec<{}>", entity.name) },
                ],
            });
        }

        mocks
    }

    pub fn name() -> &'static str { "MockSpecifier" }
    pub fn tier() -> u8 { 8 }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MockSpec {
    pub name: String,
    pub target_trait: String,
    pub methods: Vec<MockMethod>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MockMethod {
    pub name: String,
    pub return_behavior: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::intent::FieldType;

    #[test]
    fn test_test_case_generator() {
        let entity = Entity::new("User", "A user");
        let tests = TestCaseGenerator::generate(&entity);
        assert!(tests.len() >= 7);
        assert!(tests.iter().any(|t| t.name == "test_create_user"));
        assert!(tests.iter().any(|t| t.name == "test_user_not_found"));
    }

    #[test]
    fn test_test_case_generator_with_fields() {
        let mut entity = Entity::new("User", "A user");
        entity.fields.push(EntityField::new("email", FieldType::String));
        let tests = TestCaseGenerator::generate(&entity);
        assert!(tests.iter().any(|t| t.name == "test_user_requires_email"));
    }

    #[test]
    fn test_fixture_designer() {
        let entity = Entity::new("User", "A user");
        let fixtures = TestFixtureDesigner::design(&entity);
        assert!(fixtures.len() >= 2);
        assert!(fixtures.iter().any(|f| f.name == "make_user"));
    }

    #[test]
    fn test_fixture_designer_with_relationships() {
        let mut entity = Entity::new("User", "A user");
        entity.relationships.push(Relationship {
            target_entity: "Post".into(),
            relationship_type: RelationshipType::HasMany,
            cardinality: Cardinality::OneToMany,
            description: "".into(),
        });
        let fixtures = TestFixtureDesigner::design(&entity);
        assert!(fixtures.iter().any(|f| f.fixture_type == FixtureType::Graph));
    }

    #[test]
    fn test_integration_test_planner() {
        let entities = vec![Entity::new("User", "A user"), Entity::new("Post", "A post")];
        let plans = IntegrationTestPlanner::plan(&entities);
        assert!(plans.len() >= 2);
        assert!(plans.iter().any(|p| p.name == "user_crud_flow"));
    }

    #[test]
    fn test_integration_test_planner_with_relationships() {
        let mut user = Entity::new("User", "A user");
        user.relationships.push(Relationship {
            target_entity: "Post".into(),
            relationship_type: RelationshipType::HasMany,
            cardinality: Cardinality::OneToMany,
            description: "".into(),
        });
        let plans = IntegrationTestPlanner::plan(&[user]);
        assert!(plans.iter().any(|p| p.name.contains("relationship")));
    }

    #[test]
    fn test_mock_specifier() {
        let entities = vec![Entity::new("User", "A user")];
        let mocks = MockSpecifier::specify(&entities);
        assert_eq!(mocks.len(), 1);
        assert_eq!(mocks[0].name, "MockUserRepository");
        assert!(mocks[0].methods.len() >= 4);
    }

    #[test]
    fn test_mock_specifier_multiple() {
        let entities = vec![Entity::new("User", "A user"), Entity::new("Post", "A post")];
        let mocks = MockSpecifier::specify(&entities);
        assert_eq!(mocks.len(), 2);
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(TestCaseGenerator::name(), "TestCaseGenerator");
        assert_eq!(TestCaseGenerator::tier(), 8);
        assert_eq!(TestFixtureDesigner::name(), "TestFixtureDesigner");
        assert_eq!(IntegrationTestPlanner::name(), "IntegrationTestPlanner");
        assert_eq!(MockSpecifier::name(), "MockSpecifier");
    }
}

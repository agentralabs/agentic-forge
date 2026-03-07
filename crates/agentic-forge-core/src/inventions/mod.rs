//! 32 Inventions across 8 tiers.

pub mod tier1_decomposition;
pub mod tier2_entity;
pub mod tier3_operation;
pub mod tier4_structure;
pub mod tier5_dependency;
pub mod tier6_blueprint;
pub mod tier7_integration;
pub mod tier8_test;

pub use tier1_decomposition::*;
pub use tier2_entity::*;
pub use tier3_operation::*;
pub use tier4_structure::*;
pub use tier5_dependency::*;
pub use tier6_blueprint::*;
pub use tier7_integration::*;
pub use tier8_test::*;

pub const INVENTION_COUNT: usize = 32;

pub fn all_invention_names() -> Vec<&'static str> {
    vec![
        // Tier 1 - Decomposition
        "LayerDecomposer", "ConcernAnalyzer", "BoundaryInferrer", "CrossCuttingDetector",
        // Tier 2 - Entity
        "EntityInferrer", "RelationshipMapper", "FieldDeriver", "ValidationRuleGenerator",
        // Tier 3 - Operation
        "OperationInferrer", "SignatureGenerator", "ErrorFlowDesigner", "AsyncAnalyzer",
        // Tier 4 - Structure
        "FileStructureGenerator", "ImportGraphGenerator", "ModuleHierarchyBuilder", "ConfigDesigner",
        // Tier 5 - Dependency
        "DependencyInferrer", "VersionResolver", "ApiSpecExtractor", "ConflictResolver",
        // Tier 6 - Blueprint
        "SkeletonGenerator", "TypeFirstMaterializer", "ContractSpecifier", "GenerationPlanner",
        // Tier 7 - Integration
        "WiringDiagramBuilder", "DataFlowSpecifier", "InitSequencer", "ShutdownSequencer",
        // Tier 8 - Test
        "TestCaseGenerator", "TestFixtureDesigner", "IntegrationTestPlanner", "MockSpecifier",
    ]
}

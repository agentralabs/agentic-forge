//! Tier 1: Decomposition inventions.
//! LayerDecomposer, ConcernAnalyzer, BoundaryInferrer, CrossCuttingDetector

use crate::types::blueprint::*;
use crate::types::intent::*;

pub struct LayerDecomposer;

impl LayerDecomposer {
    pub fn decompose(domain: Domain) -> Vec<ArchitectureLayer> {
        match domain {
            Domain::Web | Domain::Api => vec![
                ArchitectureLayer {
                    name: "presentation".into(),
                    description: "HTTP handlers and routing".into(),
                    modules: vec!["handlers".into(), "routes".into()],
                    allowed_dependencies: vec!["application".into()],
                },
                ArchitectureLayer {
                    name: "application".into(),
                    description: "Business logic and use cases".into(),
                    modules: vec!["services".into(), "use_cases".into()],
                    allowed_dependencies: vec!["domain".into()],
                },
                ArchitectureLayer {
                    name: "domain".into(),
                    description: "Core domain models and logic".into(),
                    modules: vec!["models".into(), "entities".into()],
                    allowed_dependencies: vec![],
                },
                ArchitectureLayer {
                    name: "infrastructure".into(),
                    description: "External integrations".into(),
                    modules: vec!["repositories".into(), "clients".into()],
                    allowed_dependencies: vec!["domain".into()],
                },
            ],
            Domain::Cli => vec![
                ArchitectureLayer {
                    name: "cli".into(),
                    description: "Command-line interface".into(),
                    modules: vec!["commands".into(), "args".into()],
                    allowed_dependencies: vec!["core".into()],
                },
                ArchitectureLayer {
                    name: "core".into(),
                    description: "Core business logic".into(),
                    modules: vec!["engine".into(), "types".into()],
                    allowed_dependencies: vec![],
                },
            ],
            Domain::Library => vec![
                ArchitectureLayer {
                    name: "public_api".into(),
                    description: "Public API surface".into(),
                    modules: vec!["lib".into()],
                    allowed_dependencies: vec!["internal".into()],
                },
                ArchitectureLayer {
                    name: "internal".into(),
                    description: "Internal implementation".into(),
                    modules: vec!["engine".into(), "types".into()],
                    allowed_dependencies: vec![],
                },
            ],
            _ => vec![
                ArchitectureLayer {
                    name: "core".into(),
                    description: "Core module".into(),
                    modules: vec!["core".into()],
                    allowed_dependencies: vec![],
                },
                ArchitectureLayer {
                    name: "interface".into(),
                    description: "Interface layer".into(),
                    modules: vec!["interface".into()],
                    allowed_dependencies: vec!["core".into()],
                },
            ],
        }
    }

    pub fn name() -> &'static str {
        "LayerDecomposer"
    }
    pub fn tier() -> u8 {
        1
    }
}

pub struct ConcernAnalyzer;

impl ConcernAnalyzer {
    pub fn analyze(intent: &IntentSpec) -> Vec<CrossCuttingConcern> {
        let mut concerns = vec![
            CrossCuttingConcern {
                name: "error_handling".into(),
                concern_type: ConcernType::ErrorHandling,
                affected_layers: vec!["all".into()],
                implementation_strategy: "Result type with custom error enum".into(),
            },
            CrossCuttingConcern {
                name: "logging".into(),
                concern_type: ConcernType::Logging,
                affected_layers: vec!["all".into()],
                implementation_strategy: "tracing crate with structured logging".into(),
            },
        ];

        if matches!(intent.domain, Domain::Web | Domain::Api | Domain::Service) {
            concerns.push(CrossCuttingConcern {
                name: "authentication".into(),
                concern_type: ConcernType::Authentication,
                affected_layers: vec!["presentation".into(), "application".into()],
                implementation_strategy: "middleware-based auth".into(),
            });
            concerns.push(CrossCuttingConcern {
                name: "validation".into(),
                concern_type: ConcernType::Validation,
                affected_layers: vec!["presentation".into(), "domain".into()],
                implementation_strategy: "validation traits on domain types".into(),
            });
        }

        if intent
            .constraints
            .iter()
            .any(|c| matches!(c.constraint_type, ConstraintType::Performance(_)))
        {
            concerns.push(CrossCuttingConcern {
                name: "caching".into(),
                concern_type: ConcernType::Caching,
                affected_layers: vec!["application".into(), "infrastructure".into()],
                implementation_strategy: "in-memory cache with TTL".into(),
            });
        }

        concerns
    }

    pub fn name() -> &'static str {
        "ConcernAnalyzer"
    }
    pub fn tier() -> u8 {
        1
    }
}

pub struct BoundaryInferrer;

impl BoundaryInferrer {
    pub fn infer_boundaries(entities: &[EntitySpec]) -> Vec<ModuleBoundary> {
        let mut boundaries = Vec::new();
        let mut groups: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for entity in entities {
            let group = if entity.is_aggregate_root {
                entity.name.clone()
            } else {
                "shared".into()
            };
            groups.entry(group).or_default().push(entity.name.clone());
        }

        for (name, entities) in groups {
            boundaries.push(ModuleBoundary {
                name: name.to_lowercase(),
                entities,
                is_bounded_context: true,
            });
        }

        boundaries
    }

    pub fn name() -> &'static str {
        "BoundaryInferrer"
    }
    pub fn tier() -> u8 {
        1
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleBoundary {
    pub name: String,
    pub entities: Vec<String>,
    pub is_bounded_context: bool,
}

pub struct CrossCuttingDetector;

impl CrossCuttingDetector {
    pub fn detect(entities: &[EntitySpec]) -> Vec<String> {
        let mut cross_cutting = Vec::new();
        let mut ref_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for entity in entities {
            for field in &entity.fields {
                if let FieldType::Reference(ref r) = field.field_type {
                    *ref_counts.entry(r.clone()).or_insert(0) += 1;
                }
            }
        }

        for entity in entities {
            if ref_counts.get(&entity.name).copied().unwrap_or(0) >= 2 {
                cross_cutting.push(entity.name.clone());
            }
        }

        cross_cutting
    }

    pub fn name() -> &'static str {
        "CrossCuttingDetector"
    }
    pub fn tier() -> u8 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_decomposer_web() {
        let layers = LayerDecomposer::decompose(Domain::Web);
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].name, "presentation");
    }

    #[test]
    fn test_layer_decomposer_cli() {
        let layers = LayerDecomposer::decompose(Domain::Cli);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].name, "cli");
    }

    #[test]
    fn test_layer_decomposer_library() {
        let layers = LayerDecomposer::decompose(Domain::Library);
        assert_eq!(layers.len(), 2);
    }

    #[test]
    fn test_layer_decomposer_default() {
        let layers = LayerDecomposer::decompose(Domain::Embedded);
        assert_eq!(layers.len(), 2);
    }

    #[test]
    fn test_concern_analyzer_api() {
        let intent = IntentSpec::new("Test", "An API", Domain::Api);
        let concerns = ConcernAnalyzer::analyze(&intent);
        assert!(concerns.len() >= 4);
        assert!(concerns.iter().any(|c| c.name == "authentication"));
    }

    #[test]
    fn test_concern_analyzer_library() {
        let intent = IntentSpec::new("Test", "A library", Domain::Library);
        let concerns = ConcernAnalyzer::analyze(&intent);
        assert!(concerns.len() >= 2);
        assert!(!concerns.iter().any(|c| c.name == "authentication"));
    }

    #[test]
    fn test_concern_analyzer_with_performance() {
        let intent =
            IntentSpec::new("Test", "An API", Domain::Api).with_constraint(Constraint::new(
                "perf",
                ConstraintType::Performance("< 100ms".into()),
                "Fast",
            ));
        let concerns = ConcernAnalyzer::analyze(&intent);
        assert!(concerns.iter().any(|c| c.name == "caching"));
    }

    #[test]
    fn test_boundary_inferrer() {
        let entities = vec![
            EntitySpec::new("User", "A user").as_aggregate_root(),
            EntitySpec::new("Post", "A post"),
        ];
        let boundaries = BoundaryInferrer::infer_boundaries(&entities);
        assert!(!boundaries.is_empty());
    }

    #[test]
    fn test_cross_cutting_detector() {
        let entities = vec![
            EntitySpec::new("User", "A user")
                .with_field(FieldSpec::new("role", FieldType::Reference("Role".into()))),
            EntitySpec::new("Post", "A post").with_field(FieldSpec::new(
                "author_role",
                FieldType::Reference("Role".into()),
            )),
            EntitySpec::new("Role", "A role"),
        ];
        let cross = CrossCuttingDetector::detect(&entities);
        assert!(cross.contains(&"Role".to_string()));
    }

    #[test]
    fn test_cross_cutting_detector_empty() {
        let entities = vec![EntitySpec::new("User", "A user")];
        let cross = CrossCuttingDetector::detect(&entities);
        assert!(cross.is_empty());
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(LayerDecomposer::name(), "LayerDecomposer");
        assert_eq!(LayerDecomposer::tier(), 1);
        assert_eq!(ConcernAnalyzer::name(), "ConcernAnalyzer");
        assert_eq!(BoundaryInferrer::name(), "BoundaryInferrer");
        assert_eq!(CrossCuttingDetector::name(), "CrossCuttingDetector");
    }
}

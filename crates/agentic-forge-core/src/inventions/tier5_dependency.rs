//! Tier 5: Dependency inventions.
//! DependencyInferrer, VersionResolver, ApiSpecExtractor, ConflictResolver

use crate::types::blueprint::*;
use crate::types::intent::*;

pub struct DependencyInferrer;

impl DependencyInferrer {
    pub fn infer(
        domain: Domain,
        entities: &[Entity],
        constraints: &[Constraint],
    ) -> Vec<Dependency> {
        let mut deps = Vec::new();

        // Core deps for Rust
        deps.push(Dependency::new("serde", "1.0"));
        deps.push(Dependency::new("serde_json", "1.0"));
        deps.push(Dependency::new("thiserror", "2.0"));

        match domain {
            Domain::Web | Domain::Api => {
                deps.push(Dependency::new("axum", "0.7"));
                deps.push(Dependency::new("tokio", "1.35"));
                deps.push(Dependency::new("tower", "0.4"));
                deps.push(Dependency::new("tower-http", "0.5"));
                deps.push(Dependency::new("tracing", "0.1"));
            }
            Domain::Cli => {
                deps.push(Dependency::new("clap", "4.4"));
                deps.push(Dependency::new("anyhow", "1.0"));
            }
            Domain::Library => {
                deps.push(Dependency::new("tracing", "0.1"));
            }
            Domain::Service => {
                deps.push(Dependency::new("tokio", "1.35"));
                deps.push(Dependency::new("tracing", "0.1"));
            }
            _ => {}
        }

        if !entities.is_empty() {
            deps.push(Dependency::new("uuid", "1.6"));
            deps.push(Dependency::new("chrono", "0.4"));
        }

        for constraint in constraints {
            if matches!(constraint.constraint_type, ConstraintType::Security(_)) {
                deps.push(Dependency::new("argon2", "0.5"));
                deps.push(Dependency::new("jsonwebtoken", "9.2"));
            }
        }

        deps
    }

    pub fn name() -> &'static str {
        "DependencyInferrer"
    }
    pub fn tier() -> u8 {
        5
    }
}

pub struct VersionResolver;

impl VersionResolver {
    pub fn resolve(deps: &[Dependency]) -> Vec<ResolvedDependency> {
        deps.iter()
            .map(|d| {
                let compatible = Self::check_compatibility(&d.name, &d.version);
                ResolvedDependency {
                    name: d.name.clone(),
                    requested_version: d.version.clone(),
                    resolved_version: d.version.clone(),
                    is_compatible: compatible,
                    conflicts: Vec::new(),
                }
            })
            .collect()
    }

    fn check_compatibility(name: &str, version: &str) -> bool {
        let major: u32 = version
            .split('.')
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        match name {
            "serde" => major >= 1,
            "tokio" => major >= 1,
            "axum" => version.starts_with("0.7") || major >= 1,
            _ => true,
        }
    }

    pub fn name() -> &'static str {
        "VersionResolver"
    }
    pub fn tier() -> u8 {
        5
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolvedDependency {
    pub name: String,
    pub requested_version: String,
    pub resolved_version: String,
    pub is_compatible: bool,
    pub conflicts: Vec<String>,
}

pub struct ApiSpecExtractor;

impl ApiSpecExtractor {
    pub fn extract(entities: &[Entity], domain: Domain) -> Vec<ApiEndpoint> {
        if !matches!(domain, Domain::Web | Domain::Api | Domain::Service) {
            return Vec::new();
        }

        let mut endpoints = Vec::new();
        for entity in entities {
            let base = format!("/api/{}", entity.name.to_lowercase());
            endpoints.push(ApiEndpoint {
                method: "GET".into(),
                path: base.clone(),
                description: format!("List all {}s", entity.name.to_lowercase()),
                request_body: None,
                response_type: format!("Vec<{}>", entity.name),
            });
            endpoints.push(ApiEndpoint {
                method: "POST".into(),
                path: base.clone(),
                description: format!("Create a {}", entity.name.to_lowercase()),
                request_body: Some(format!("Create{}Input", entity.name)),
                response_type: entity.name.clone(),
            });
            endpoints.push(ApiEndpoint {
                method: "GET".into(),
                path: format!("{}/:id", base),
                description: format!("Get {} by ID", entity.name.to_lowercase()),
                request_body: None,
                response_type: entity.name.clone(),
            });
            endpoints.push(ApiEndpoint {
                method: "PUT".into(),
                path: format!("{}/:id", base),
                description: format!("Update {}", entity.name.to_lowercase()),
                request_body: Some(format!("Update{}Input", entity.name)),
                response_type: entity.name.clone(),
            });
            endpoints.push(ApiEndpoint {
                method: "DELETE".into(),
                path: format!("{}/:id", base),
                description: format!("Delete {}", entity.name.to_lowercase()),
                request_body: None,
                response_type: "()".into(),
            });
        }
        endpoints
    }

    pub fn name() -> &'static str {
        "ApiSpecExtractor"
    }
    pub fn tier() -> u8 {
        5
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
    pub request_body: Option<String>,
    pub response_type: String,
}

pub struct ConflictResolver;

impl ConflictResolver {
    pub fn resolve(deps: &[Dependency]) -> Vec<DependencyConflict> {
        let mut conflicts = Vec::new();
        let mut seen: std::collections::HashMap<String, Vec<&Dependency>> =
            std::collections::HashMap::new();

        for dep in deps {
            seen.entry(dep.name.clone()).or_default().push(dep);
        }

        for (name, versions) in &seen {
            if versions.len() > 1 {
                let version_strs: Vec<&str> = versions.iter().map(|d| d.version.as_str()).collect();
                conflicts.push(DependencyConflict {
                    dependency: name.clone(),
                    versions: version_strs.iter().map(|v| v.to_string()).collect(),
                    resolution: format!("Use latest: {}", version_strs.last().unwrap()),
                });
            }
        }

        conflicts
    }

    pub fn name() -> &'static str {
        "ConflictResolver"
    }
    pub fn tier() -> u8 {
        5
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyConflict {
    pub dependency: String,
    pub versions: Vec<String>,
    pub resolution: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_inferrer_api() {
        let deps = DependencyInferrer::infer(Domain::Api, &[], &[]);
        assert!(deps.iter().any(|d| d.name == "axum"));
        assert!(deps.iter().any(|d| d.name == "tokio"));
        assert!(deps.iter().any(|d| d.name == "serde"));
    }

    #[test]
    fn test_dependency_inferrer_cli() {
        let deps = DependencyInferrer::infer(Domain::Cli, &[], &[]);
        assert!(deps.iter().any(|d| d.name == "clap"));
        assert!(!deps.iter().any(|d| d.name == "axum"));
    }

    #[test]
    fn test_dependency_inferrer_with_entities() {
        let entities = vec![Entity::new("User", "A user")];
        let deps = DependencyInferrer::infer(Domain::Api, &entities, &[]);
        assert!(deps.iter().any(|d| d.name == "uuid"));
        assert!(deps.iter().any(|d| d.name == "chrono"));
    }

    #[test]
    fn test_dependency_inferrer_security() {
        let constraints = vec![Constraint::new(
            "auth",
            ConstraintType::Security("jwt".into()),
            "JWT auth",
        )];
        let deps = DependencyInferrer::infer(Domain::Api, &[], &constraints);
        assert!(deps.iter().any(|d| d.name == "jsonwebtoken"));
    }

    #[test]
    fn test_version_resolver() {
        let deps = vec![
            Dependency::new("serde", "1.0"),
            Dependency::new("tokio", "1.35"),
        ];
        let resolved = VersionResolver::resolve(&deps);
        assert!(resolved.iter().all(|r| r.is_compatible));
    }

    #[test]
    fn test_api_spec_extractor() {
        let entities = vec![Entity::new("User", "A user")];
        let endpoints = ApiSpecExtractor::extract(&entities, Domain::Api);
        assert_eq!(endpoints.len(), 5);
        assert!(endpoints.iter().any(|e| e.method == "POST"));
    }

    #[test]
    fn test_api_spec_extractor_non_api() {
        let entities = vec![Entity::new("User", "A user")];
        let endpoints = ApiSpecExtractor::extract(&entities, Domain::Library);
        assert!(endpoints.is_empty());
    }

    #[test]
    fn test_conflict_resolver_no_conflicts() {
        let deps = vec![
            Dependency::new("serde", "1.0"),
            Dependency::new("tokio", "1.35"),
        ];
        let conflicts = ConflictResolver::resolve(&deps);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_conflict_resolver_with_conflicts() {
        let deps = vec![
            Dependency::new("serde", "1.0"),
            Dependency::new("serde", "2.0"),
        ];
        let conflicts = ConflictResolver::resolve(&deps);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].dependency, "serde");
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(DependencyInferrer::name(), "DependencyInferrer");
        assert_eq!(VersionResolver::name(), "VersionResolver");
        assert_eq!(ApiSpecExtractor::name(), "ApiSpecExtractor");
        assert_eq!(ConflictResolver::name(), "ConflictResolver");
    }
}

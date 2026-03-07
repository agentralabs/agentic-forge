//! Tier 7: Integration inventions.
//! WiringDiagramBuilder, DataFlowSpecifier, InitSequencer, ShutdownSequencer

use crate::types::blueprint::*;

pub struct WiringDiagramBuilder;

impl WiringDiagramBuilder {
    pub fn build(entities: &[Entity], layers: &[ArchitectureLayer]) -> Vec<ComponentWiring> {
        let mut wirings = Vec::new();

        for entity in entities {
            let service = format!("{}Service", entity.name);
            let repo = format!("{}Repository", entity.name);
            wirings.push(ComponentWiring {
                source: service.clone(),
                target: repo.clone(),
                wiring_type: WiringType::DependencyInjection,
                description: format!("{} depends on {}", service, repo),
            });
        }

        for layer in layers {
            for dep in &layer.allowed_dependencies {
                wirings.push(ComponentWiring {
                    source: layer.name.clone(),
                    target: dep.clone(),
                    wiring_type: WiringType::DirectCall,
                    description: format!("{} layer depends on {} layer", layer.name, dep),
                });
            }
        }

        wirings
    }

    pub fn name() -> &'static str { "WiringDiagramBuilder" }
    pub fn tier() -> u8 { 7 }
}

pub struct DataFlowSpecifier;

impl DataFlowSpecifier {
    pub fn specify(entities: &[Entity], is_async: bool) -> Vec<DataFlow> {
        let mut flows = Vec::new();

        for entity in entities {
            flows.push(DataFlow {
                source: "Client".into(),
                target: format!("{}Handler", entity.name),
                data_type: format!("{}Request", entity.name),
                direction: FlowDirection::Unidirectional,
                is_async,
            });
            flows.push(DataFlow {
                source: format!("{}Handler", entity.name),
                target: format!("{}Service", entity.name),
                data_type: entity.name.clone(),
                direction: FlowDirection::Bidirectional,
                is_async,
            });
            flows.push(DataFlow {
                source: format!("{}Service", entity.name),
                target: format!("{}Repository", entity.name),
                data_type: entity.name.clone(),
                direction: FlowDirection::Bidirectional,
                is_async,
            });
        }

        flows
    }

    pub fn name() -> &'static str { "DataFlowSpecifier" }
    pub fn tier() -> u8 { 7 }
}

pub struct InitSequencer;

impl InitSequencer {
    pub fn sequence(blueprint: &Blueprint) -> Vec<InitStep> {
        let mut steps = Vec::new();
        let mut order = 0;

        steps.push(InitStep { order: { order += 1; order }, name: "config".into(), description: "Load configuration".into() });
        steps.push(InitStep { order: { order += 1; order }, name: "logging".into(), description: "Initialize logging/tracing".into() });

        if blueprint.dependencies.iter().any(|d| d.name.contains("postgres") || d.name.contains("sqlx")) {
            steps.push(InitStep { order: { order += 1; order }, name: "database".into(), description: "Connect to database".into() });
            steps.push(InitStep { order: { order += 1; order }, name: "migrations".into(), description: "Run database migrations".into() });
        }

        for entity in &blueprint.entities {
            steps.push(InitStep { order: { order += 1; order }, name: format!("{}_repo", entity.name.to_lowercase()), description: format!("Initialize {} repository", entity.name) });
            steps.push(InitStep { order: { order += 1; order }, name: format!("{}_service", entity.name.to_lowercase()), description: format!("Initialize {} service", entity.name) });
        }

        if blueprint.dependencies.iter().any(|d| d.name == "axum" || d.name == "actix-web") {
            steps.push(InitStep { order: { order += 1; order }, name: "router".into(), description: "Configure routes".into() });
            steps.push(InitStep { order: { order += 1; order }, name: "server".into(), description: "Start HTTP server".into() });
        }
        let _ = order;

        steps
    }

    pub fn name() -> &'static str { "InitSequencer" }
    pub fn tier() -> u8 { 7 }
}

pub struct ShutdownSequencer;

impl ShutdownSequencer {
    pub fn sequence(blueprint: &Blueprint) -> Vec<ShutdownStep> {
        let mut steps = Vec::new();
        let mut order = 0;

        if blueprint.dependencies.iter().any(|d| d.name == "axum" || d.name == "actix-web") {
            steps.push(ShutdownStep { order: { order += 1; order }, name: "server".into(), description: "Stop accepting connections".into(), timeout_ms: 5000 });
            steps.push(ShutdownStep { order: { order += 1; order }, name: "drain_requests".into(), description: "Drain in-flight requests".into(), timeout_ms: 30000 });
        }

        if blueprint.dependencies.iter().any(|d| d.name.contains("postgres") || d.name.contains("sqlx")) {
            steps.push(ShutdownStep { order: { order += 1; order }, name: "database".into(), description: "Close database connections".into(), timeout_ms: 5000 });
        }

        steps.push(ShutdownStep { order: { order += 1; order }, name: "flush_logs".into(), description: "Flush log buffers".into(), timeout_ms: 2000 });
        let _ = order;

        steps
    }

    pub fn name() -> &'static str { "ShutdownSequencer" }
    pub fn tier() -> u8 { 7 }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InitStep {
    pub order: usize,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShutdownStep {
    pub order: usize,
    pub name: String,
    pub description: String,
    pub timeout_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::intent::Domain;

    #[test]
    fn test_wiring_diagram_builder() {
        let entities = vec![Entity::new("User", "A user"), Entity::new("Post", "A post")];
        let layers = vec![
            ArchitectureLayer { name: "application".into(), description: "".into(), modules: vec![], allowed_dependencies: vec!["domain".into()] },
        ];
        let wirings = WiringDiagramBuilder::build(&entities, &layers);
        assert!(wirings.len() >= 3);
        assert!(wirings.iter().any(|w| w.source == "UserService"));
    }

    #[test]
    fn test_data_flow_specifier() {
        let entities = vec![Entity::new("User", "A user")];
        let flows = DataFlowSpecifier::specify(&entities, true);
        assert_eq!(flows.len(), 3);
        assert!(flows.iter().all(|f| f.is_async));
    }

    #[test]
    fn test_data_flow_sync() {
        let entities = vec![Entity::new("User", "A user")];
        let flows = DataFlowSpecifier::specify(&entities, false);
        assert!(flows.iter().all(|f| !f.is_async));
    }

    #[test]
    fn test_init_sequencer() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Api);
        bp.entities.push(Entity::new("User", "A user"));
        bp.dependencies.push(Dependency::new("axum", "0.7"));
        let steps = InitSequencer::sequence(&bp);
        assert!(steps.len() >= 6);
        assert_eq!(steps[0].name, "config");
        assert!(steps.iter().any(|s| s.name == "server"));
    }

    #[test]
    fn test_init_sequencer_with_db() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Api);
        bp.dependencies.push(Dependency::new("sqlx", "0.7"));
        let steps = InitSequencer::sequence(&bp);
        assert!(steps.iter().any(|s| s.name == "database"));
        assert!(steps.iter().any(|s| s.name == "migrations"));
    }

    #[test]
    fn test_shutdown_sequencer() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Api);
        bp.dependencies.push(Dependency::new("axum", "0.7"));
        let steps = ShutdownSequencer::sequence(&bp);
        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.name == "server"));
        assert!(steps.iter().any(|s| s.name == "flush_logs"));
    }

    #[test]
    fn test_shutdown_sequencer_minimal() {
        let bp = Blueprint::new("Test", "Test", Domain::Library);
        let steps = ShutdownSequencer::sequence(&bp);
        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.name == "flush_logs"));
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(WiringDiagramBuilder::name(), "WiringDiagramBuilder");
        assert_eq!(DataFlowSpecifier::name(), "DataFlowSpecifier");
        assert_eq!(InitSequencer::name(), "InitSequencer");
        assert_eq!(ShutdownSequencer::name(), "ShutdownSequencer");
    }
}

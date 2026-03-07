//! AgenticForge CLI — aforge command.

use agentic_forge_core::engine::ForgeEngine;
use agentic_forge_core::types::blueprint::*;
use agentic_forge_core::types::ids::*;
use agentic_forge_core::types::intent::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "aforge",
    version,
    about = "AgenticForge — Blueprint Engine CLI"
)]
struct Cli {
    #[arg(long, default_value = "text")]
    format: String,
    #[arg(long)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Blueprint {
        #[command(subcommand)]
        action: BlueprintAction,
    },
    Entity {
        #[command(subcommand)]
        action: EntityAction,
    },
    Dependency {
        #[command(subcommand)]
        action: DependencyAction,
    },
    Structure {
        #[command(subcommand)]
        action: StructureAction,
    },
    Skeleton {
        #[command(subcommand)]
        action: SkeletonAction,
    },
    Test {
        #[command(subcommand)]
        action: TestAction,
    },
    Import {
        #[command(subcommand)]
        action: ImportAction,
    },
    Wiring {
        #[command(subcommand)]
        action: WiringAction,
    },
    Export {
        #[command(subcommand)]
        action: ExportAction,
    },
    Serve {
        #[arg(long, default_value = "stdio")]
        mode: String,
    },
    Info,
    Version,
    Validate {
        blueprint_id: String,
    },
    Decompose {
        #[arg(long)]
        domain: String,
    },
    Infer {
        #[arg(long)]
        description: String,
    },
    Resolve {
        blueprint_id: String,
    },
    Init {
        #[arg(long)]
        name: String,
        #[arg(long)]
        domain: String,
    },
    Status {
        blueprint_id: String,
    },
    Summary {
        blueprint_id: String,
    },
    Concerns {
        #[arg(long)]
        domain: String,
    },
    Layers {
        #[arg(long)]
        domain: String,
    },
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Health,
    Clean,
}

#[derive(Subcommand)]
enum BlueprintAction {
    Create {
        name: String,
        #[arg(long)]
        domain: String,
        #[arg(long, default_value = "")]
        description: String,
    },
    Get {
        id: String,
    },
    List {
        #[arg(long)]
        status: Option<String>,
    },
    Validate {
        id: String,
    },
    Export {
        id: String,
        #[arg(long, default_value = "json")]
        format: String,
    },
    Delete {
        id: String,
    },
    Update {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
enum EntityAction {
    Add {
        blueprint_id: String,
        name: String,
        #[arg(long, default_value = "")]
        description: String,
    },
    Infer {
        blueprint_id: String,
        description: String,
    },
    List {
        blueprint_id: String,
    },
    Remove {
        blueprint_id: String,
        entity_id: String,
    },
    Fields {
        blueprint_id: String,
        entity_name: String,
    },
}

#[derive(Subcommand)]
enum DependencyAction {
    Resolve {
        blueprint_id: String,
    },
    Add {
        blueprint_id: String,
        name: String,
        version: String,
    },
    List {
        blueprint_id: String,
    },
    Remove {
        blueprint_id: String,
        dep_id: String,
    },
}

#[derive(Subcommand)]
enum StructureAction {
    Generate { blueprint_id: String },
}

#[derive(Subcommand)]
enum SkeletonAction {
    Create { blueprint_id: String },
}

#[derive(Subcommand)]
enum TestAction {
    Generate { blueprint_id: String },
    List { blueprint_id: String },
}

#[derive(Subcommand)]
enum ImportAction {
    Graph { blueprint_id: String },
}

#[derive(Subcommand)]
enum WiringAction {
    Create { blueprint_id: String },
    List { blueprint_id: String },
}

#[derive(Subcommand)]
enum ExportAction {
    Json { blueprint_id: String },
    Forge { blueprint_id: String, path: String },
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Set { key: String, value: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_writer(std::io::stderr)
            .init();
    }

    let mut engine = ForgeEngine::new();

    match cli.command {
        Commands::Info => {
            println!("AgenticForge v{}", env!("CARGO_PKG_VERSION"));
            println!("Sister #11 — The Forge");
            println!("Blueprint Engine for project architecture");
            println!("Inventions: {}", agentic_forge_core::types::INVENTION_COUNT);
            println!("MCP Tools: {}", agentic_forge_core::types::MCP_TOOL_COUNT);
        }
        Commands::Version => {
            println!("aforge {}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Health => {
            println!("healthy");
        }
        Commands::Clean => {
            println!("Cleaned forge state");
        }
        Commands::Serve { mode } => {
            if mode == "stdio" {
                agentic_forge_mcp::transport::run_stdio()
                    .await
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            } else {
                eprintln!("Unknown serve mode: {}", mode);
            }
        }
        Commands::Blueprint { action } => match action {
            BlueprintAction::Create {
                name,
                domain,
                description,
            } => {
                let d = Domain::from_name(&domain).unwrap_or(Domain::Custom);
                let id = engine.create_blueprint(&name, &description, d)?;
                println!(
                    "{}",
                    serde_json::json!({"blueprint_id": id.to_string(), "name": name})
                );
            }
            BlueprintAction::Get { id } => {
                let bp_id: BlueprintId = id.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                println!("{}", serde_json::to_string_pretty(bp)?);
            }
            BlueprintAction::List { status: _ } => {
                for bp in engine.store.list() {
                    println!("{} — {} [{}]", bp.id, bp.name, bp.status.name());
                }
            }
            BlueprintAction::Validate { id } => {
                let bp_id: BlueprintId = id.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                let report =
                    agentic_forge_core::engine::validator::BlueprintValidator::validate(bp)?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            BlueprintAction::Export { id, format: _ } => {
                let bp_id: BlueprintId = id.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                println!("{}", serde_json::to_string_pretty(bp)?);
            }
            BlueprintAction::Delete { id } => {
                let bp_id: BlueprintId = id.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                engine.writer().delete_blueprint(&bp_id)?;
                println!("Deleted {}", id);
            }
            BlueprintAction::Update {
                id,
                name,
                description,
            } => {
                let bp_id: BlueprintId = id.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                if let Some(n) = name {
                    engine.writer().rename_blueprint(&bp_id, n)?;
                }
                if let Some(d) = description {
                    engine.writer().set_description(&bp_id, d)?;
                }
                println!("Updated {}", id);
            }
        },
        Commands::Entity { action } => match action {
            EntityAction::Add {
                blueprint_id,
                name,
                description,
            } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let entity = Entity::new(&name, &description);
                let eid = engine.writer().add_entity(&bp_id, entity)?;
                println!(
                    "{}",
                    serde_json::json!({"entity_id": eid.to_string(), "name": name})
                );
            }
            EntityAction::Infer {
                blueprint_id,
                description,
            } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let inferred = agentic_forge_core::inventions::EntityInferrer::infer(&description);
                for spec in &inferred {
                    let entity = Entity::new(&spec.name, &spec.description);
                    if let Ok(eid) = engine.writer().add_entity(&bp_id, entity) {
                        println!("Added {} ({})", spec.name, eid);
                    }
                }
            }
            EntityAction::List { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                for e in &bp.entities {
                    println!("{} — {}", e.id, e.name);
                }
            }
            EntityAction::Remove {
                blueprint_id,
                entity_id,
            } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let eid: EntityId = entity_id.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                engine.writer().remove_entity(&bp_id, &eid)?;
                println!("Removed entity {}", entity_id);
            }
            EntityAction::Fields {
                blueprint_id,
                entity_name,
            } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                let entity = bp
                    .find_entity(&entity_name)
                    .ok_or_else(|| anyhow::anyhow!("Entity not found"))?;
                for f in &entity.fields {
                    println!("{}: {}", f.name, f.field_type.name());
                }
            }
        },
        Commands::Dependency { action } => match action {
            DependencyAction::Resolve { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                let deps = agentic_forge_core::inventions::DependencyInferrer::infer(
                    bp.domain,
                    &bp.entities,
                    &[],
                );
                for dep in deps {
                    if engine.writer().add_dependency(&bp_id, dep).is_ok() {
                        println!("Added dependency");
                    }
                }
            }
            DependencyAction::Add {
                blueprint_id,
                name,
                version,
            } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let dep = Dependency::new(&name, &version);
                engine.writer().add_dependency(&bp_id, dep)?;
                println!("Added {} {}", name, version);
            }
            DependencyAction::List { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                for d in &bp.dependencies {
                    println!("{} = \"{}\"", d.name, d.version);
                }
            }
            DependencyAction::Remove {
                blueprint_id,
                dep_id,
            } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let did: DependencyId = dep_id.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                engine.writer().remove_dependency(&bp_id, &did)?;
                println!("Removed {}", dep_id);
            }
        },
        Commands::Structure {
            action: StructureAction::Generate { blueprint_id },
        } => {
            let bp_id: BlueprintId = blueprint_id
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            let bp = engine.store.load(&bp_id)?.clone();
            let files = agentic_forge_core::inventions::FileStructureGenerator::generate(&bp);
            for f in &files {
                println!("{}", f.path);
            }
        }
        Commands::Skeleton {
            action: SkeletonAction::Create { blueprint_id },
        } => {
            let bp_id: BlueprintId = blueprint_id
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            let bp = engine.store.load(&bp_id)?;
            for entity in &bp.entities {
                println!("--- {} ---", entity.name);
                println!(
                    "{}",
                    agentic_forge_core::inventions::SkeletonGenerator::generate(entity)
                );
            }
        }
        Commands::Test { action } => match action {
            TestAction::Generate { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let entities: Vec<Entity> = engine.store.load(&bp_id)?.entities.clone();
                for entity in &entities {
                    let tests = agentic_forge_core::inventions::TestCaseGenerator::generate(entity);
                    for tc in tests {
                        let _ = engine.writer().add_test_case(&bp_id, tc);
                    }
                }
                println!("Tests generated for {} entities", entities.len());
            }
            TestAction::List { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                for tc in &bp.test_cases {
                    println!("{} [{:?}]", tc.name, tc.test_type);
                }
            }
        },
        Commands::Import {
            action: ImportAction::Graph { blueprint_id },
        } => {
            let bp_id: BlueprintId = blueprint_id
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            let bp = engine.store.load(&bp_id)?;
            let edges = agentic_forge_core::inventions::ImportGraphGenerator::generate(&bp.files);
            for e in &edges {
                println!("{} -> {}", e.from_file, e.to_file);
            }
        }
        Commands::Wiring { action } => match action {
            WiringAction::Create { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                let wirings = agentic_forge_core::inventions::WiringDiagramBuilder::build(
                    &bp.entities,
                    &bp.layers,
                );
                for w in &wirings {
                    println!("{} -> {} ({:?})", w.source, w.target, w.wiring_type);
                }
            }
            WiringAction::List { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                for w in &bp.wiring {
                    println!("{} -> {}", w.source, w.target);
                }
            }
        },
        Commands::Export { action } => match action {
            ExportAction::Json { blueprint_id } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?;
                println!("{}", serde_json::to_string_pretty(bp)?);
            }
            ExportAction::Forge { blueprint_id, path } => {
                let bp_id: BlueprintId = blueprint_id
                    .parse()
                    .map_err(|e: String| anyhow::anyhow!(e))?;
                let bp = engine.store.load(&bp_id)?.clone();
                let p = std::path::Path::new(&path);
                agentic_forge_core::format::ForgeWriter::write_to_file(&[bp], p)?;
                println!("Exported to {}", path);
            }
        },
        Commands::Validate { blueprint_id } => {
            let bp_id: BlueprintId = blueprint_id
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            let bp = engine.store.load(&bp_id)?;
            let report = agentic_forge_core::engine::validator::BlueprintValidator::validate(bp)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::Decompose { domain } => {
            let d = Domain::from_name(&domain).unwrap_or(Domain::Custom);
            let layers = agentic_forge_core::inventions::LayerDecomposer::decompose(d);
            for l in &layers {
                println!("{}: {}", l.name, l.description);
            }
        }
        Commands::Infer { description } => {
            let entities = agentic_forge_core::inventions::EntityInferrer::infer(&description);
            for e in &entities {
                println!("{}: {}", e.name, e.description);
            }
        }
        Commands::Resolve { blueprint_id } => {
            let bp_id: BlueprintId = blueprint_id
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            let bp = engine.store.load(&bp_id)?;
            let deps = agentic_forge_core::inventions::DependencyInferrer::infer(
                bp.domain,
                &bp.entities,
                &[],
            );
            for d in &deps {
                println!("{} = \"{}\"", d.name, d.version);
            }
        }
        Commands::Init { name, domain } => {
            let d = Domain::from_name(&domain).unwrap_or(Domain::Custom);
            let id = engine.create_blueprint(&name, "", d)?;
            println!("Created blueprint {} ({})", name, id);
        }
        Commands::Status { blueprint_id } => {
            let bp_id: BlueprintId = blueprint_id
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            let bp = engine.store.load(&bp_id)?;
            println!("Status: {}", bp.status.name());
        }
        Commands::Summary { blueprint_id } => {
            let bp_id: BlueprintId = blueprint_id
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
            let r = engine.reader();
            let summary = r.blueprint_summary(&bp_id)?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        Commands::Concerns { domain } => {
            let d = Domain::from_name(&domain).unwrap_or(Domain::Custom);
            let intent = IntentSpec::new("temp", "temp", d);
            let concerns = agentic_forge_core::inventions::ConcernAnalyzer::analyze(&intent);
            for c in &concerns {
                println!("{}: {:?}", c.name, c.concern_type);
            }
        }
        Commands::Layers { domain } => {
            let d = Domain::from_name(&domain).unwrap_or(Domain::Custom);
            let layers = agentic_forge_core::inventions::LayerDecomposer::decompose(d);
            for l in &layers {
                println!("{}: {}", l.name, l.description);
            }
        }
        Commands::Config { action } => match action {
            ConfigAction::Show => {
                println!("Default configuration");
            }
            ConfigAction::Set { key, value } => {
                println!("Set {} = {}", key, value);
            }
        },
    }

    Ok(())
}

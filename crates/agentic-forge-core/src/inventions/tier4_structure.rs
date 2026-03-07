//! Tier 4: Structure inventions.
//! FileStructureGenerator, ImportGraphGenerator, ModuleHierarchyBuilder, ConfigDesigner

use crate::types::blueprint::*;
use crate::types::intent::*;

pub struct FileStructureGenerator;

impl FileStructureGenerator {
    pub fn generate(blueprint: &Blueprint) -> Vec<FileBlueprint> {
        let mut files = Vec::new();
        let lang = blueprint.metadata.get("language").map(|s| s.as_str()).unwrap_or("rust");

        match lang {
            "rust" => Self::generate_rust_structure(blueprint, &mut files),
            "python" => Self::generate_python_structure(blueprint, &mut files),
            "typescript" => Self::generate_typescript_structure(blueprint, &mut files),
            _ => Self::generate_rust_structure(blueprint, &mut files),
        }

        files
    }

    fn generate_rust_structure(bp: &Blueprint, files: &mut Vec<FileBlueprint>) {
        files.push(FileBlueprint::new("Cargo.toml", FileType::Config));
        files.push(FileBlueprint::new("src/lib.rs", FileType::Source));
        files.push(FileBlueprint::new("src/main.rs", FileType::Source));
        files.push(FileBlueprint::new("src/types/mod.rs", FileType::Source));
        files.push(FileBlueprint::new("src/types/error.rs", FileType::Source));

        for entity in &bp.entities {
            let name = entity.name.to_lowercase();
            files.push(FileBlueprint::new(&format!("src/models/{}.rs", name), FileType::Source));
            files.push(FileBlueprint::new(&format!("tests/test_{}.rs", name), FileType::Test));
        }

        for layer in &bp.layers {
            files.push(FileBlueprint::new(&format!("src/{}/mod.rs", layer.name), FileType::Source));
        }
    }

    fn generate_python_structure(bp: &Blueprint, files: &mut Vec<FileBlueprint>) {
        files.push(FileBlueprint::new("pyproject.toml", FileType::Config));
        files.push(FileBlueprint::new("src/__init__.py", FileType::Source));
        files.push(FileBlueprint::new("src/models/__init__.py", FileType::Source));

        for entity in &bp.entities {
            let name = entity.name.to_lowercase();
            files.push(FileBlueprint::new(&format!("src/models/{}.py", name), FileType::Source));
            files.push(FileBlueprint::new(&format!("tests/test_{}.py", name), FileType::Test));
        }
    }

    fn generate_typescript_structure(bp: &Blueprint, files: &mut Vec<FileBlueprint>) {
        files.push(FileBlueprint::new("package.json", FileType::Config));
        files.push(FileBlueprint::new("tsconfig.json", FileType::Config));
        files.push(FileBlueprint::new("src/index.ts", FileType::Source));

        for entity in &bp.entities {
            let name = entity.name.to_lowercase();
            files.push(FileBlueprint::new(&format!("src/models/{}.ts", name), FileType::Source));
            files.push(FileBlueprint::new(&format!("tests/{}.test.ts", name), FileType::Test));
        }
    }

    pub fn name() -> &'static str { "FileStructureGenerator" }
    pub fn tier() -> u8 { 4 }
}

pub struct ImportGraphGenerator;

impl ImportGraphGenerator {
    pub fn generate(files: &[FileBlueprint]) -> Vec<ImportEdge> {
        let mut edges = Vec::new();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();

        for file in files {
            if file.path.ends_with("main.rs") || file.path.ends_with("index.ts") {
                if let Some(lib) = paths.iter().find(|p| p.ends_with("lib.rs") || p.ends_with("index.ts")) {
                    if *lib != file.path.as_str() {
                        edges.push(ImportEdge {
                            from_file: file.path.clone(),
                            to_file: lib.to_string(),
                            imported_symbols: vec!["*".into()],
                        });
                    }
                }
            }

            for import in &file.imports {
                if let Some(target) = paths.iter().find(|p| p.contains(import)) {
                    edges.push(ImportEdge {
                        from_file: file.path.clone(),
                        to_file: target.to_string(),
                        imported_symbols: vec![import.clone()],
                    });
                }
            }
        }

        edges
    }

    pub fn name() -> &'static str { "ImportGraphGenerator" }
    pub fn tier() -> u8 { 4 }
}

pub struct ModuleHierarchyBuilder;

impl ModuleHierarchyBuilder {
    pub fn build(files: &[FileBlueprint]) -> Vec<ModuleNode> {
        let mut modules: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for file in files {
            if let Some(parent) = file.path.rsplit_once('/').map(|(p, _)| p.to_string()) {
                modules.entry(parent).or_default().push(file.path.clone());
            }
        }

        modules.into_iter().map(|(name, children)| ModuleNode {
            name,
            children,
            is_public: true,
        }).collect()
    }

    pub fn name() -> &'static str { "ModuleHierarchyBuilder" }
    pub fn tier() -> u8 { 4 }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleNode {
    pub name: String,
    pub children: Vec<String>,
    pub is_public: bool,
}

pub struct ConfigDesigner;

impl ConfigDesigner {
    pub fn design(domain: Domain, dependencies: &[Dependency]) -> Vec<ConfigEntry> {
        let mut config = Vec::new();

        config.push(ConfigEntry { key: "app.name".into(), value_type: "string".into(), default: "my-app".into(), description: "Application name".into() });
        config.push(ConfigEntry { key: "app.version".into(), value_type: "string".into(), default: "0.1.0".into(), description: "Application version".into() });

        if matches!(domain, Domain::Web | Domain::Api | Domain::Service) {
            config.push(ConfigEntry { key: "server.host".into(), value_type: "string".into(), default: "0.0.0.0".into(), description: "Server host".into() });
            config.push(ConfigEntry { key: "server.port".into(), value_type: "u16".into(), default: "8080".into(), description: "Server port".into() });
        }

        if dependencies.iter().any(|d| d.name.contains("postgres") || d.name.contains("sqlx") || d.name.contains("diesel")) {
            config.push(ConfigEntry { key: "database.url".into(), value_type: "string".into(), default: "postgres://localhost/mydb".into(), description: "Database URL".into() });
            config.push(ConfigEntry { key: "database.max_connections".into(), value_type: "u32".into(), default: "10".into(), description: "Max DB connections".into() });
        }

        config.push(ConfigEntry { key: "log.level".into(), value_type: "string".into(), default: "info".into(), description: "Log level".into() });

        config
    }

    pub fn name() -> &'static str { "ConfigDesigner" }
    pub fn tier() -> u8 { 4 }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value_type: String,
    pub default: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_structure_rust() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Api);
        bp.entities.push(Entity::new("User", "A user"));
        let files = FileStructureGenerator::generate(&bp);
        assert!(files.iter().any(|f| f.path == "src/lib.rs"));
        assert!(files.iter().any(|f| f.path == "src/models/user.rs"));
        assert!(files.iter().any(|f| f.path == "tests/test_user.rs"));
    }

    #[test]
    fn test_file_structure_python() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Api);
        bp.metadata.insert("language".into(), "python".into());
        bp.entities.push(Entity::new("User", "A user"));
        let files = FileStructureGenerator::generate(&bp);
        assert!(files.iter().any(|f| f.path == "pyproject.toml"));
        assert!(files.iter().any(|f| f.path == "src/models/user.py"));
    }

    #[test]
    fn test_file_structure_typescript() {
        let mut bp = Blueprint::new("Test", "Test", Domain::Web);
        bp.metadata.insert("language".into(), "typescript".into());
        bp.entities.push(Entity::new("User", "A user"));
        let files = FileStructureGenerator::generate(&bp);
        assert!(files.iter().any(|f| f.path == "package.json"));
        assert!(files.iter().any(|f| f.path == "src/models/user.ts"));
    }

    #[test]
    fn test_import_graph() {
        let files = vec![
            FileBlueprint::new("src/main.rs", FileType::Source),
            FileBlueprint::new("src/lib.rs", FileType::Source),
        ];
        let edges = ImportGraphGenerator::generate(&files);
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_module_hierarchy() {
        let files = vec![
            FileBlueprint::new("src/main.rs", FileType::Source),
            FileBlueprint::new("src/lib.rs", FileType::Source),
            FileBlueprint::new("src/models/user.rs", FileType::Source),
        ];
        let modules = ModuleHierarchyBuilder::build(&files);
        assert!(!modules.is_empty());
    }

    #[test]
    fn test_config_designer_api() {
        let config = ConfigDesigner::design(Domain::Api, &[]);
        assert!(config.iter().any(|c| c.key == "server.port"));
    }

    #[test]
    fn test_config_designer_library() {
        let config = ConfigDesigner::design(Domain::Library, &[]);
        assert!(!config.iter().any(|c| c.key == "server.port"));
    }

    #[test]
    fn test_config_designer_with_db() {
        let deps = vec![Dependency::new("sqlx", "0.7")];
        let config = ConfigDesigner::design(Domain::Api, &deps);
        assert!(config.iter().any(|c| c.key == "database.url"));
    }

    #[test]
    fn test_invention_metadata() {
        assert_eq!(FileStructureGenerator::name(), "FileStructureGenerator");
        assert_eq!(ImportGraphGenerator::name(), "ImportGraphGenerator");
        assert_eq!(ModuleHierarchyBuilder::name(), "ModuleHierarchyBuilder");
        assert_eq!(ConfigDesigner::name(), "ConfigDesigner");
    }
}

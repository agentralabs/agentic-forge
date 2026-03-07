//! ID types for AgenticForge.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! forge_id {
    ($name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_context(context: &str) -> Self {
                Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, context.as_bytes()))
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}-{}", $prefix, self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let uuid_str = s.strip_prefix(concat!($prefix, "-")).unwrap_or(s);
                Uuid::parse_str(uuid_str)
                    .map(Self)
                    .map_err(|e| format!("Invalid {}: {}", stringify!($name), e))
            }
        }
    };
}

forge_id!(ForgeId, "frg");
forge_id!(BlueprintId, "bp");
forge_id!(EntityId, "ent");
forge_id!(OperationId, "op");
forge_id!(FileId, "file");
forge_id!(DependencyId, "dep");
forge_id!(TestCaseId, "tc");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forge_id_new() {
        let id1 = ForgeId::new();
        let id2 = ForgeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_forge_id_from_context() {
        let id1 = ForgeId::from_context("test");
        let id2 = ForgeId::from_context("test");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_forge_id_display() {
        let id = ForgeId::new();
        let s = id.to_string();
        assert!(s.starts_with("frg-"));
    }

    #[test]
    fn test_blueprint_id_unique() {
        let id1 = BlueprintId::new();
        let id2 = BlueprintId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_entity_id_from_context() {
        let id = EntityId::from_context("User");
        let id2 = EntityId::from_context("User");
        assert_eq!(id, id2);
    }

    #[test]
    fn test_operation_id_display() {
        let id = OperationId::new();
        assert!(id.to_string().starts_with("op-"));
    }

    #[test]
    fn test_file_id_display() {
        let id = FileId::new();
        assert!(id.to_string().starts_with("file-"));
    }

    #[test]
    fn test_dependency_id_display() {
        let id = DependencyId::new();
        assert!(id.to_string().starts_with("dep-"));
    }

    #[test]
    fn test_test_case_id_display() {
        let id = TestCaseId::new();
        assert!(id.to_string().starts_with("tc-"));
    }

    #[test]
    fn test_id_serialization() {
        let id = BlueprintId::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: BlueprintId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_id_from_str() {
        let id = ForgeId::new();
        let s = id.to_string();
        let parsed: ForgeId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_all_ids_default() {
        let _: ForgeId = Default::default();
        let _: BlueprintId = Default::default();
        let _: EntityId = Default::default();
        let _: OperationId = Default::default();
        let _: FileId = Default::default();
        let _: DependencyId = Default::default();
        let _: TestCaseId = Default::default();
    }
}

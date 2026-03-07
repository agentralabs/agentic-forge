//! AgenticForge — Blueprint engine for complete project architecture.
//!
//! Creates complete project blueprints (all files, types, signatures, deps, tests)
//! BEFORE any LLM code generation. Sister #11 "The Forge" in the Agentra Labs ecosystem.

pub mod bridges;
pub mod cache;
pub mod engine;
#[cfg(feature = "format")]
pub mod format;
pub mod index;
pub mod inventions;
pub mod metrics;
pub mod query;
pub mod security;
pub mod storage;
pub mod types;
pub mod validation;

pub use engine::{ForgeEngine, QueryEngine, WriteEngine};
pub use types::{
    BlueprintId, DependencyId, EntityId, FileId, ForgeError, ForgeId, ForgeResult, OperationId,
    TestCaseId,
};

//! Core types for AgenticForge.

pub mod blueprint;
pub mod error;
pub mod ids;
pub mod intent;

pub use blueprint::*;
pub use error::*;
pub use ids::*;
pub use intent::*;

pub const FORGE_MAGIC: [u8; 4] = [0x46, 0x52, 0x47, 0x45]; // "FRGE"
pub const FORMAT_VERSION: u32 = 1;
pub const HEADER_SIZE: usize = 256;
pub const FOOTER_SIZE: usize = 64;
pub const FOOTER_MAGIC: [u8; 8] = [0x46, 0x52, 0x47, 0x45, 0x45, 0x4E, 0x44, 0x00]; // "FRGEEND\0"
pub const MAX_ENTITIES: usize = 10_000;
pub const MAX_FILES: usize = 100_000;
pub const MAX_DEPENDENCIES: usize = 10_000;
pub const INVENTION_COUNT: usize = 32;
pub const MCP_TOOL_COUNT: usize = 15;

pub fn now_micros() -> u64 {
    chrono::Utc::now().timestamp_micros() as u64
}

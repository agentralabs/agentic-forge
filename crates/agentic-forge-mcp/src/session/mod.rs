//! Session management for MCP server.

use agentic_forge_core::engine::ForgeEngine;

pub struct SessionManager {
    pub engine: ForgeEngine,
    pub session_id: String,
    pub initialized: bool,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            engine: ForgeEngine::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

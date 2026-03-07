//! Server configuration.

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub transport: String,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: "stdio".into(),
            log_level: "info".into(),
        }
    }
}

//! Security module — auth, session, permissions.

use std::collections::HashMap;

pub struct AuthManager {
    token: Option<String>,
    sessions: HashMap<String, SessionBinding>,
    failed_attempts: u32,
    max_attempts: u32,
}

impl AuthManager {
    pub fn new() -> Self {
        let token = std::env::var("AGENTIC_AUTH_TOKEN").ok();
        Self {
            token,
            sessions: HashMap::new(),
            failed_attempts: 0,
            max_attempts: 10,
        }
    }

    pub fn is_auth_required(&self) -> bool {
        self.token.is_some()
    }

    pub fn authenticate(&mut self, provided_token: &str) -> Result<String, String> {
        if let Some(ref expected) = self.token {
            if provided_token == expected {
                let session_id = uuid::Uuid::new_v4().to_string();
                self.sessions.insert(session_id.clone(), SessionBinding {
                    session_id: session_id.clone(),
                    created_at: chrono::Utc::now().timestamp(),
                    last_activity: chrono::Utc::now().timestamp(),
                    permissions: Permissions::default(),
                });
                self.failed_attempts = 0;
                Ok(session_id)
            } else {
                self.failed_attempts += 1;
                Err("Invalid token".into())
            }
        } else {
            Ok("anonymous".into())
        }
    }

    pub fn is_rate_limited(&self) -> bool {
        self.failed_attempts >= self.max_attempts
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn validate_session(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }

    pub fn revoke_session(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SessionBinding {
    pub session_id: String,
    pub created_at: i64,
    pub last_activity: i64,
    pub permissions: Permissions,
}

#[derive(Debug, Clone, Default)]
pub struct Permissions {
    pub can_read: bool,
    pub can_write: bool,
    pub can_delete: bool,
    pub can_admin: bool,
}

impl Permissions {
    pub fn full() -> Self {
        Self { can_read: true, can_write: true, can_delete: true, can_admin: true }
    }

    pub fn read_only() -> Self {
        Self { can_read: true, ..Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_no_token() {
        let mut auth = AuthManager::new();
        // With no env var set, auth is not required
        if !auth.is_auth_required() {
            let session = auth.authenticate("anything").unwrap();
            assert_eq!(session, "anonymous");
        }
    }

    #[test]
    fn test_rate_limiting() {
        let mut auth = AuthManager {
            token: Some("secret".into()),
            sessions: HashMap::new(),
            failed_attempts: 0,
            max_attempts: 3,
        };
        assert!(!auth.is_rate_limited());
        auth.authenticate("wrong").unwrap_err();
        auth.authenticate("wrong").unwrap_err();
        auth.authenticate("wrong").unwrap_err();
        assert!(auth.is_rate_limited());
    }

    #[test]
    fn test_session_management() {
        let mut auth = AuthManager {
            token: Some("secret".into()),
            sessions: HashMap::new(),
            failed_attempts: 0,
            max_attempts: 10,
        };
        let session = auth.authenticate("secret").unwrap();
        assert!(auth.validate_session(&session));
        assert_eq!(auth.session_count(), 1);
        assert!(auth.revoke_session(&session));
        assert!(!auth.validate_session(&session));
    }

    #[test]
    fn test_permissions() {
        let full = Permissions::full();
        assert!(full.can_read && full.can_write && full.can_delete && full.can_admin);
        let ro = Permissions::read_only();
        assert!(ro.can_read && !ro.can_write);
    }

    #[test]
    fn test_default_permissions() {
        let p = Permissions::default();
        assert!(!p.can_read);
        assert!(!p.can_write);
    }
}

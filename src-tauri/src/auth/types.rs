//! Authentication types for OAuth token management

use serde::{Deserialize, Serialize};

/// Stored OAuth token information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Token expiration timestamp (Unix epoch seconds)
    pub expires_at: Option<i64>,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

impl AuthToken {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            refresh_token: None,
            expires_at: None,
            email: None,
            display_name: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                now >= expires_at
            }
            None => false, // No expiration means we assume it's valid
        }
    }
}

/// Authentication status for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

impl AuthStatus {
    pub fn not_authenticated() -> Self {
        Self {
            is_authenticated: false,
            email: None,
            display_name: None,
        }
    }

    pub fn from_token(token: &AuthToken) -> Self {
        Self {
            is_authenticated: !token.is_expired(),
            email: token.email.clone(),
            display_name: token.display_name.clone(),
        }
    }
}

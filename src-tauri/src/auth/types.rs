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

    /// Check if the access token is expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                // Add 60 second buffer to prevent edge cases
                now >= expires_at - 60
            }
            None => false, // No expiration means we assume it's valid
        }
    }

    /// Check if we can refresh the token (has a valid refresh token)
    pub fn can_refresh(&self) -> bool {
        self.refresh_token.is_some() && !self.refresh_token.as_ref().unwrap().is_empty()
    }

    /// Check if we're authenticated (either valid token or can refresh)
    pub fn is_authenticated(&self) -> bool {
        !self.is_expired() || self.can_refresh()
    }
}

/// Authentication status for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub is_authenticated: bool,
    /// Whether the access token needs refreshing (but we have a refresh token)
    pub needs_refresh: bool,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

impl AuthStatus {
    pub fn not_authenticated() -> Self {
        Self {
            is_authenticated: false,
            needs_refresh: false,
            email: None,
            display_name: None,
        }
    }

    pub fn from_token(token: &AuthToken) -> Self {
        Self {
            // User is authenticated if they have a valid token OR can refresh
            is_authenticated: token.is_authenticated(),
            needs_refresh: token.is_expired() && token.can_refresh(),
            email: token.email.clone(),
            display_name: token.display_name.clone(),
        }
    }
}

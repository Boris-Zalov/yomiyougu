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
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub client_secret: Option<String>,
}

impl AuthToken {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            refresh_token: None,
            expires_at: None,
            email: None,
            display_name: None,
            client_id: None,
            client_secret: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_new() {
        let token = AuthToken::new("test_token".to_string());
        assert_eq!(token.access_token, "test_token");
        assert!(token.refresh_token.is_none());
        assert!(token.expires_at.is_none());
        assert!(token.client_id.is_none());
    }

    #[test]
    fn test_auth_token_not_expired_when_no_expiry() {
        let token = AuthToken::new("test".to_string());
        assert!(!token.is_expired());
    }

    #[test]
    fn test_auth_token_expired() {
        let mut token = AuthToken::new("test".to_string());
        // Set expiry to 1 hour ago
        token.expires_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                - 3600,
        );
        assert!(token.is_expired());
    }

    #[test]
    fn test_auth_token_not_expired() {
        let mut token = AuthToken::new("test".to_string());
        // Set expiry to 1 hour from now
        token.expires_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                + 3600,
        );
        assert!(!token.is_expired());
    }

    #[test]
    fn test_auth_token_can_refresh() {
        let mut token = AuthToken::new("test".to_string());
        assert!(!token.can_refresh());

        token.refresh_token = Some("refresh".to_string());
        assert!(token.can_refresh());

        token.refresh_token = Some("".to_string());
        assert!(!token.can_refresh());
    }

    #[test]
    fn test_auth_token_is_authenticated() {
        // Fresh token without expiry - authenticated
        let token = AuthToken::new("test".to_string());
        assert!(token.is_authenticated());

        // Expired token without refresh - not authenticated
        let mut expired = AuthToken::new("test".to_string());
        expired.expires_at = Some(0); // Long expired
        assert!(!expired.is_authenticated());

        // Expired token with refresh - authenticated
        expired.refresh_token = Some("refresh".to_string());
        assert!(expired.is_authenticated());
    }

    #[test]
    fn test_auth_status_from_token() {
        // Valid token
        let token = AuthToken::new("test".to_string());
        let status = AuthStatus::from_token(&token);
        assert!(status.is_authenticated);
        assert!(!status.needs_refresh);

        // Expired token with refresh
        let mut expired = AuthToken::new("test".to_string());
        expired.expires_at = Some(0);
        expired.refresh_token = Some("refresh".to_string());
        expired.email = Some("test@example.com".to_string());
        let status = AuthStatus::from_token(&expired);
        assert!(status.is_authenticated);
        assert!(status.needs_refresh);
        assert_eq!(status.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_auth_status_not_authenticated() {
        let status = AuthStatus::not_authenticated();
        assert!(!status.is_authenticated);
        assert!(!status.needs_refresh);
        assert!(status.email.is_none());
    }
}

//! Authentication-related Tauri commands
//!
//! Implements Google OAuth 2.0 with refresh token support for both desktop and mobile.
//! Desktop: Uses local HTTP server callback
//! Mobile: Uses deep link callback (handled in frontend)

use crate::auth::{self, AuthStatus, AuthToken};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// OAuth configuration returned to frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthConfig {
    pub auth_url: String,
    pub state: String,
    pub code_verifier: String,
}

/// Token response from Google OAuth
#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    #[allow(dead_code)]
    token_type: Option<String>,
    #[allow(dead_code)]
    scope: Option<String>,
}

/// User info from Google
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: Option<String>,
    name: Option<String>,
}

/// Generate a random string for state/PKCE
fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate PKCE code challenge from verifier (S256 method)
fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    base64_url_encode(&result)
}

/// URL-safe base64 encoding (no padding)
fn base64_url_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(data)
}

/// Get the current authentication status
#[tauri::command]
pub async fn get_auth_status(app: tauri::AppHandle) -> Result<AuthStatus, String> {
    auth::get_auth_status(&app).map_err(|e| String::from(e))
}

/// Generate OAuth URL for Google sign-in
/// Returns the auth URL and state/verifier for PKCE verification
#[tauri::command]
pub async fn get_google_auth_url(
    client_id: String,
    redirect_uri: String,
    scope: String,
) -> Result<OAuthConfig, String> {
    let state = generate_random_string(32);
    let code_verifier = generate_random_string(64);
    let code_challenge = generate_code_challenge(&code_verifier);

    let params = [
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", scope.as_str()),
        ("state", state.as_str()),
        ("code_challenge", code_challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("access_type", "offline"), // This is key for getting refresh tokens!
        ("prompt", "consent"),      // Force consent to always get refresh token
    ];

    let query_string: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?{}",
        query_string
    );

    Ok(OAuthConfig {
        auth_url,
        state,
        code_verifier,
    })
}

/// Exchange authorization code for tokens
#[tauri::command]
pub async fn exchange_google_code(
    app: tauri::AppHandle,
    code: String,
    code_verifier: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
) -> Result<AuthStatus, String> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("code", code);
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("redirect_uri", redirect_uri);
    params.insert("grant_type", "authorization_code".to_string());
    params.insert("code_verifier", code_verifier);

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        log::error!("Token exchange failed: {}", error_text);
        return Err(format!("Token exchange failed: {}", error_text));
    }

    let token_response: GoogleTokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    log::info!(
        "Token exchange successful, refresh_token present: {}",
        token_response.refresh_token.is_some()
    );

    // Calculate expiration time
    let expires_at = token_response.expires_in.map(|expires_in| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64 + expires_in)
            .unwrap_or(0)
    });

    // Fetch user info
    let user_info = fetch_user_info(&token_response.access_token).await.ok();

    // Create and save token
    let mut token = AuthToken::new(token_response.access_token);
    token.refresh_token = token_response.refresh_token;
    token.expires_at = expires_at;
    token.email = user_info.as_ref().and_then(|u| u.email.clone());
    token.display_name = user_info.as_ref().and_then(|u| u.name.clone());

    auth::save_token(&app, &token).map_err(|e| String::from(e))?;

    Ok(AuthStatus::from_token(&token))
}

/// Refresh the access token using stored refresh token
#[tauri::command]
pub async fn refresh_google_token(
    app: tauri::AppHandle,
    client_id: String,
    client_secret: String,
) -> Result<AuthStatus, String> {
    // Load existing token
    let token = auth::load_token(&app).map_err(|e| String::from(e))?;

    let refresh_token = token
        .refresh_token
        .as_ref()
        .ok_or_else(|| "No refresh token available. Please sign in again.".to_string())?;

    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("refresh_token", refresh_token.clone());
    params.insert("grant_type", "refresh_token".to_string());

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        log::error!("Token refresh failed: {}", error_text);
        return Err(format!("Token refresh failed: {}", error_text));
    }

    let token_response: GoogleTokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    // Calculate expiration time
    let expires_at = token_response.expires_in.map(|expires_in| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64 + expires_in)
            .unwrap_or(0)
    });

    // Update token (keep existing refresh token if new one not provided)
    let mut new_token = AuthToken::new(token_response.access_token);
    new_token.refresh_token = token_response.refresh_token.or(token.refresh_token);
    new_token.expires_at = expires_at;
    new_token.email = token.email;
    new_token.display_name = token.display_name;

    auth::save_token(&app, &new_token).map_err(|e| String::from(e))?;

    log::info!("Token refreshed successfully");
    Ok(AuthStatus::from_token(&new_token))
}

/// Fetch user info from Google
async fn fetch_user_info(access_token: &str) -> Result<GoogleUserInfo, String> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    if !response.status().is_success() {
        return Err("Failed to fetch user info".to_string());
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))
}

/// Save Google auth token (called after OAuth flow)
#[tauri::command]
pub async fn save_google_auth_token(
    app: tauri::AppHandle,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
) -> Result<AuthStatus, String> {
    let mut token = AuthToken::new(access_token);
    token.refresh_token = refresh_token;
    token.expires_at = expires_at;

    auth::save_token(&app, &token).map_err(|e| String::from(e))?;
    Ok(AuthStatus::from_token(&token))
}

/// Set auth token manually (for development/testing)
#[tauri::command]
pub async fn set_auth_token(
    app: tauri::AppHandle,
    access_token: String,
    refresh_token: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
) -> Result<AuthStatus, String> {
    let mut token = AuthToken::new(access_token);
    token.refresh_token = refresh_token;
    token.email = email;
    token.display_name = display_name;

    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64 + 3600)
        .ok();
    token.expires_at = expires_at;

    auth::save_token(&app, &token).map_err(|e| String::from(e))?;
    Ok(AuthStatus::from_token(&token))
}

/// Logout from Google (clear stored tokens)
#[tauri::command]
pub async fn google_logout(app: tauri::AppHandle) -> Result<AuthStatus, String> {
    auth::clear_token(&app).map_err(|e| String::from(e))?;
    Ok(AuthStatus::not_authenticated())
}

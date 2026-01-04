//! Authentication-related Tauri commands
//!
//! Implements Google OAuth 2.0 with refresh token support for both desktop and mobile.
//! Desktop: Uses local HTTP server callback
//! Mobile: Uses deep link callback (handled in frontend)

use crate::auth::{self, AuthStatus, AuthToken};
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tauri_plugin_opener::OpenerExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

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

/// HTML response shown after successful authentication
const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login Successful</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            background-color: #f4f6f8;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            color: #333;
        }

        .container {
            background-color: #ffffff;
            padding: 40px;
            border-radius: 12px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
            text-align: center;
            max-width: 400px;
            width: 90%;
        }

        /* Success Icon */
        .icon-circle {
            width: 80px;
            height: 80px;
            background-color: #e6f4ea;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 20px auto;
        }

        .icon-circle svg {
            width: 40px;
            height: 40px;
            fill: #34a853;
        }

        h1 {
            margin: 0 0 10px 0;
            font-size: 24px;
            color: #202124;
        }

        p {
            margin: 0 0 30px 0;
            font-size: 16px;
            color: #5f6368;
            line-height: 1.5;
        }
    </style>
</head>
<body>

    <div class="container">
        <div class="icon-circle">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
            </svg>
        </div>

        <h1>Login Successful</h1>
        <p>You have successfully signed in. You can now close this window and return to the app.</p>
    </div>
</body>
</html>"#;

const ERROR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login Failed</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            background-color: #f4f6f8;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            color: #333;
        }

        .container {
            background-color: #ffffff;
            padding: 40px;
            border-radius: 12px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
            text-align: center;
            max-width: 400px;
            width: 90%;
        }

        /* Error Icon Styles */
        .icon-circle {
            width: 80px;
            height: 80px;
            background-color: #fce8e6;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 20px auto;
        }

        .icon-circle svg {
            width: 40px;
            height: 40px;
            fill: #d93025;
        }

        h1 {
            margin: 0 0 10px 0;
            font-size: 24px;
            color: #202124;
        }

        p {
            margin: 0 0 30px 0;
            font-size: 16px;
            color: #5f6368;
            line-height: 1.5;
        }
    </style>
</head>
<body>

    <div class="container">
        <div class="icon-circle">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
            </svg>
        </div>

        <h1>Login Failed</h1>
        <p>We were unable to sign you in. Please return to the app and try again.</p>
    </div>
</body>
</html>"#;

/// Get the current authentication status
#[tauri::command]
pub async fn get_auth_status(app: tauri::AppHandle) -> Result<AuthStatus, String> {
    auth::get_auth_status(&app).map_err(|e| String::from(e))
}

/// Google OAuth sign-in flow with local HTTP server
/// This command:
/// 1. Starts a local HTTP server on a random port
/// 2. Opens the browser with the OAuth URL
/// 3. Waits for the callback with the authorization code
/// 4. Exchanges the code for tokens
/// 5. Returns the authentication status
#[tauri::command]
pub async fn google_sign_in(
    app: tauri::AppHandle,
    client_id: String,
    client_secret: String,
    scope: String,
) -> Result<AuthStatus, String> {
    const OAUTH_PORT: u16 = 8085;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", OAUTH_PORT))
        .await
        .map_err(|e| format!("Failed to start local server on port {}: {}. Make sure no other app is using this port.", OAUTH_PORT, e))?;

    let redirect_uri = format!("http://127.0.0.1:{}", OAUTH_PORT);

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
        ("access_type", "offline"),
        ("prompt", "consent"),
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

    log::info!(
        "Opening OAuth URL in browser, listening on port {}",
        OAUTH_PORT
    );

    app.opener()
        .open_url(&auth_url, None::<&str>)
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(300), // 5 minute timeout
        wait_for_oauth_callback(&listener, &state),
    )
    .await
    .map_err(|_| "OAuth timeout - please try again".to_string())?;

    let code = result?;

    let auth_status = exchange_code_for_tokens(
        &app,
        code,
        code_verifier,
        client_id,
        client_secret,
        redirect_uri,
    )
    .await?;

    Ok(auth_status)
}

/// Wait for OAuth callback and extract authorization code
async fn wait_for_oauth_callback(
    listener: &TcpListener,
    expected_state: &str,
) -> Result<String, String> {
    loop {
        let (mut socket, _) = listener
            .accept()
            .await
            .map_err(|e| format!("Failed to accept connection: {}", e))?;

        let (reader, mut writer) = socket.split();
        let mut buf_reader = BufReader::new(reader);
        let mut request_line = String::new();

        buf_reader
            .read_line(&mut request_line)
            .await
            .map_err(|e| format!("Failed to read request: {}", e))?;

        log::debug!("Received OAuth callback: {}", request_line);

        // Parse the request to extract code and state
        if let Some(path) = request_line.split_whitespace().nth(1) {
            if let Some(query_start) = path.find('?') {
                let query = &path[query_start + 1..];
                let params: HashMap<&str, &str> = query
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.split('=');
                        Some((parts.next()?, parts.next()?))
                    })
                    .collect();

                if let Some(error) = params.get("error") {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        ERROR_HTML.len(),
                        ERROR_HTML
                    );
                    let _ = writer.write_all(response.as_bytes()).await;
                    return Err(format!("OAuth error: {}", error));
                }

                // Verify state
                if let Some(received_state) = params.get("state") {
                    if *received_state != expected_state {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            ERROR_HTML.len(),
                            ERROR_HTML
                        );
                        let _ = writer.write_all(response.as_bytes()).await;
                        return Err("Invalid state parameter - possible CSRF attack".to_string());
                    }
                }

                // Extract code
                if let Some(code) = params.get("code") {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        SUCCESS_HTML.len(),
                        SUCCESS_HTML
                    );
                    let _ = writer.write_all(response.as_bytes()).await;
                    return Ok(urlencoding::decode(code).unwrap_or_default().to_string());
                }
            }
        }

        // If we didn't get a valid OAuth callback, send an error and continue listening
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = writer.write_all(response.as_bytes()).await;
    }
}

/// Exchange authorization code for tokens
async fn exchange_code_for_tokens(
    app: &tauri::AppHandle,
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
    token.client_id = Some(params.get("client_id").unwrap().clone());
    token.client_secret = Some(params.get("client_secret").unwrap().clone());

    auth::save_token(app, &token).map_err(|e| String::from(e))?;

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

    let new_token = refresh_token_internal(&client_id, &client_secret, &token)
        .await
        .map_err(|e| String::from(e))?;

    auth::save_token(&app, &new_token).map_err(|e| String::from(e))?;

    log::info!("Token refreshed successfully");
    Ok(AuthStatus::from_token(&new_token))
}

/// Internal function to refresh a token (reusable from other modules)
pub async fn refresh_token_internal(
    client_id: &str,
    client_secret: &str,
    token: &AuthToken,
) -> Result<AuthToken, crate::error::AppError> {
    let refresh_token = token
        .refresh_token
        .as_ref()
        .ok_or_else(|| crate::error::AppError::not_authenticated())?;

    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", client_id.to_string());
    params.insert("client_secret", client_secret.to_string());
    params.insert("refresh_token", refresh_token.clone());
    params.insert("grant_type", "refresh_token".to_string());

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| crate::error::AppError::sync_failed(format!("Failed to refresh token: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        log::error!("Token refresh failed: {}", error_text);
        return Err(crate::error::AppError::sync_failed(format!("Token refresh failed: {}", error_text)));
    }

    let token_response: GoogleTokenResponse = response
        .json()
        .await
        .map_err(|e| crate::error::AppError::sync_failed(format!("Failed to parse token response: {}", e)))?;

    // Calculate expiration time
    let expires_at = token_response.expires_in.map(|expires_in| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64 + expires_in)
            .unwrap_or(0)
    });

    // Update token (keep existing refresh token if new one not provided)
    let mut new_token = AuthToken::new(token_response.access_token);
    new_token.refresh_token = token_response.refresh_token.or(token.refresh_token.clone());
    new_token.expires_at = expires_at;
    new_token.email = token.email.clone();
    new_token.display_name = token.display_name.clone();
    new_token.client_id = token.client_id.clone();
    new_token.client_secret = token.client_secret.clone();

    Ok(new_token)
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

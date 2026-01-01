//! OAuth token storage
//!
//! Uses IOTA Stronghold for credential storage.
//! Tokens are stored in an encrypted vault with argon2 key derivation.

use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use tauri::Manager;
use tauri_plugin_stronghold::stronghold::Stronghold;

use super::types::{AuthStatus, AuthToken};
use crate::error::AppError;

const VAULT_FILENAME: &str = "credentials.hold";
const SALT_FILENAME: &str = "salt.txt";

static TOKEN_KEY: LazyLock<String> = LazyLock::new(|| {
    std::env::var("STRONGHOLD_TOKEN_KEY").unwrap_or_else(|_| "google_oauth_token".to_string())
});
static VAULT_PASSWORD: LazyLock<String> = LazyLock::new(|| {
    std::env::var("STRONGHOLD_VAULT_PASSWORD").unwrap_or_else(|_| "yomiyougu_secure_vault_2025".to_string())
});

fn get_vault_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_local_data_dir()
        .map(|path| path.join(VAULT_FILENAME))
        .map_err(AppError::config_read_failed)
}

fn get_salt_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_local_data_dir()
        .map(|path| path.join(SALT_FILENAME))
        .map_err(AppError::config_read_failed)
}

/// Generate password hash using argon2 (matches the plugin's implementation)
fn hash_password(password: &str, salt_path: &PathBuf) -> Result<Vec<u8>, AppError> {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};

    let salt_bytes = if salt_path.exists() {
        fs::read(salt_path).map_err(AppError::config_read_failed)?
    } else {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let salt: Vec<u8> = (0..16).map(|_| rng.gen()).collect(); // 16 bytes for salt
        if let Some(parent) = salt_path.parent() {
            fs::create_dir_all(parent).map_err(AppError::config_write_failed)?;
        }
        fs::write(salt_path, &salt).map_err(AppError::config_write_failed)?;
        salt
    };

    // Create a valid salt string from bytes (base64-encoded)
    let salt = SaltString::b64_encode(&salt_bytes)
        .map_err(|e| AppError::config_read_failed(format!("Salt encoding failed: {}", e)))?;

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::config_read_failed(format!("Argon2 hash failed: {}", e)))?;

    // Extract the hash bytes (32 bytes)
    let hash_bytes = hash
        .hash
        .ok_or_else(|| AppError::config_read_failed("Hash output missing"))?;

    Ok(hash_bytes.as_bytes().to_vec())
}

/// Open or create the Stronghold vault
fn open_vault(app: &tauri::AppHandle) -> Result<Stronghold, AppError> {
    let vault_path = get_vault_path(app)?;
    let salt_path = get_salt_path(app)?;
    let password = hash_password(&VAULT_PASSWORD, &salt_path)?;

    if let Some(parent) = vault_path.parent() {
        fs::create_dir_all(parent).map_err(AppError::config_write_failed)?;
    }

    Stronghold::new(&vault_path, password).map_err(|e| AppError::config_read_failed(e.to_string()))
}

/// Check if user is authenticated (has valid token or can refresh)
pub fn get_auth_status(app: &tauri::AppHandle) -> Result<AuthStatus, AppError> {
    match load_token(app) {
        Ok(token) => {
            log::debug!(
                "Token loaded: is_expired={}, can_refresh={}, is_authenticated={}",
                token.is_expired(),
                token.can_refresh(),
                token.is_authenticated()
            );
            Ok(AuthStatus::from_token(&token))
        }
        Err(e) => {
            log::debug!("Failed to load token: {:?}", e);
            Ok(AuthStatus::not_authenticated())
        }
    }
}

/// Load OAuth token from secure storage
pub fn load_token(app: &tauri::AppHandle) -> Result<AuthToken, AppError> {
    log::debug!("Loading token from Stronghold vault...");
    let stronghold = open_vault(app)?;
    
    let client_name = b"auth_client";
    let client = stronghold.load_client(client_name)
        .or_else(|_| stronghold.create_client(client_name))
        .map_err(|e| AppError::config_read_failed(e.to_string()))?;
    
    let store = client.store();
    let data = store
        .get(TOKEN_KEY.as_bytes())
        .map_err(|e| AppError::config_read_failed(e.to_string()))?;

    log::debug!("Token data from store: {:?}", data.as_ref().map(|d| d.len()));

    match data {
        Some(bytes) => {
            let json =
                String::from_utf8(bytes).map_err(|e| AppError::config_parse_failed(e.to_string()))?;
            log::debug!("Token JSON loaded: {} bytes", json.len());
            let token: AuthToken =
                serde_json::from_str(&json).map_err(AppError::config_parse_failed)?;
            Ok(token)
        }
        None => {
            log::debug!("No token found in store");
            Err(AppError::not_authenticated())
        }
    }
}

pub fn save_token(app: &tauri::AppHandle, token: &AuthToken) -> Result<(), AppError> {
    let stronghold = open_vault(app)?;
    
    let client_name = b"auth_client";
    let client = stronghold.load_client(client_name)
        .or_else(|_| stronghold.create_client(client_name))
        .map_err(|e| AppError::config_read_failed(e.to_string()))?;
    
    let store = client.store();

    let json = serde_json::to_string(token).map_err(AppError::serialization_failed)?;

    store
        .insert(TOKEN_KEY.as_bytes().to_vec(), json.into_bytes(), None)
        .map_err(|e| AppError::config_write_failed(e.to_string()))?;

    stronghold.write_client(client_name)
        .map_err(|e| AppError::config_write_failed(e.to_string()))?;

    stronghold
        .save()
        .map_err(|e| AppError::config_write_failed(e.to_string()))?;

    log::info!("OAuth token securely stored in Stronghold vault");
    Ok(())
}

/// Clear stored OAuth token (logout)
pub fn clear_token(app: &tauri::AppHandle) -> Result<(), AppError> {
    let stronghold = open_vault(app)?;
    
    let client_name = b"auth_client";
    let client = stronghold.load_client(client_name)
        .or_else(|_| stronghold.create_client(client_name))
        .map_err(|e| AppError::config_read_failed(e.to_string()))?;
    
    let store = client.store();

    let _ = store.delete(TOKEN_KEY.as_bytes());

    let _ = stronghold.write_client(client_name);
    stronghold
        .save()
        .map_err(|e| AppError::config_write_failed(e.to_string()))?;

    log::info!("OAuth token cleared from Stronghold vault");
    Ok(())
}

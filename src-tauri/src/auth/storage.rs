//! OAuth token storage - file I/O operations for persisting auth tokens

use std::fs;
use std::path::PathBuf;
use tauri::Manager;

use super::types::{AuthStatus, AuthToken};
use crate::error::AppError;

const AUTH_FILENAME: &str = "auth.json";

/// Get the path to the auth token file
pub fn get_auth_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_config_dir()
        .map(|path| path.join(AUTH_FILENAME))
        .map_err(AppError::config_read_failed)
}

/// Check if user is authenticated (has valid token)
pub fn get_auth_status(app: &tauri::AppHandle) -> Result<AuthStatus, AppError> {
    match load_token(app) {
        Ok(token) => Ok(AuthStatus::from_token(&token)),
        Err(_) => Ok(AuthStatus::not_authenticated()),
    }
}

/// Load OAuth token from disk
pub fn load_token(app: &tauri::AppHandle) -> Result<AuthToken, AppError> {
    let path = get_auth_path(app)?;

    if !path.exists() {
        return Err(AppError::not_authenticated());
    }

    let json = fs::read_to_string(&path).map_err(AppError::config_read_failed)?;

    let token: AuthToken =
        serde_json::from_str(&json).map_err(AppError::config_parse_failed)?;

    Ok(token)
}

/// Save OAuth token to disk
pub fn save_token(app: &tauri::AppHandle, token: &AuthToken) -> Result<(), AppError> {
    let path = get_auth_path(app)?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::config_write_failed)?;
    }

    let json = serde_json::to_string_pretty(token).map_err(AppError::serialization_failed)?;

    fs::write(&path, json).map_err(AppError::config_write_failed)?;

    Ok(())
}

/// Clear stored OAuth token (logout)
pub fn clear_token(app: &tauri::AppHandle) -> Result<(), AppError> {
    let path = get_auth_path(app)?;

    if path.exists() {
        fs::remove_file(&path).map_err(AppError::config_write_failed)?;
    }

    Ok(())
}

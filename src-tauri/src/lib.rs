use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub theme_mode: String,
    pub accepted_license: bool,
    pub google_drive_enabled: bool,
    pub username: String,
}

/// Resolves the application configuration path safely.
/// Returns a Result to handle cases where the system path cannot be resolved.
fn get_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|path| path.join("config.json"))
        .map_err(|e| format!("Failed to resolve config directory: {}", e))
}

#[tauri::command]
fn check_config_exists(app: tauri::AppHandle) -> Result<bool, String> {
    let path = get_config_path(&app)?;
    Ok(path.exists())
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = get_config_path(&app)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let path = get_config_path(&app)?;
    
    if !path.exists() {
        return Err("Configuration file not found".to_string());
    }

    let json = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    Ok(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![check_config_exists, save_config, get_config])
        .run(tauri::generate_context!())
        .expect("Critical error while running tauri application");
}

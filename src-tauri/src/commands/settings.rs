//! Settings-related Tauri commands

use crate::settings::{self, AppSettings, SettingCategory, SettingValue};
use serde_json::Value;
use std::collections::HashMap;

/// Check if settings file exists (for first-run detection)
#[tauri::command]
pub async fn check_settings_exists(app: tauri::AppHandle) -> Result<bool, String> {
    settings::settings_exist(&app).map_err(|e| e.into())
}

/// Get complete settings with schema for UI rendering
#[tauri::command]
pub async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    settings::load_settings(&app).map_err(|e| e.into())
}

/// Get settings organized by category (for settings page)
#[tauri::command]
pub async fn get_settings_schema(app: tauri::AppHandle) -> Result<Vec<SettingCategory>, String> {
    let settings = settings::load_settings(&app).map_err(|e| String::from(e))?;
    Ok(settings.categories)
}

/// Get a single setting value by key
#[tauri::command]
pub async fn get_setting(
    app: tauri::AppHandle,
    key: String,
) -> Result<Option<SettingValue>, String> {
    let settings = settings::load_settings(&app).map_err(|e| String::from(e))?;
    Ok(settings.get(&key).cloned())
}

/// Update settings from UI form data
#[tauri::command]
pub async fn save_settings_from_schema(
    app: tauri::AppHandle,
    form_data: HashMap<String, Value>,
) -> Result<AppSettings, String> {
    settings::update_settings_from_map(&app, form_data).map_err(|e| e.into())
}

/// Update a single setting
#[tauri::command]
pub async fn update_setting(
    app: tauri::AppHandle,
    key: String,
    value: Value,
) -> Result<AppSettings, String> {
    let mut updates = HashMap::new();
    updates.insert(key, value);
    settings::update_settings_from_map(&app, updates).map_err(|e| e.into())
}

/// Complete initial setup wizard
#[tauri::command]
pub async fn complete_setup(
    app: tauri::AppHandle,
    initial_settings: Option<HashMap<String, Value>>,
) -> Result<(), String> {
    // Apply any initial settings from the wizard
    if let Some(updates) = initial_settings {
        settings::update_settings_from_map(&app, updates).map_err(|e| String::from(e))?;
    }

    // Mark setup as completed
    settings::complete_setup(&app).map_err(|e| e.into())
}

/// Reset all settings to defaults
#[tauri::command]
pub async fn reset_all_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    settings::reset_settings(&app).map_err(|e| e.into())
}

/// Reset a specific setting to its default
#[tauri::command]
pub async fn reset_setting(app: tauri::AppHandle, key: String) -> Result<AppSettings, String> {
    settings::reset_setting(&app, &key).map_err(|e| e.into())
}

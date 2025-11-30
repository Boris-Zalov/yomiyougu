//! Settings storage - file I/O operations for persisting settings

use std::fs;
use std::path::PathBuf;
use tauri::Manager;

use super::schema::create_default_settings;
use super::types::AppSettings;
use crate::error::AppError;

const SETTINGS_FILENAME: &str = "settings.json";

/// Get the path to the settings file
pub fn get_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_config_dir()
        .map(|path| path.join(SETTINGS_FILENAME))
        .map_err(|e| AppError::config_read_failed(e))
}

/// Check if settings file exists
pub fn settings_exist(app: &tauri::AppHandle) -> Result<bool, AppError> {
    let path = get_settings_path(app)?;
    Ok(path.exists())
}

/// Load settings from disk, returning defaults if not found
pub fn load_settings(app: &tauri::AppHandle) -> Result<AppSettings, AppError> {
    let path = get_settings_path(app)?;

    if !path.exists() {
        return Ok(create_default_settings());
    }

    let json = fs::read_to_string(&path).map_err(|e| AppError::config_read_failed(e))?;

    let settings: AppSettings =
        serde_json::from_str(&json).map_err(|e| AppError::config_parse_failed(e))?;

    // TODO: Handle schema version migrations here
    // if settings.version < SETTINGS_VERSION { migrate(settings) }

    Ok(settings)
}

/// Save settings to disk
pub fn save_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), AppError> {
    let path = get_settings_path(app)?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::config_write_failed(e))?;
    }

    let json =
        serde_json::to_string_pretty(settings).map_err(|e| AppError::serialization_failed(e))?;

    fs::write(&path, json).map_err(|e| AppError::config_write_failed(e))?;

    Ok(())
}

/// Initialize settings with defaults and save to disk
pub fn initialize_settings(app: &tauri::AppHandle) -> Result<AppSettings, AppError> {
    let settings = create_default_settings();
    save_settings(app, &settings)?;
    Ok(settings)
}

/// Update specific settings from a key-value map (used by UI)
pub fn update_settings_from_map(
    app: &tauri::AppHandle,
    updates: std::collections::HashMap<String, serde_json::Value>,
) -> Result<AppSettings, AppError> {
    let mut settings = load_settings(app)?;

    for (key, value) in updates {
        let setting_value = json_to_setting_value(value)
            .ok_or_else(|| AppError::invalid_setting_value(&key, "unsupported type"))?;

        if !settings.set(&key, setting_value) {
            return Err(AppError::invalid_setting_key(&key));
        }
    }

    save_settings(app, &settings)?;
    Ok(settings)
}

/// Convert JSON value to SettingValue
fn json_to_setting_value(value: serde_json::Value) -> Option<super::types::SettingValue> {
    use super::types::SettingValue;
    use serde_json::Value;

    match value {
        Value::Bool(b) => Some(SettingValue::Bool(b)),
        Value::String(s) => Some(SettingValue::String(s)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(SettingValue::Number(i))
            } else if let Some(f) = n.as_f64() {
                Some(SettingValue::Float(f))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Mark setup as completed
pub fn complete_setup(app: &tauri::AppHandle) -> Result<(), AppError> {
    let mut settings = load_settings(app)?;
    settings.setup_completed = true;
    settings.accepted_license = true;
    save_settings(app, &settings)
}

/// Reset all settings to defaults
pub fn reset_settings(app: &tauri::AppHandle) -> Result<AppSettings, AppError> {
    let mut settings = load_settings(app)?;
    settings.reset_all();
    save_settings(app, &settings)?;
    Ok(settings)
}

/// Reset a specific setting to its default
pub fn reset_setting(app: &tauri::AppHandle, key: &str) -> Result<AppSettings, AppError> {
    let mut settings = load_settings(app)?;
    if !settings.reset(key) {
        return Err(AppError::invalid_setting_key(key));
    }
    save_settings(app, &settings)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_setting_value() {
        use serde_json::json;

        assert!(matches!(
            json_to_setting_value(json!(true)),
            Some(super::super::types::SettingValue::Bool(true))
        ));

        assert!(matches!(
            json_to_setting_value(json!("test")),
            Some(super::super::types::SettingValue::String(_))
        ));

        assert!(matches!(
            json_to_setting_value(json!(42)),
            Some(super::super::types::SettingValue::Number(42))
        ));
    }
}

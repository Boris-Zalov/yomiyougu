//! Device management commands

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use crate::error::AppError;

const STORE_FILENAME: &str = "device.json";
const DEVICE_ID_KEY: &str = "device_id";

/// Get or create a unique device ID
#[tauri::command]
pub async fn get_or_create_device_id(app: AppHandle) -> Result<String, String> {
    get_or_create_device_id_impl(&app).map_err(|e| e.into())
}

fn get_or_create_device_id_impl(app: &AppHandle) -> Result<String, AppError> {
    let store = app.store(STORE_FILENAME)
        .map_err(|e| AppError::config_read_failed(format!("Failed to open device store: {}", e)))?;
    
    // Try to get existing device ID
    if let Some(value) = store.get(DEVICE_ID_KEY) {
        if let Some(device_id) = value.as_str() {
            log::info!("Found existing device ID: {}", device_id);
            return Ok(device_id.to_string());
        }
    }
    
    // Generate a new device ID
    let device_id = uuid::Uuid::new_v4().to_string();
    log::info!("Generated new device ID: {}", device_id);
    
    // Save to store
    store.set(DEVICE_ID_KEY, serde_json::json!(device_id));
    store.save()
        .map_err(|e| AppError::config_read_failed(format!("Failed to save device ID: {}", e)))?;
    
    Ok(device_id)
}

/// Get the device ID without creating one (for internal use)
pub fn get_device_id(app: &AppHandle) -> Option<String> {
    let store = app.store(STORE_FILENAME).ok()?;
    store.get(DEVICE_ID_KEY)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

use serde::Serialize;
use std::fmt;

/// Application-wide error type for consistent error handling
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ConfigNotFound,
    ConfigReadFailed,
    ConfigWriteFailed,
    ConfigParseFailed,
    SerializationFailed,
    InvalidSettingKey,
    InvalidSettingValue,
    IoError,
    DatabaseNotInitialized,
    DatabaseConnectionFailed,
    DatabaseMigrationFailed,
    DatabaseQueryFailed,
    DatabasePathError,
    DatabaseError,
    DuplicateEntry,
    NotAuthenticated,
    SyncFailed,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn config_not_found() -> Self {
        Self::new(ErrorCode::ConfigNotFound, "Configuration file not found")
    }

    pub fn config_read_failed(err: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::ConfigReadFailed,
            format!("Failed to read config: {}", err),
        )
    }

    pub fn config_write_failed(err: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::ConfigWriteFailed,
            format!("Failed to write config: {}", err),
        )
    }

    pub fn config_parse_failed(err: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::ConfigParseFailed,
            format!("Failed to parse config: {}", err),
        )
    }

    pub fn serialization_failed(err: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::SerializationFailed,
            format!("Serialization error: {}", err),
        )
    }

    pub fn invalid_setting_key(key: &str) -> Self {
        Self::new(
            ErrorCode::InvalidSettingKey,
            format!("Invalid setting key: {}", key),
        )
    }

    pub fn invalid_setting_value(key: &str, reason: &str) -> Self {
        Self::new(
            ErrorCode::InvalidSettingValue,
            format!("Invalid value for '{}': {}", key, reason),
        )
    }

    pub fn not_authenticated() -> Self {
        Self::new(ErrorCode::NotAuthenticated, "Not authenticated with Google")
    }

    pub fn sync_failed(err: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::SyncFailed,
            format!("Sync failed: {}", err),
        )
    }

    pub fn database_error(err: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::DatabaseError,
            format!("Database error: {}", err),
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

// Convert to String for Tauri command returns
impl From<AppError> for String {
    fn from(err: AppError) -> String {
        serde_json::to_string(&err).unwrap_or(err.message)
    }
}

//! Sync data types for remote snapshot and merge operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// REMOTE STATE TYPES (stored in sync_snapshot.json on Google Drive)
// ============================================================================

/// Remote book state - synced metadata for a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBookState {
    pub uuid: String,
    pub file_hash: Option<String>,
    pub title: String,
    pub filename: String,
    pub current_page: i32,
    pub total_pages: i32,
    pub is_favorite: bool,
    pub reading_status: String,
    pub last_read_at: Option<i64>,  // Unix timestamp (millis)
    pub added_at: i64,               // Unix timestamp (millis)
    pub updated_at: i64,             // Unix timestamp (millis)
    pub deleted_at: Option<i64>,     // Unix timestamp (millis) - soft delete
}

/// Remote bookmark state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBookmarkState {
    pub uuid: String,
    pub book_uuid: String,  // Reference to parent book by UUID
    pub name: String,
    pub description: Option<String>,
    pub page: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

/// Remote collection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCollectionState {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

/// Remote book-collection relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBookCollectionState {
    pub uuid: String,
    pub book_uuid: String,
    pub collection_uuid: String,
    pub added_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

/// Remote book settings state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBookSettingsState {
    pub uuid: String,
    pub book_uuid: String,
    pub reading_direction: Option<String>,
    pub page_display_mode: Option<String>,
    pub image_fit_mode: Option<String>,
    pub reader_background: Option<String>,
    pub sync_progress: Option<bool>,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

/// The complete sync snapshot stored on Google Drive
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncSnapshot {
    /// Schema version for forward compatibility
    pub version: u32,
    /// Device ID that last modified this snapshot
    pub last_modified_by: Option<String>,
    /// When this snapshot was last modified (Unix timestamp millis)
    pub last_modified_at: i64,
    
    /// Books indexed by UUID
    pub books: HashMap<String, RemoteBookState>,
    /// Bookmarks indexed by UUID
    pub bookmarks: HashMap<String, RemoteBookmarkState>,
    /// Collections indexed by UUID
    pub collections: HashMap<String, RemoteCollectionState>,
    /// Book-collection relationships indexed by UUID
    pub book_collections: HashMap<String, RemoteBookCollectionState>,
    /// Book settings indexed by UUID
    pub book_settings: HashMap<String, RemoteBookSettingsState>,
    /// App settings (key-value pairs)
    #[serde(default)]
    pub app_settings: HashMap<String, serde_json::Value>,
    /// When app settings were last modified
    #[serde(default)]
    pub app_settings_updated_at: i64,
}

impl SyncSnapshot {
    pub const CURRENT_VERSION: u32 = 1;
    
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            last_modified_by: None,
            last_modified_at: 0,
            books: HashMap::new(),
            bookmarks: HashMap::new(),
            collections: HashMap::new(),
            book_collections: HashMap::new(),
            book_settings: HashMap::new(),
            app_settings: HashMap::new(),
            app_settings_updated_at: 0,
        }
    }
}

// ============================================================================
// SYNC STATUS TYPES (for frontend/UI)
// ============================================================================

/// Options controlling what gets synced based on user settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOptions {
    /// Sync books (metadata only, not files - see sync_books_files)
    pub sync_books: bool,
    /// Sync book files to Google Drive (actual comic files)
    pub sync_books_files: bool,
    /// Sync app settings
    pub sync_settings: bool,
    /// Sync reading progress (current_page, last_read_at, bookmarks)
    pub sync_progress: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            sync_books: false,
            sync_books_files: false,
            sync_settings: false,
            sync_progress: true,  // Progress is on by default
        }
    }
}

/// Current sync status for display in UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    /// Never synced
    NeverSynced,
    /// Currently syncing
    Syncing,
    /// Last sync was successful
    Synced { last_sync_at: i64 },
    /// Last sync failed
    Failed { error: String, last_attempt_at: i64 },
    /// Sync is disabled (user not authenticated)
    Disabled,
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub books_uploaded: usize,
    pub books_downloaded: usize,
    pub bookmarks_uploaded: usize,
    pub bookmarks_downloaded: usize,
    pub collections_uploaded: usize,
    pub collections_downloaded: usize,
    pub conflicts_resolved: usize,
    pub errors: Vec<String>,
    pub completed_at: i64,
}

impl SyncResult {
    pub fn empty() -> Self {
        Self {
            success: true,
            books_uploaded: 0,
            books_downloaded: 0,
            bookmarks_uploaded: 0,
            bookmarks_downloaded: 0,
            collections_uploaded: 0,
            collections_downloaded: 0,
            conflicts_resolved: 0,
            errors: Vec::new(),
            completed_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// Remote (server) always wins
    RemoteWins,
    /// Local always wins
    LocalWins,
    /// Most recent timestamp wins
    #[default]
    LastWriteWins,
}

// ============================================================================
// TIMESTAMP HELPERS
// ============================================================================

/// Convert chrono::NaiveDateTime to Unix timestamp (milliseconds)
pub fn to_timestamp(dt: &chrono::NaiveDateTime) -> i64 {
    dt.and_utc().timestamp_millis()
}

/// Convert Unix timestamp (milliseconds) to chrono::NaiveDateTime
pub fn from_timestamp(ts: i64) -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp_millis(ts)
        .map(|dt| dt.naive_utc())
        .unwrap_or_else(|| chrono::Utc::now().naive_utc())
}

/// Convert Option<chrono::NaiveDateTime> to Option<i64>
pub fn to_opt_timestamp(dt: &Option<chrono::NaiveDateTime>) -> Option<i64> {
    dt.as_ref().map(to_timestamp)
}

/// Convert Option<i64> to Option<chrono::NaiveDateTime>
pub fn from_opt_timestamp(ts: Option<i64>) -> Option<chrono::NaiveDateTime> {
    ts.map(from_timestamp)
}

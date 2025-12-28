//! Database models for Diesel

use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{book_collections, book_settings, bookmarks, books, collections, sync_state};

// ============================================================================
// COLLECTIONS
// ============================================================================

/// Collection model for organizing books into groups
#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = collections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub uuid: Option<String>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

/// New collection for insertion
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = collections)]
pub struct NewCollection {
    pub name: String,
    pub description: Option<String>,
    pub uuid: Option<String>,
}

/// Collection update (partial)
#[derive(Debug, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = collections)]
pub struct UpdateCollection {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// ============================================================================
// BOOK-COLLECTION JUNCTION
// ============================================================================

/// Junction table model for many-to-many book-collection relationship
#[derive(
    Debug, Clone, Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize,
)]
#[diesel(table_name = book_collections)]
#[diesel(belongs_to(Book))]
#[diesel(belongs_to(Collection))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BookCollection {
    pub id: i32,
    pub book_id: i32,
    pub collection_id: i32,
    pub added_at: chrono::NaiveDateTime,
    pub uuid: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

/// New book-collection relationship for insertion
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = book_collections)]
pub struct NewBookCollection {
    pub book_id: i32,
    pub collection_id: i32,
    pub uuid: Option<String>,
}

// ============================================================================
// BOOKS
// ============================================================================

/// Reading status enum matching database constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingStatus {
    Unread,
    Reading,
    Completed,
    OnHold,
    Dropped,
}

impl ReadingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReadingStatus::Unread => "unread",
            ReadingStatus::Reading => "reading",
            ReadingStatus::Completed => "completed",
            ReadingStatus::OnHold => "on_hold",
            ReadingStatus::Dropped => "dropped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "unread" => Some(ReadingStatus::Unread),
            "reading" => Some(ReadingStatus::Reading),
            "completed" => Some(ReadingStatus::Completed),
            "on_hold" => Some(ReadingStatus::OnHold),
            "dropped" => Some(ReadingStatus::Dropped),
            _ => None,
        }
    }
}

/// Book model for manga/comics
#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = books)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Book {
    pub id: i32,
    pub file_path: String,
    pub filename: String,
    pub file_size: Option<i32>,
    pub file_hash: Option<String>,
    pub title: String,
    pub current_page: i32,
    pub total_pages: i32,
    pub last_read_at: Option<chrono::NaiveDateTime>,
    pub added_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub is_favorite: bool,
    pub reading_status: String,
    pub uuid: Option<String>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl Book {
    /// Get reading status as enum
    pub fn status(&self) -> ReadingStatus {
        ReadingStatus::from_str(&self.reading_status).unwrap_or(ReadingStatus::Unread)
    }

    /// Calculate reading progress as percentage
    pub fn progress(&self) -> f32 {
        if self.total_pages == 0 {
            0.0
        } else {
            (self.current_page as f32 / self.total_pages as f32) * 100.0
        }
    }
}

/// New book for insertion
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = books)]
pub struct NewBook {
    pub file_path: String,
    pub filename: String,
    pub file_size: Option<i32>,
    pub file_hash: Option<String>,
    pub title: String,
    pub total_pages: i32,
    pub uuid: Option<String>,
}

/// Book update (partial)
#[derive(Debug, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = books)]
pub struct UpdateBook {
    pub title: Option<String>,
    pub current_page: Option<i32>,
    pub total_pages: Option<i32>,
    pub last_read_at: Option<Option<chrono::NaiveDateTime>>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub is_favorite: Option<bool>,
    pub reading_status: Option<String>,
}

// ============================================================================
// BOOKMARKS
// ============================================================================

/// Bookmark model for saving specific pages
#[derive(
    Debug, Clone, Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize,
)]
#[diesel(table_name = bookmarks)]
#[diesel(belongs_to(Book))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Bookmark {
    pub id: i32,
    pub book_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub page: i32,
    pub created_at: chrono::NaiveDateTime,
    pub uuid: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

/// New bookmark for insertion
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = bookmarks)]
pub struct NewBookmark {
    pub book_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub page: i32,
    pub uuid: Option<String>,
}

// ============================================================================
// BOOK SETTINGS
// ============================================================================

/// Reading direction options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingDirection {
    Ltr,
    Rtl,
    Vertical,
}

impl ReadingDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReadingDirection::Ltr => "ltr",
            ReadingDirection::Rtl => "rtl",
            ReadingDirection::Vertical => "vertical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ltr" => Some(ReadingDirection::Ltr),
            "rtl" => Some(ReadingDirection::Rtl),
            "vertical" => Some(ReadingDirection::Vertical),
            _ => None,
        }
    }
}

/// Page display mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageDisplayMode {
    Single,
    Double,
    Auto,
}

/// Image fit mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFitMode {
    FitWidth,
    FitHeight,
    FitScreen,
    Original,
}

/// Book-specific settings overrides
#[derive(
    Debug, Clone, Queryable, Identifiable, Selectable, Associations, Serialize, Deserialize,
)]
#[diesel(table_name = book_settings)]
#[diesel(belongs_to(Book))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BookSettings {
    pub id: i32,
    pub book_id: i32,
    pub reading_direction: Option<String>,
    pub page_display_mode: Option<String>,
    pub image_fit_mode: Option<String>,
    pub sync_progress: Option<bool>,
    pub updated_at: chrono::NaiveDateTime,
    pub uuid: Option<String>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

/// New book settings for insertion
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = book_settings)]
pub struct NewBookSettings {
    pub book_id: i32,
    pub reading_direction: Option<String>,
    pub page_display_mode: Option<String>,
    pub image_fit_mode: Option<String>,
    pub sync_progress: Option<bool>,
    pub uuid: Option<String>,
}

/// Book settings update (partial)
#[derive(Debug, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = book_settings)]
pub struct UpdateBookSettings {
    pub reading_direction: Option<Option<String>>,
    pub page_display_mode: Option<Option<String>>,
    pub image_fit_mode: Option<Option<String>>,
    pub sync_progress: Option<Option<bool>>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// ============================================================================
// DTOs for frontend
// ============================================================================

/// Book with its settings and collection names
#[derive(Debug, Serialize, Deserialize)]
pub struct BookWithDetails {
    #[serde(flatten)]
    pub book: Book,
    pub collection_names: Vec<String>,
    pub collection_ids: Vec<i32>,
    pub settings: Option<BookSettings>,
    pub bookmark_count: i64,
}

/// Collection with book count
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionWithCount {
    #[serde(flatten)]
    pub collection: Collection,
    pub book_count: i64,
}

// ============================================================================
// SYNC STATE
// ============================================================================

/// Sync state tracking for synchronization
#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = sync_state)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SyncState {
    pub id: i32,
    pub last_sync_at: Option<chrono::NaiveDateTime>,
    pub last_sync_device: Option<String>,
    pub sync_file_id: Option<String>,
}

/// Sync state update
#[derive(Debug, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = sync_state)]
pub struct UpdateSyncState {
    pub last_sync_at: Option<Option<chrono::NaiveDateTime>>,
    pub last_sync_device: Option<Option<String>>,
    pub sync_file_id: Option<Option<String>>,
}

//! Library management commands for Tauri frontend
//!
//! Provides commands for managing books and collections

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::database::models::{
    Book, BookWithDetails, Collection, CollectionWithCount, ImportResult, NewCollection,
    UpdateBook, UpdateCollection,
};
use crate::database::operations;
use crate::error::AppError;
use crate::settings::storage;

// ============================================================================
// COLLECTION COMMANDS
// ============================================================================

/// Create a new collection
#[tauri::command]
pub async fn create_collection(
    name: String,
    description: Option<String>,
    cover_path: Option<String>,
) -> Result<Collection, String> {
    let new_collection = NewCollection {
        name,
        description,
        cover_path,
    };

    operations::create_collection(new_collection).map_err(|e| e.into())
}

/// Get all collections with book counts
#[tauri::command]
pub async fn get_collections() -> Result<Vec<CollectionWithCount>, String> {
    operations::get_all_collections().map_err(|e| e.into())
}

/// Get a single collection by ID
#[tauri::command]
pub async fn get_collection(collection_id: i32) -> Result<Collection, String> {
    operations::get_collection_by_id(collection_id).map_err(|e| e.into())
}

/// Update a collection
#[tauri::command]
pub async fn update_collection(
    collection_id: i32,
    name: Option<String>,
    description: Option<Option<String>>,
    cover_path: Option<Option<String>>,
) -> Result<Collection, String> {
    let updates = UpdateCollection {
        name,
        description,
        cover_path,
        updated_at: None,
    };

    operations::update_collection(collection_id, updates).map_err(|e| e.into())
}

/// Delete a collection
#[tauri::command]
pub async fn delete_collection(collection_id: i32) -> Result<(), String> {
    operations::delete_collection(collection_id).map_err(|e| e.into())
}

// ============================================================================
// BOOK COMMANDS
// ============================================================================

/// Get all books with optional filtering
#[tauri::command]
pub async fn get_books(
    collection_id: Option<i32>,
    status: Option<String>,
    favorites_only: bool,
) -> Result<Vec<BookWithDetails>, String> {
    operations::get_all_books(collection_id, status, favorites_only).map_err(|e| e.into())
}

/// Get a single book by ID
#[tauri::command]
pub async fn get_book(book_id: i32) -> Result<Book, String> {
    operations::get_book_by_id(book_id).map_err(|e| e.into())
}

/// Update a book
#[tauri::command]
pub async fn update_book(
    book_id: i32,
    title: Option<String>,
    current_page: Option<i32>,
    collection_id: Option<Option<i32>>,
    is_favorite: Option<bool>,
    reading_status: Option<String>,
) -> Result<Book, String> {
    let updates = UpdateBook {
        title,
        current_page,
        total_pages: None,
        last_read_at: if current_page.is_some() {
            Some(Some(chrono::Utc::now().naive_utc()))
        } else {
            None
        },
        updated_at: None,
        collection_id,
        is_favorite,
        reading_status,
    };

    operations::update_book(book_id, updates).map_err(|e| e.into())
}

/// Delete a book
#[tauri::command]
pub async fn delete_book(book_id: i32) -> Result<(), String> {
    operations::delete_book(book_id).map_err(|e| e.into())
}

/// Import books from a zip/cbz archive file
#[tauri::command]
pub async fn import_books_from_archive(
    app: AppHandle,
    file_path: String,
    collection_id: Option<i32>,
) -> Result<ImportResult, String> {
    let archive_path = PathBuf::from(&file_path);

    if !archive_path.exists() {
        return Err("File does not exist".into());
    }

    let ext = archive_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    if !matches!(ext.as_deref(), Some("zip") | Some("cbz")) {
        return Err("Only .zip and .cbz files are supported".into());
    }

    let settings = storage::load_settings(&app).map_err(|e: AppError| e)?;
    let backup_files = settings
        .get("library.backup_imported_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let library_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("library");

    if backup_files {
        std::fs::create_dir_all(&library_dir)
            .map_err(|e| format!("Failed to create library directory: {}", e))?;
    }

    operations::import_books_from_archive(&archive_path, collection_id, backup_files, &library_dir)
        .map_err(|e| e.into())
}
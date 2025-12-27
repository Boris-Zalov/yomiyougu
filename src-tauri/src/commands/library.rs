//! Library management commands for Tauri frontend
//!
//! Provides commands for managing books and collections

use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_fs::FsExt;

use crate::database::models::{
    Book, BookWithDetails, Collection, CollectionWithCount, NewCollection, UpdateBook,
    UpdateCollection,
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
) -> Result<Collection, String> {
    let new_collection = NewCollection { name, description };

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
) -> Result<Collection, String> {
    let updates = UpdateCollection {
        name,
        description,
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
        is_favorite,
        reading_status,
    };

    operations::update_book(book_id, updates).map_err(|e| e.into())
}

/// Set the collections for a book (replaces existing)
#[tauri::command]
pub async fn set_book_collections(book_id: i32, collection_ids: Vec<i32>) -> Result<(), String> {
    operations::set_book_collections(book_id, collection_ids).map_err(|e| e.into())
}

/// Add a book to a collection
#[tauri::command]
pub async fn add_book_to_collection(book_id: i32, collection_id: i32) -> Result<(), String> {
    operations::add_book_to_collection(book_id, collection_id)
        .map(|_| ())
        .map_err(|e| e.into())
}

/// Remove a book from a collection
#[tauri::command]
pub async fn remove_book_from_collection(book_id: i32, collection_id: i32) -> Result<(), String> {
    operations::remove_book_from_collection(book_id, collection_id).map_err(|e| e.into())
}

/// Delete a book
#[tauri::command]
pub async fn delete_book(book_id: i32) -> Result<(), String> {
    operations::delete_book(book_id).map_err(|e| e.into())
}

/// Import a single book from a zip/cbz/rar/cbr archive file
/// Each archive is treated as a single book regardless of internal structure
#[tauri::command]
pub async fn import_book_from_archive(
    app: AppHandle,
    file_path: String,
    collection_id: Option<i32>,
    original_filename: Option<String>,
) -> Result<Book, String> {
    use std::io::{Read, Write};

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

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get app cache directory: {}", e))?;

    log::info!("Library dir: {:?}, Cache dir: {:?}", library_dir, cache_dir);

    // Determine if this is an Android content URI or a regular file path
    let is_content_uri = file_path.starts_with("content://");

    // Get the actual file path to process
    let (archive_path, temp_file_path) = if is_content_uri {
        log::info!(
            "Processing Android content URI: {}",
            &file_path[..80.min(file_path.len())]
        );

        log::info!("Creating cache directory: {:?}", cache_dir);
        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            return Err(format!(
                "Failed to create cache directory {:?}: {}",
                cache_dir, e
            ));
        }

        if !cache_dir.exists() {
            return Err(format!(
                "Cache directory doesn't exist after creation: {:?}",
                cache_dir
            ));
        }
        log::info!("Cache directory verified: {:?}", cache_dir);

        let fs_scope = app.fs_scope();

        fs_scope
            .allow_file(&file_path)
            .map_err(|e| format!("Failed to allow file access: {}", e))?;

        let file_url = tauri::Url::parse(&file_path)
            .map_err(|e| format!("Failed to parse content URI: {}", e))?;

        log::debug!("Opening content URI...");
        let mut file = app
            .fs()
            .open(
                file_url,
                tauri_plugin_fs::OpenOptions::new().read(true).clone(),
            )
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| format!("Failed to read file content: {}", e))?;

        log::info!("Read {} bytes from content URI", content.len());

        let filename = original_filename
            .clone()
            .unwrap_or_else(|| format!("import_{}.cbz", chrono::Utc::now().timestamp_millis()));
        let temp_path = cache_dir.join(&filename);

        log::debug!("Writing to temp file: {:?}", temp_path);

        let mut temp_file = std::fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file at {:?}: {}", temp_path, e))?;
        temp_file
            .write_all(&content)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        log::info!("Copied content URI to temp file: {:?}", temp_path);

        (temp_path.clone(), Some(temp_path))
    } else {
        // Regular file path (desktop)
        let path = PathBuf::from(&file_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }
        (path, None)
    };

    let ext = archive_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    if !matches!(
        ext.as_deref(),
        Some("zip") | Some("cbz") | Some("rar") | Some("cbr")
    ) {
        // Clean up temp file
        if let Some(ref temp_path) = temp_file_path {
            let _ = std::fs::remove_file(temp_path);
        }
        return Err("Only .zip, .cbz, .rar, and .cbr files are supported".into());
    }

    if backup_files {
        std::fs::create_dir_all(&library_dir)
            .map_err(|e| format!("Failed to create library directory: {}", e))?;
    }

    // Run blocking I/O operations on a separate thread
    let result = tauri::async_runtime::spawn_blocking(move || {
        operations::import_book_from_archive(
            &archive_path,
            collection_id,
            backup_files,
            &library_dir,
            original_filename,
        )
        .map_err(|e| e.into())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;

    if let Some(temp_path) = temp_file_path {
        if !backup_files {
            let _ = std::fs::remove_file(&temp_path);
            log::debug!("Cleaned up temp file: {:?}", temp_path);
        }
    }

    result
}

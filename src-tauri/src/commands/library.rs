//! Library management commands for Tauri frontend
//!
//! Provides commands for managing books and collections

use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_fs::FsExt;

use crate::database::models::{
    Book, BookSettings, BookWithDetails, Collection, CollectionWithCount, NewCollection, UpdateBook,
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
    let new_collection = NewCollection { 
        name, 
        description,
        uuid: Some(uuid::Uuid::new_v4().to_string()),
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

/// Delete a book - removes local files (if in appdata) and queues cloud file deletion
#[tauri::command]
pub async fn delete_book(app: AppHandle, book_id: i32) -> Result<(), String> {
    delete_book_impl(&app, book_id).await.map_err(|e| e.into())
}

async fn delete_book_impl(app: &AppHandle, book_id: i32) -> Result<(), AppError> {
    use crate::auth;
    use crate::sync::DriveSync;
    
    // Get the book first to access its file path and hash
    let book = operations::get_book_by_id(book_id)?;
    
    // Get the app data directory to check if the file is stored within it
    let app_data_dir = app.path()
        .app_data_dir()
        .ok();
    
    // Delete local file only if it's stored in app data (not external reference)
    if !book.file_path.starts_with("cloud://") {
        let path = std::path::Path::new(&book.file_path);
        
        // Only delete if the file is within app data directory
        let should_delete = app_data_dir
            .as_ref()
            .map(|app_dir| path.starts_with(app_dir))
            .unwrap_or(false);
        
        if should_delete && path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                log::warn!("Failed to delete local file {:?}: {}", path, e);
                // Continue with deletion - don't fail if local file can't be removed
            } else {
                log::info!("Deleted local file: {:?}", path);
            }
        } else if !should_delete {
            log::info!("Keeping external file (not in app data): {:?}", path);
        }
    }
    
    // Try to delete from cloud if user is authenticated and book has a hash
    if let Some(ref file_hash) = book.file_hash {
        if let Ok(auth_status) = auth::get_auth_status(app) {
            if auth_status.is_authenticated {
                // Try to get a valid access token
                if let Ok(token) = auth::load_token(app) {
                    let access_token = if token.is_expired() {
                        // Try to refresh the token
                        if let (Ok(client_id), Ok(client_secret)) = (
                            std::env::var("VITE_GOOGLE_CLIENT_ID"),
                            std::env::var("VITE_GOOGLE_CLIENT_SECRET"),
                        ) {
                            match crate::commands::auth::refresh_token_internal(&client_id, &client_secret, &token).await {
                                Ok(new_token) => {
                                    let _ = auth::save_token(app, &new_token);
                                    Some(new_token.access_token)
                                }
                                Err(e) => {
                                    log::warn!("Failed to refresh token for cloud deletion: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        Some(token.access_token)
                    };
                    
                    if let Some(access_token) = access_token {
                        let drive = DriveSync::with_token(access_token);
                        match drive.delete_book_file(file_hash).await {
                            Ok(deleted) => {
                                if deleted {
                                    log::info!("Deleted cloud file for book {}", book_id);
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to delete cloud file for book {}: {}", book_id, e);
                                // Continue - cloud deletion failure shouldn't prevent local deletion
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Soft-delete the book from database
    operations::delete_book(book_id)?;
    
    Ok(())
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
    let save_to_app_storage = settings
        .get("library.save_to_app_storage")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Get default reading settings to apply to the new book
    let default_reading_direction = settings
        .get("reading.direction")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());
    let default_page_display_mode = settings
        .get("reading.page_display_mode")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());
    let default_image_fit_mode = settings
        .get("reading.image_fit_mode")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());

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

    // For Android content URIs, file MUST be saved to app storage because the cache is temporary
    // and content URIs can't be referenced later (system restriction)
    let effective_save_to_storage = if is_content_uri { true } else { save_to_app_storage };

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

    if effective_save_to_storage {
        std::fs::create_dir_all(&library_dir)
            .map_err(|e| format!("Failed to create library directory: {}", e))?;
    }

    // Run blocking I/O operations on a separate thread
    let result = tauri::async_runtime::spawn_blocking(move || {
        operations::import_book_from_archive(
            &archive_path,
            collection_id,
            effective_save_to_storage,
            &library_dir,
            original_filename,
        )
        .map_err(|e| e.into())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;

    if let Some(temp_path) = temp_file_path {
        let _ = std::fs::remove_file(&temp_path);
        log::debug!("Cleaned up temp file: {:?}", temp_path);
    }

    // If import was successful, create default book settings
    if let Ok(ref book) = result {
        // Only create settings if we have non-default values from app settings
        let has_custom_defaults = default_reading_direction.is_some() 
            || default_page_display_mode.is_some()
            || default_image_fit_mode.is_some();
        
        if has_custom_defaults {
            if let Err(e) = operations::update_book_settings(
                book.id,
                default_reading_direction.map(Some),
                default_page_display_mode.map(Some),
                default_image_fit_mode.map(Some),
                None, // sync_progress - use global default
            ) {
                log::warn!("Failed to create default book settings for book {}: {}", book.id, e);
            }
        }
    }

    result
}

// ============================================================================
// BOOK SETTINGS COMMANDS
// ============================================================================

/// Get book settings by book ID
#[tauri::command]
pub async fn get_book_settings(book_id: i32) -> Result<Option<BookSettings>, String> {
    operations::get_book_settings(book_id).map_err(|e| e.into())
}

/// Update book settings (creates if not exists)
#[tauri::command]
pub async fn update_book_settings(
    book_id: i32,
    reading_direction: Option<Option<String>>,
    page_display_mode: Option<Option<String>>,
    image_fit_mode: Option<Option<String>>,
    sync_progress: Option<Option<bool>>,
) -> Result<BookSettings, String> {
    operations::update_book_settings(
        book_id,
        reading_direction,
        page_display_mode,
        image_fit_mode,
        sync_progress,
    )
    .map_err(|e| e.into())
}

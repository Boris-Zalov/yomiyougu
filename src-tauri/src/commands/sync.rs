//! Sync-related Tauri commands

use tauri::{AppHandle, Manager};

use crate::auth;
use crate::commands::device::get_device_id;
use crate::error::AppError;
use crate::settings::{load_settings, SettingValue};
use crate::sync::{DriveSync, MergeEngine, SyncOptions, SyncResult, SyncStatus, ConflictStrategy};

#[tauri::command]
pub fn get_sync_status(app: AppHandle) -> Result<SyncStatus, String> {
    get_sync_status_impl(&app).map_err(|e| e.into())
}

fn get_sync_status_impl(app: &AppHandle) -> Result<SyncStatus, AppError> {
    // Check if user is authenticated
    let auth_status = auth::get_auth_status(app)?;
    
    if !auth_status.is_authenticated {
        return Ok(SyncStatus::Disabled);
    }

    // Get last sync from database
    use diesel::prelude::*;
    use crate::database::get_connection;
    use crate::schema::sync_state;
    use crate::database::models::SyncState;

    let mut conn = get_connection()?;
    let state: Option<SyncState> = sync_state::table
        .find(1)
        .first(&mut conn)
        .optional()
        .map_err(|e| AppError::database_error(e.to_string()))?;

    match state.and_then(|s| s.last_sync_at) {
        Some(last_sync) => Ok(SyncStatus::Synced { 
            last_sync_at: last_sync.and_utc().timestamp_millis() 
        }),
        None => Ok(SyncStatus::NeverSynced),
    }
}

/// Trigger a manual sync
#[tauri::command]
pub async fn sync_now(app: AppHandle) -> Result<SyncResult, String> {
    sync_now_impl(&app).await.map_err(|e| e.into())
}

async fn sync_now_impl(app: &AppHandle) -> Result<SyncResult, AppError> {
    log::info!("Starting manual sync...");
    
    // Check authentication
    let auth_status = auth::get_auth_status(app)?;
    if !auth_status.is_authenticated {
        return Err(AppError::not_authenticated());
    }

    // Load sync options from user settings
    let settings = load_settings(app)?;
    let sync_options = SyncOptions {
        sync_books: matches!(settings.get("sync.books"), Some(SettingValue::Bool(true))),
        sync_books_files: matches!(settings.get("sync.books"), Some(SettingValue::Bool(true))),
        sync_settings: matches!(settings.get("sync.settings"), Some(SettingValue::Bool(true))),
        sync_progress: matches!(settings.get("sync.progress"), Some(SettingValue::Bool(true))),
    };

    log::info!(
        "Sync options: books={}, files={}, settings={}, progress={}",
        sync_options.sync_books,
        sync_options.sync_books_files,
        sync_options.sync_settings,
        sync_options.sync_progress
    );

    // Check if anything is enabled to sync
    if !sync_options.sync_books && !sync_options.sync_settings && !sync_options.sync_progress {
        log::warn!("No sync options enabled, nothing to sync");
        return Ok(SyncResult::empty());
    }

    // Check if token needs refresh
    let token = auth::load_token(app)?;
    
    log::info!(
        "Token status: is_expired={}, can_refresh={}, has_refresh_token={}",
        token.is_expired(),
        token.can_refresh(),
        token.refresh_token.is_some()
    );
    
    let access_token = if token.is_expired() {
        if !token.can_refresh() {
            log::error!("Access token expired and no refresh token available - user needs to re-authenticate");
            return Err(AppError::not_authenticated());
        }
        
        // Refresh the token
        log::info!("Access token expired, attempting refresh...");
        let client_id = std::env::var("VITE_GOOGLE_CLIENT_ID")
            .map_err(|_| AppError::config_read_failed("VITE_GOOGLE_CLIENT_ID not set"))?;
        let client_secret = std::env::var("VITE_GOOGLE_CLIENT_SECRET")
            .map_err(|_| AppError::config_read_failed("VITE_GOOGLE_CLIENT_SECRET not set"))?;
        
        match crate::commands::auth::refresh_token_internal(&client_id, &client_secret, &token).await {
            Ok(new_token) => {
                log::info!(
                    "Token refreshed successfully, new expiration: {:?}",
                    new_token.expires_at
                );
                auth::save_token(app, &new_token)?;
                new_token.access_token
            }
            Err(e) => {
                log::error!("Failed to refresh token: {:?}", e);
                // Clear the stored token so user knows they need to re-auth
                if let Err(clear_err) = auth::clear_token(app) {
                    log::warn!("Failed to clear invalid token: {:?}", clear_err);
                }
                return Err(AppError::sync_failed(format!(
                    "Your Google Drive access has expired. Please sign in again. ({})", e
                )));
            }
        }
    } else {
        token.access_token
    };

    let drive = DriveSync::with_token(access_token.clone());
    
    // Download remote snapshot
    log::info!("Downloading remote snapshot...");
    let remote_snapshot = drive.download_snapshot().await?;
    let existing_file_id = drive.find_sync_file().await?;
    
    // Merge local and remote
    log::info!("Merging local and remote data...");
    let device_id = get_device_id(app).unwrap_or_else(|| format!("device-{}", uuid::Uuid::new_v4()));
    let engine = MergeEngine::new(device_id, ConflictStrategy::default(), sync_options.clone());
    let (updated_snapshot, mut result) = engine.sync(app, remote_snapshot)?;
    
    // Upload updated snapshot
    log::info!("Uploading updated snapshot...");
    let file_id = drive.upload_snapshot(&updated_snapshot, existing_file_id.as_deref()).await?;
    
    // Sync book files if enabled
    if sync_options.sync_books_files {
        log::info!("Syncing book files...");
        sync_book_files(app, &drive, &updated_snapshot, &mut result).await?;
    }
    
    // Save file ID to local state
    use diesel::prelude::*;
    use crate::database::get_connection;
    use crate::schema::sync_state;

    let mut conn = get_connection()?;
    diesel::update(sync_state::table.find(1))
        .set(sync_state::sync_file_id.eq(Some(&file_id)))
        .execute(&mut conn)
        .map_err(|e| AppError::database_error(e.to_string()))?;

    log::info!(
        "Sync completed: {} books up, {} books down, {} bookmarks up, {} bookmarks down",
        result.books_uploaded,
        result.books_downloaded,
        result.bookmarks_uploaded,
        result.bookmarks_downloaded
    );

    Ok(result)
}

/// Sync book files between local storage and Google Drive
/// Only uploads local files to Drive - downloads happen on-demand when user tries to read
async fn sync_book_files(
    _app: &AppHandle,
    drive: &DriveSync,
    _snapshot: &crate::sync::SyncSnapshot,
    result: &mut SyncResult,
) -> Result<(), AppError> {
    use crate::database::get_connection;
    use crate::schema::books;
    use diesel::prelude::*;
    use crate::database::models::Book;
    
    let mut conn = get_connection()?;
    
    // Get all local books with file_hash (non-deleted)
    let local_books: Vec<Book> = books::table
        .filter(books::deleted_at.is_null())
        .filter(books::file_hash.is_not_null())
        .load(&mut conn)
        .map_err(|e| AppError::database_error(e.to_string()))?;
    
    // Get list of files already on Drive
    let remote_files = drive.list_book_files().await?;
    log::info!("Found {} book files on Drive", remote_files.len());
    let remote_hashes: std::collections::HashSet<String> = remote_files
        .iter()
        .map(|f| f.file_hash.clone())
        .collect();
    
    // Upload local books that aren't on Drive yet
    for book in &local_books {
        if let Some(ref file_hash) = book.file_hash {
            if !remote_hashes.contains(file_hash) {
                // Check if the local file exists
                if std::path::Path::new(&book.file_path).exists() {
                    log::info!("Uploading book file: {} ({})", book.title, file_hash);
                    match drive.upload_book_file(&book.file_path, file_hash).await {
                        Ok(_) => {
                            result.books_uploaded += 1;
                        }
                        Err(e) => {
                            log::error!("Failed to upload book {}: {}", book.title, e);
                            result.errors.push(format!("Failed to upload {}: {}", book.title, e));
                        }
                    }
                } else {
                    log::warn!("Book file not found locally: {}", book.file_path);
                }
            }
        }
    }
    
    // Note: Downloads happen on-demand when user tries to read a cloud:// book
    // Books synced from other devices will have cloud://{uuid} paths until downloaded
    
    Ok(())
}

/// Download a cloud-only book file from Google Drive
/// This is called when user tries to read a book that has cloud:// file path
#[tauri::command]
pub async fn download_cloud_book(app: AppHandle, book_id: i32) -> Result<crate::database::models::Book, String> {
    download_cloud_book_impl(&app, book_id).await.map_err(|e| e.into())
}

async fn download_cloud_book_impl(app: &AppHandle, book_id: i32) -> Result<crate::database::models::Book, AppError> {
    use crate::database::get_connection;
    use crate::database::models::Book;
    use crate::schema::books;
    use diesel::prelude::*;

    log::info!("Downloading cloud book with id: {}", book_id);

    // Get the book from database
    let mut conn = get_connection()?;
    let book: Book = books::table
        .find(book_id)
        .first(&mut conn)
        .map_err(|e| AppError::database_error(format!("Book not found: {}", e)))?;

    // Verify it's a cloud-only book
    if !book.file_path.starts_with("cloud://") {
        return Err(AppError::sync_failed("Book is not cloud-only, file already exists locally"));
    }

    // Get file_hash - required for cloud books
    let file_hash = book.file_hash.as_ref()
        .ok_or_else(|| AppError::sync_failed("Cloud book missing file_hash"))?;

    // Check authentication
    let auth_status = auth::get_auth_status(app)?;
    if !auth_status.is_authenticated {
        return Err(AppError::not_authenticated());
    }

    // Get access token (refresh if needed)
    let token = auth::load_token(app)?;
    let access_token = if token.is_expired() {
        if !token.can_refresh() {
            return Err(AppError::not_authenticated());
        }
        
        let client_id = std::env::var("VITE_GOOGLE_CLIENT_ID")
            .map_err(|_| AppError::config_read_failed("VITE_GOOGLE_CLIENT_ID not set"))?;
        let client_secret = std::env::var("VITE_GOOGLE_CLIENT_SECRET")
            .map_err(|_| AppError::config_read_failed("VITE_GOOGLE_CLIENT_SECRET not set"))?;
        
        match crate::commands::auth::refresh_token_internal(&client_id, &client_secret, &token).await {
            Ok(new_token) => {
                auth::save_token(app, &new_token)?;
                new_token.access_token
            }
            Err(e) => {
                return Err(AppError::sync_failed(format!("Token refresh failed: {}", e)));
            }
        }
    } else {
        token.access_token
    };

    let drive = DriveSync::with_token(access_token);

    // Determine local storage path
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| AppError::config_read_failed(format!("Failed to get app data dir: {}", e)))?;
    
    let books_dir = app_data_dir.join("books");
    std::fs::create_dir_all(&books_dir)
        .map_err(|e| AppError::sync_failed(format!("Failed to create books directory: {}", e)))?;

    // Use the original filename for the local file
    let target_path = books_dir.join(&book.filename);
    let target_path_str = target_path.to_string_lossy().to_string();

    log::info!("Downloading book to: {}", target_path_str);

    // Download the file
    drive.download_book_file(file_hash, &target_path_str).await?;

    // Update the book's file_path in the database
    diesel::update(books::table.find(book_id))
        .set((
            books::file_path.eq(&target_path_str),
            books::updated_at.eq(chrono::Utc::now().naive_utc()),
        ))
        .execute(&mut conn)
        .map_err(|e| AppError::database_error(format!("Failed to update book path: {}", e)))?;

    // Return the updated book
    let updated_book: Book = books::table
        .find(book_id)
        .first(&mut conn)
        .map_err(|e| AppError::database_error(format!("Failed to fetch updated book: {}", e)))?;

    log::info!("Successfully downloaded cloud book: {}", updated_book.title);

    Ok(updated_book)
}

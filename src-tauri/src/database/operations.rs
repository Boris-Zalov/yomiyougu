//! Database operations for books and collections
//!
//! Provides CRUD operations and business logic for library management

use diesel::prelude::*;
use log::{debug, error, info, warn};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::database::connection::establish_connection;
use crate::database::models::*;
use crate::error::{AppError, ErrorCode};
use crate::schema::{book_collections, book_settings, bookmarks, books, collections};

// ============================================================================
// COLLECTIONS
// ============================================================================

/// Create a new collection
pub fn create_collection(new_collection: NewCollection) -> Result<Collection, AppError> {
    info!("Creating new collection: {}", new_collection.name);
    let mut conn = establish_connection()?;

    diesel::insert_into(collections::table)
        .values(&new_collection)
        .returning(Collection::as_returning())
        .get_result(&mut conn)
        .map(|collection: Collection| {
            info!(
                "Collection created successfully: {} (ID: {})",
                collection.name, collection.id
            );
            collection
        })
        .map_err(|e| {
            error!(
                "Failed to create collection '{}': {}",
                new_collection.name, e
            );
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to create collection: {}", e),
            )
        })
}

/// Get all collections with book counts (excludes soft-deleted)
pub fn get_all_collections() -> Result<Vec<CollectionWithCount>, AppError> {
    debug!("Fetching all collections with book counts");
    let mut conn = establish_connection()?;

    let collections_list = collections::table
        .filter(collections::deleted_at.is_null())
        .select(Collection::as_select())
        .load(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to load collections: {}", e),
            )
        })?;

    // Get book counts for each collection via junction table (exclude deleted books)
    let mut result = Vec::new();
    for collection in collections_list {
        let count = book_collections::table
            .inner_join(books::table)
            .filter(book_collections::collection_id.eq(collection.id))
            .filter(book_collections::deleted_at.is_null())
            .filter(books::deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)
            .unwrap_or(0);

        result.push(CollectionWithCount {
            collection,
            book_count: count,
        });
    }

    info!("Retrieved {} collections", result.len());
    Ok(result)
}

/// Get a single collection by ID (returns error if soft-deleted)
pub fn get_collection_by_id(collection_id: i32) -> Result<Collection, AppError> {
    let mut conn = establish_connection()?;

    collections::table
        .find(collection_id)
        .filter(collections::deleted_at.is_null())
        .select(Collection::as_select())
        .first(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to find collection: {}", e),
            )
        })
}

/// Update a collection
pub fn update_collection(
    collection_id: i32,
    updates: UpdateCollection,
) -> Result<Collection, AppError> {
    info!("Updating collection ID: {}", collection_id);
    let mut conn = establish_connection()?;

    let mut final_updates = updates;
    final_updates.updated_at = Some(chrono::Utc::now().naive_utc());

    diesel::update(collections::table.find(collection_id))
        .set(&final_updates)
        .returning(Collection::as_returning())
        .get_result(&mut conn)
        .map(|collection: Collection| {
            info!("Collection {} updated successfully", collection_id);
            collection
        })
        .map_err(|e| {
            error!("Failed to update collection {}: {}", collection_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to update collection: {}", e),
            )
        })
}

/// Delete a collection (soft delete - sets deleted_at)
/// Also modifies the name to avoid UNIQUE constraint conflicts with new collections
pub fn delete_collection(collection_id: i32) -> Result<(), AppError> {
    info!("Soft-deleting collection ID: {}", collection_id);
    let mut conn = establish_connection()?;

    let now = chrono::Utc::now().naive_utc();
    let timestamp = now.and_utc().timestamp();
    
    // Get current name to modify it
    let collection: Collection = collections::table
        .find(collection_id)
        .select(Collection::as_select())
        .first(&mut conn)
        .map_err(|e| {
            error!("Failed to find collection {}: {}", collection_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to find collection: {}", e),
            )
        })?;
    
    // Append deletion timestamp to name to free up the name for reuse
    let deleted_name = format!("{}__deleted_{}", collection.name, timestamp);
    
    diesel::update(collections::table.find(collection_id))
        .set((
            collections::name.eq(deleted_name),
            collections::deleted_at.eq(Some(now)),
            collections::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to delete collection {}: {}", collection_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to delete collection: {}", e),
            )
        })?;

    info!("Collection {} soft-deleted successfully", collection_id);
    Ok(())
}

// ============================================================================
// BOOKS
// ============================================================================

/// Create a new book
pub fn create_book(new_book: NewBook) -> Result<Book, AppError> {
    info!(
        "Creating new book: {} ({} pages)",
        new_book.title, new_book.total_pages
    );
    let mut conn = establish_connection()?;

    diesel::insert_into(books::table)
        .values(&new_book)
        .returning(Book::as_returning())
        .get_result(&mut conn)
        .map(|book: Book| {
            info!(
                "Book created successfully: {} (ID: {})",
                book.title, book.id
            );
            book
        })
        .map_err(|e| {
            error!("Failed to create book '{}': {}", new_book.title, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to create book: {}", e),
            )
        })
}

/// Get all books with optional filtering
pub fn get_all_books(
    collection_id: Option<i32>,
    status: Option<String>,
    favorites_only: bool,
) -> Result<Vec<BookWithDetails>, AppError> {
    debug!(
        "Fetching books - collection: {:?}, status: {:?}, favorites: {}",
        collection_id, status, favorites_only
    );
    let mut conn = establish_connection()?;

    // If filtering by collection, get book IDs from junction table first
    let book_ids_in_collection: Option<Vec<i32>> = if let Some(cid) = collection_id {
        Some(
            book_collections::table
                .filter(book_collections::collection_id.eq(cid))
                .filter(book_collections::deleted_at.is_null())
                .select(book_collections::book_id)
                .load(&mut conn)
                .map_err(|e| {
                    AppError::new(
                        ErrorCode::DatabaseQueryFailed,
                        format!("Failed to load book collection mappings: {}", e),
                    )
                })?,
        )
    } else {
        None
    };

    // Start with filter for non-deleted books
    let mut query = books::table
        .filter(books::deleted_at.is_null())
        .into_boxed();

    if let Some(ref ids) = book_ids_in_collection {
        query = query.filter(books::id.eq_any(ids));
    }

    if let Some(status_str) = status {
        query = query.filter(books::reading_status.eq(status_str));
    }

    if favorites_only {
        query = query.filter(books::is_favorite.eq(true));
    }

    let books_list = query
        .select(Book::as_select())
        .order(books::last_read_at.desc())
        .then_order_by(books::added_at.desc())
        .load(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to load books: {}", e),
            )
        })?;

    // Load collection names and IDs for each book
    let result: Vec<BookWithDetails> = books_list
        .into_iter()
        .map(|book| {
            let book_collections_data: Vec<(i32, String)> = book_collections::table
                .inner_join(collections::table)
                .filter(book_collections::book_id.eq(book.id))
                .select((collections::id, collections::name))
                .load(&mut conn)
                .unwrap_or_default();

            let collection_ids: Vec<i32> =
                book_collections_data.iter().map(|(id, _)| *id).collect();
            let collection_names: Vec<String> = book_collections_data
                .into_iter()
                .map(|(_, name)| name)
                .collect();

            BookWithDetails {
                book,
                collection_names,
                collection_ids,
                settings: None,
                bookmark_count: 0,
            }
        })
        .collect();

    info!("Retrieved {} books", result.len());
    Ok(result)
}

/// Get a single book by ID
pub fn get_book_by_id(book_id: i32) -> Result<Book, AppError> {
    let mut conn = establish_connection()?;

    books::table
        .find(book_id)
        .select(Book::as_select())
        .first(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to find book: {}", e),
            )
        })
}

/// Update a book
pub fn update_book(book_id: i32, updates: UpdateBook) -> Result<Book, AppError> {
    info!("Updating book ID: {}", book_id);
    let mut conn = establish_connection()?;

    let mut final_updates = updates;
    final_updates.updated_at = Some(chrono::Utc::now().naive_utc());

    diesel::update(books::table.find(book_id))
        .set(&final_updates)
        .returning(Book::as_returning())
        .get_result(&mut conn)
        .map(|book: Book| {
            info!("Book {} updated successfully", book_id);
            book
        })
        .map_err(|e| {
            error!("Failed to update book {}: {}", book_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to update book: {}", e),
            )
        })
}

/// Delete a book (soft delete - sets deleted_at)
pub fn delete_book(book_id: i32) -> Result<(), AppError> {
    info!("Soft-deleting book ID: {}", book_id);
    let mut conn = establish_connection()?;

    let now = chrono::Utc::now().naive_utc();
    
    diesel::update(books::table.find(book_id))
        .set((
            books::deleted_at.eq(Some(now)),
            books::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to delete book {}: {}", book_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to delete book: {}", e),
            )
        })?;

    info!("Book {} soft-deleted successfully", book_id);
    Ok(())
}

/// Check if a file hash already exists in the database (excludes soft-deleted)
pub fn find_book_by_hash(file_hash: &str) -> Result<Option<Book>, AppError> {
    let mut conn = establish_connection()?;

    books::table
        .filter(books::file_hash.eq(file_hash))
        .filter(books::deleted_at.is_null())
        .select(Book::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to check for duplicates: {}", e),
            )
        })
}

/// Find a soft-deleted book by hash (for restoration)
pub fn find_deleted_book_by_hash(file_hash: &str) -> Result<Option<Book>, AppError> {
    let mut conn = establish_connection()?;

    books::table
        .filter(books::file_hash.eq(file_hash))
        .filter(books::deleted_at.is_not_null())
        .select(Book::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to find deleted book: {}", e),
            )
        })
}

/// Restore a soft-deleted book with a new file path
pub fn restore_deleted_book(book_id: i32, new_file_path: &str) -> Result<Book, AppError> {
    info!("Restoring soft-deleted book ID: {} with path: {}", book_id, new_file_path);
    let mut conn = establish_connection()?;

    let now = chrono::Utc::now().naive_utc();

    diesel::update(books::table.find(book_id))
        .set((
            books::deleted_at.eq(None::<chrono::NaiveDateTime>),
            books::file_path.eq(new_file_path),
            books::updated_at.eq(now),
        ))
        .returning(Book::as_returning())
        .get_result(&mut conn)
        .map(|book| {
            info!("Book {} restored successfully", book_id);
            book
        })
        .map_err(|e| {
            error!("Failed to restore book {}: {}", book_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to restore book: {}", e),
            )
        })
}

// ============================================================================
// FILE PROCESSING HELPERS
// ============================================================================

/// Archive type detected from magic bytes
#[derive(Debug, Clone, Copy, PartialEq)]
enum ArchiveType {
    Zip,
    #[cfg(not(target_os = "android"))]
    Rar,
}

/// Detect archive type from magic bytes (file signature)
fn detect_archive_type(path: &Path) -> Result<ArchiveType, AppError> {
    let mut file = fs::File::open(path)
        .map_err(|e| AppError::new(ErrorCode::IoError, format!("Failed to open file: {}", e)))?;

    let mut magic = [0u8; 8];
    file.read(&mut magic).map_err(|e| {
        AppError::new(
            ErrorCode::IoError,
            format!("Failed to read file header: {}", e),
        )
    })?;

    // ZIP: starts with "PK" (0x50 0x4B)
    if magic[0] == 0x50 && magic[1] == 0x4B {
        return Ok(ArchiveType::Zip);
    }

    // RAR: starts with "Rar!" (0x52 0x61 0x72 0x21)
    #[cfg(not(target_os = "android"))]
    if magic[0] == 0x52 && magic[1] == 0x61 && magic[2] == 0x72 && magic[3] == 0x21 {
        return Ok(ArchiveType::Rar);
    }

    // On Android, RAR is not supported
    #[cfg(target_os = "android")]
    if magic[0] == 0x52 && magic[1] == 0x61 && magic[2] == 0x72 && magic[3] == 0x21 {
        return Err(AppError::new(
            ErrorCode::IoError,
            "RAR/CBR archives are not supported on Android. Please convert to CBZ format.",
        ));
    }

    // If we can't detect, try to infer from extension as fallback
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("zip") | Some("cbz") => Ok(ArchiveType::Zip),
        #[cfg(not(target_os = "android"))]
        Some("rar") | Some("cbr") => Ok(ArchiveType::Rar),
        #[cfg(target_os = "android")]
        Some("rar") | Some("cbr") => Err(AppError::new(
            ErrorCode::IoError,
            "RAR/CBR archives are not supported on Android. Please convert to CBZ format.",
        )),
        _ => Err(AppError::new(
            ErrorCode::IoError,
            "Unsupported or unrecognized archive format",
        )),
    }
}

/// Check if a file is an image based on extension
fn is_image_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png")
}

/// Extract book title from filename (removes only archive extensions)
fn extract_title(filename: &str) -> String {
    let lower = filename.to_lowercase();

    // Remove only known archive extensions
    if lower.ends_with(".cbz") {
        filename[..filename.len() - 4].to_string()
    } else if lower.ends_with(".zip") {
        filename[..filename.len() - 4].to_string()
    } else if lower.ends_with(".cbr") {
        filename[..filename.len() - 4].to_string()
    } else if lower.ends_with(".rar") {
        filename[..filename.len() - 4].to_string()
    } else if lower.ends_with(".cb7") {
        filename[..filename.len() - 4].to_string()
    } else if lower.ends_with(".7z") {
        filename[..filename.len() - 3].to_string()
    } else {
        filename.to_string()
    }
}

/// Calculate hash for a specific book (folder) within an archive
fn calculate_archive_hash(archive_path: &Path) -> Result<String, AppError> {
    match detect_archive_type(archive_path)? {
        ArchiveType::Zip => calculate_zip_hash(archive_path),
        #[cfg(not(target_os = "android"))]
        ArchiveType::Rar => calculate_rar_hash(archive_path),
    }
}

/// Calculate hash for all images in a ZIP/CBZ archive
fn calculate_zip_hash(archive_path: &Path) -> Result<String, AppError> {
    let file = fs::File::open(archive_path)
        .map_err(|e| AppError::new(ErrorCode::IoError, format!("Failed to open archive: {}", e)))?;

    let mut archive = ZipArchive::new(file).map_err(|e| {
        AppError::new(
            ErrorCode::IoError,
            format!("Failed to read zip archive: {}", e),
        )
    })?;

    let mut hasher = Sha256::new();
    let mut image_files: Vec<String> = Vec::new();

    // Collect all image file names
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to read archive entry: {}", e),
            )
        })?;

        let file_name = file.name().to_string();
        if !file.is_dir()
            && is_image_file(&file_name)
            && !file_name.starts_with('.')
            && !file_name.contains("/.")
        {
            image_files.push(file_name);
        }
    }

    // Sort for consistent hashing
    image_files.sort();

    // Hash all image content
    for file_name in &image_files {
        let mut file = archive.by_name(file_name).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to read file '{}': {}", file_name, e),
            )
        })?;

        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = file.read(&mut buffer).map_err(|e| {
                AppError::new(
                    ErrorCode::IoError,
                    format!("Failed to read file content: {}", e),
                )
            })?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate hash for all images in a RAR/CBR archive (desktop only)
#[cfg(not(target_os = "android"))]
fn calculate_rar_hash(archive_path: &Path) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    let mut image_entries: Vec<(String, Vec<u8>)> = Vec::new();

    let archive = unrar::Archive::new(archive_path)
        .open_for_processing()
        .map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to open RAR archive: {}", e),
            )
        })?;

    let mut current_archive = archive;
    loop {
        // Read header first to move to CursorBeforeFile state
        let header_result = current_archive.read_header();
        match header_result {
            Ok(Some(header)) => {
                let file_name = header.entry().filename.to_string_lossy().to_string();
                let is_dir = header.entry().is_directory();

                if !is_dir
                    && is_image_file(&file_name)
                    && !file_name.starts_with('.')
                    && !file_name.contains("/.")
                {
                    // Read the file content
                    let (data, next) = header.read().map_err(|e| {
                        AppError::new(
                            ErrorCode::IoError,
                            format!("Failed to read RAR entry: {}", e),
                        )
                    })?;
                    image_entries.push((file_name, data));
                    current_archive = next;
                } else {
                    // Skip non-image files
                    current_archive = header.skip().map_err(|e| {
                        AppError::new(
                            ErrorCode::IoError,
                            format!("Failed to skip RAR entry: {}", e),
                        )
                    })?;
                }
            }
            Ok(None) => break, // End of archive
            Err(e) => {
                return Err(AppError::new(
                    ErrorCode::IoError,
                    format!("Failed to read RAR header: {}", e),
                ));
            }
        }
    }

    // Sort by filename for consistent hashing
    image_entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Hash all image content
    for (_, data) in &image_entries {
        hasher.update(data);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Count images in a ZIP/CBZ archive
fn count_zip_images(archive_path: &Path) -> Result<i32, AppError> {
    let file = fs::File::open(archive_path)
        .map_err(|e| AppError::new(ErrorCode::IoError, format!("Failed to open archive: {}", e)))?;

    let mut archive = ZipArchive::new(file).map_err(|e| {
        AppError::new(
            ErrorCode::IoError,
            format!("Failed to read zip archive: {}", e),
        )
    })?;

    let mut count = 0;
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to read archive entry: {}", e),
            )
        })?;

        let file_name = file.name().to_string();
        if !file.is_dir()
            && is_image_file(&file_name)
            && !file_name.starts_with('.')
            && !file_name.contains("/.")
        {
            count += 1;
        }
    }

    Ok(count)
}

/// Count images in a RAR/CBR archive (desktop only)
#[cfg(not(target_os = "android"))]
fn count_rar_images(archive_path: &Path) -> Result<i32, AppError> {
    let archive = unrar::Archive::new(archive_path)
        .open_for_listing()
        .map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to open RAR archive for listing: {}", e),
            )
        })?;

    let mut count = 0;
    for entry in archive {
        let entry = entry.map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to read RAR entry: {}", e),
            )
        })?;

        let file_name = entry.filename.to_string_lossy().to_string();
        if !entry.is_directory()
            && is_image_file(&file_name)
            && !file_name.starts_with('.')
            && !file_name.contains("/.")
        {
            count += 1;
        }
    }

    Ok(count)
}

/// Count images in an archive (detects format using magic bytes)
fn count_archive_images(archive_path: &Path) -> Result<i32, AppError> {
    match detect_archive_type(archive_path)? {
        ArchiveType::Zip => count_zip_images(archive_path),
        #[cfg(not(target_os = "android"))]
        ArchiveType::Rar => count_rar_images(archive_path),
    }
}

// ============================================================================
// ARCHIVE IMPORT
// ============================================================================

/// Import a single book from a zip/cbz/rar/cbr archive
/// Archive type is detected using magic bytes, not file extension
/// Each archive is treated as a single book regardless of internal structure
/// If backup_files is true, copies the archive to library_dir before importing
/// Returns the imported Book or an error if the book is a duplicate
/// original_filename can be provided to override the filename extracted from the path
pub fn import_book_from_archive(
    archive_path: &Path,
    collection_id: Option<i32>,
    backup_files: bool,
    library_dir: &Path,
    original_filename: Option<String>,
) -> Result<Book, AppError> {
    info!(
        "Starting import from archive: {:?} (backup: {})",
        archive_path, backup_files
    );

    // Validate file exists
    if !archive_path.exists() {
        return Err(AppError::new(
            ErrorCode::IoError,
            "Archive file does not exist",
        ));
    }

    // Detect archive type using magic bytes
    let archive_type = detect_archive_type(archive_path)?;
    info!("Detected archive type: {:?}", archive_type);

    // Use original_filename if provided (for Android content URIs), otherwise extract from path
    let archive_filename = original_filename.unwrap_or_else(|| {
        archive_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Count images in the archive
    let total_pages = count_archive_images(archive_path)?;
    info!("Found {} image(s) in archive", total_pages);

    if total_pages == 0 {
        return Err(AppError::new(
            ErrorCode::IoError,
            "No images found in archive",
        ));
    }

    // Calculate hash for duplicate detection
    let book_hash = calculate_archive_hash(archive_path)?;

    // Check for active duplicates before backing up
    if let Some(existing_book) = find_book_by_hash(&book_hash)? {
        warn!(
            "Duplicate book detected: {} (hash: {}...)",
            archive_filename,
            &book_hash[..16]
        );
        return Err(AppError::new(
            ErrorCode::DuplicateEntry,
            format!("Duplicate of existing book '{}'", existing_book.title),
        ));
    }

    // Check if this book was previously deleted - if so, we'll restore it
    let deleted_book = find_deleted_book_by_hash(&book_hash)?;

    // Backup the file if enabled
    let effective_path = if backup_files {
        info!("Backing up archive to library directory");
        // Ensure library directory exists
        fs::create_dir_all(library_dir).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to create library directory: {}", e),
            )
        })?;

        // Generate destination path
        let mut dest_path = library_dir.join(&archive_filename);

        // Handle filename conflicts by appending a number
        if dest_path.exists() {
            let stem = dest_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("archive")
                .to_string();
            let ext = dest_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("cbz")
                .to_string();

            let mut counter = 1;
            loop {
                dest_path = library_dir.join(format!("{}_{}.{}", stem, counter, ext));
                if !dest_path.exists() {
                    break;
                }
                counter += 1;
            }
        }

        // Copy the file to the library directory
        fs::copy(archive_path, &dest_path).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to copy archive to library: {}", e),
            )
        })?;

        info!("Archive backed up to: {:?}", dest_path);
        dest_path
    } else {
        archive_path.to_path_buf()
    };

    // Get archive metadata
    let file_size: Option<i32> = fs::metadata(&effective_path)
        .ok()
        .and_then(|m| m.len().try_into().ok());

    let effective_filename = effective_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract title from filename
    let title = extract_title(&effective_filename);

    // Either restore deleted book or create new one
    let book = if let Some(deleted) = deleted_book {
        info!("Restoring previously deleted book: {} (ID: {})", deleted.title, deleted.id);
        restore_deleted_book(deleted.id, &effective_path.to_string_lossy())?
    } else {
        // Create the book entry
        let new_book = NewBook {
            file_path: effective_path.to_string_lossy().to_string(),
            filename: effective_filename,
            file_size,
            file_hash: Some(book_hash),
            title: title.clone(),
            total_pages,
            uuid: Some(uuid::Uuid::new_v4().to_string()),
        };

        create_book(new_book)?
    };
    
    info!("Imported book: {} (ID: {})", book.title, book.id);

    // Add to collection if specified
    if let Some(cid) = collection_id {
        add_book_to_collection(book.id, cid)?;
    }

    Ok(book)
}

// ============================================================================
// BOOK-COLLECTION OPERATIONS
// ============================================================================

/// Add a book to a collection
pub fn add_book_to_collection(
    book_id: i32,
    collection_id: i32,
) -> Result<BookCollection, AppError> {
    info!("Adding book {} to collection {}", book_id, collection_id);
    let mut conn = establish_connection()?;

    let new_entry = NewBookCollection {
        book_id,
        collection_id,
        uuid: Some(uuid::Uuid::new_v4().to_string()),
    };

    diesel::insert_into(book_collections::table)
        .values(&new_entry)
        .returning(BookCollection::as_returning())
        .get_result(&mut conn)
        .map(|entry: BookCollection| {
            info!(
                "Book {} added to collection {} successfully",
                book_id, collection_id
            );
            entry
        })
        .map_err(|e| {
            error!(
                "Failed to add book {} to collection {}: {}",
                book_id, collection_id, e
            );
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to add book to collection: {}", e),
            )
        })
}

/// Remove a book from a collection
pub fn remove_book_from_collection(book_id: i32, collection_id: i32) -> Result<(), AppError> {
    info!(
        "Removing book {} from collection {}",
        book_id, collection_id
    );
    let mut conn = establish_connection()?;

    diesel::delete(
        book_collections::table
            .filter(book_collections::book_id.eq(book_id))
            .filter(book_collections::collection_id.eq(collection_id)),
    )
    .execute(&mut conn)
    .map_err(|e| {
        error!(
            "Failed to remove book {} from collection {}: {}",
            book_id, collection_id, e
        );
        AppError::new(
            ErrorCode::DatabaseQueryFailed,
            format!("Failed to remove book from collection: {}", e),
        )
    })?;

    info!(
        "Book {} removed from collection {} successfully",
        book_id, collection_id
    );
    Ok(())
}

/// Set the collections for a book (replaces existing)
pub fn set_book_collections(book_id: i32, collection_ids: Vec<i32>) -> Result<(), AppError> {
    info!(
        "Setting collections for book {}: {:?}",
        book_id, collection_ids
    );
    let mut conn = establish_connection()?;

    // Remove all existing collection associations
    diesel::delete(book_collections::table.filter(book_collections::book_id.eq(book_id)))
        .execute(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to clear book collections: {}", e),
            )
        })?;

    // Add new collection associations
    for cid in collection_ids {
        let new_entry = NewBookCollection {
            book_id,
            collection_id: cid,
            uuid: Some(uuid::Uuid::new_v4().to_string()),
        };

        diesel::insert_into(book_collections::table)
            .values(&new_entry)
            .execute(&mut conn)
            .map_err(|e| {
                AppError::new(
                    ErrorCode::DatabaseQueryFailed,
                    format!("Failed to add book to collection {}: {}", cid, e),
                )
            })?;
    }

    info!("Book {} collections updated successfully", book_id);
    Ok(())
}

// ============================================================================
// BOOK SETTINGS
// ============================================================================

/// Get book settings by book ID
pub fn get_book_settings(book_id: i32) -> Result<Option<BookSettings>, AppError> {
    debug!("Fetching settings for book {}", book_id);
    let mut conn = establish_connection()?;

    book_settings::table
        .filter(book_settings::book_id.eq(book_id))
        .filter(book_settings::deleted_at.is_null())
        .select(BookSettings::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to load book settings: {}", e),
            )
        })
}

/// Update book settings (creates if not exists)
pub fn update_book_settings(
    book_id: i32,
    reading_direction: Option<Option<String>>,
    page_display_mode: Option<Option<String>>,
    image_fit_mode: Option<Option<String>>,
    sync_progress: Option<Option<bool>>,
) -> Result<BookSettings, AppError> {
    info!("Updating settings for book {}", book_id);
    let mut conn = establish_connection()?;

    // Check if settings exist
    let existing: Option<BookSettings> = book_settings::table
        .filter(book_settings::book_id.eq(book_id))
        .filter(book_settings::deleted_at.is_null())
        .select(BookSettings::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to check book settings: {}", e),
            )
        })?;

    let now = chrono::Utc::now().naive_utc();

    if let Some(_settings) = existing {
        // Update existing settings
        let updates = UpdateBookSettings {
            reading_direction,
            page_display_mode,
            image_fit_mode,
            sync_progress,
            updated_at: Some(now),
        };

        diesel::update(book_settings::table.filter(book_settings::book_id.eq(book_id)))
            .set(&updates)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to update book settings for {}: {}", book_id, e);
                AppError::new(
                    ErrorCode::DatabaseQueryFailed,
                    format!("Failed to update book settings: {}", e),
                )
            })?;
    } else {
        // Create new settings
        let new_settings = NewBookSettings {
            book_id,
            reading_direction: reading_direction.flatten(),
            page_display_mode: page_display_mode.flatten(),
            image_fit_mode: image_fit_mode.flatten(),
            sync_progress: sync_progress.flatten(),
            uuid: Some(uuid::Uuid::new_v4().to_string()),
        };

        diesel::insert_into(book_settings::table)
            .values(&new_settings)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to create book settings for {}: {}", book_id, e);
                AppError::new(
                    ErrorCode::DatabaseQueryFailed,
                    format!("Failed to create book settings: {}", e),
                )
            })?;
    }

    // Return the updated settings
    book_settings::table
        .filter(book_settings::book_id.eq(book_id))
        .select(BookSettings::as_select())
        .first(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to retrieve book settings: {}", e),
            )
        })
}

// ============================================================================
// BOOKMARKS
// ============================================================================

/// Create a new bookmark
pub fn create_bookmark(new_bookmark: NewBookmark) -> Result<Bookmark, AppError> {
    info!(
        "Creating bookmark '{}' for book {} at page {}",
        new_bookmark.name, new_bookmark.book_id, new_bookmark.page
    );
    let mut conn = establish_connection()?;

    diesel::insert_into(bookmarks::table)
        .values(&new_bookmark)
        .returning(Bookmark::as_returning())
        .get_result(&mut conn)
        .map(|bookmark: Bookmark| {
            info!(
                "Bookmark created: {} (ID: {})",
                bookmark.name, bookmark.id
            );
            bookmark
        })
        .map_err(|e| {
            error!("Failed to create bookmark: {}", e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to create bookmark: {}", e),
            )
        })
}

/// Get all bookmarks for a book (excludes soft-deleted)
pub fn get_bookmarks_for_book(book_id: i32) -> Result<Vec<Bookmark>, AppError> {
    debug!("Fetching bookmarks for book {}", book_id);
    let mut conn = establish_connection()?;

    bookmarks::table
        .filter(bookmarks::book_id.eq(book_id))
        .filter(bookmarks::deleted_at.is_null())
        .order(bookmarks::page.asc())
        .select(Bookmark::as_select())
        .load(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to load bookmarks: {}", e),
            )
        })
}

/// Get a single bookmark by ID
pub fn get_bookmark_by_id(bookmark_id: i32) -> Result<Bookmark, AppError> {
    let mut conn = establish_connection()?;

    bookmarks::table
        .find(bookmark_id)
        .filter(bookmarks::deleted_at.is_null())
        .select(Bookmark::as_select())
        .first(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to find bookmark: {}", e),
            )
        })
}

/// Update a bookmark
pub fn update_bookmark(
    bookmark_id: i32,
    name: String,
    description: Option<String>,
) -> Result<Bookmark, AppError> {
    info!("Updating bookmark {}", bookmark_id);
    let mut conn = establish_connection()?;

    let now = chrono::Utc::now().naive_utc();
    diesel::update(bookmarks::table.find(bookmark_id))
        .set((
            bookmarks::name.eq(&name),
            bookmarks::description.eq(&description),
            bookmarks::updated_at.eq(Some(now)),
        ))
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to update bookmark {}: {}", bookmark_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to update bookmark: {}", e),
            )
        })?;

    get_bookmark_by_id(bookmark_id)
}

/// Delete a bookmark (soft-delete)
pub fn delete_bookmark(bookmark_id: i32) -> Result<(), AppError> {
    info!("Deleting bookmark {}", bookmark_id);
    let mut conn = establish_connection()?;

    let now = chrono::Utc::now().naive_utc();
    diesel::update(bookmarks::table.find(bookmark_id))
        .set((
            bookmarks::deleted_at.eq(Some(now)),
            bookmarks::updated_at.eq(Some(now)),
        ))
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to delete bookmark {}: {}", bookmark_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to delete bookmark: {}", e),
            )
        })?;

    info!("Bookmark {} deleted successfully", bookmark_id);
    Ok(())
}

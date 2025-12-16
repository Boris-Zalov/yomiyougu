//! Database operations for books and collections
//!
//! Provides CRUD operations and business logic for library management

use diesel::prelude::*;
use log::{debug, error, info, warn};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::database::connection::establish_connection;
use crate::database::models::*;
use crate::error::{AppError, ErrorCode};
use crate::schema::{books, collections};

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
            info!("Collection created successfully: {} (ID: {})", collection.name, collection.id);
            collection
        })
        .map_err(|e| {
            error!("Failed to create collection '{}': {}", new_collection.name, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to create collection: {}", e),
            )
        })
}

/// Get all collections with book counts
pub fn get_all_collections() -> Result<Vec<CollectionWithCount>, AppError> {
    debug!("Fetching all collections with book counts");
    let mut conn = establish_connection()?;

    let collections_list = collections::table
        .select(Collection::as_select())
        .load(&mut conn)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to load collections: {}", e),
            )
        })?;

    // Get book counts for each collection
    let mut result = Vec::new();
    for collection in collections_list {
        let count = books::table
            .filter(books::collection_id.eq(collection.id))
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

/// Get a single collection by ID
pub fn get_collection_by_id(collection_id: i32) -> Result<Collection, AppError> {
    let mut conn = establish_connection()?;

    collections::table
        .find(collection_id)
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

/// Delete a collection (sets collection_id to NULL for associated books)
pub fn delete_collection(collection_id: i32) -> Result<(), AppError> {
    info!("Deleting collection ID: {}", collection_id);
    let mut conn = establish_connection()?;

    diesel::delete(collections::table.find(collection_id))
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to delete collection {}: {}", collection_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to delete collection: {}", e),
            )
        })?;

    info!("Collection {} deleted successfully", collection_id);
    Ok(())
}

// ============================================================================
// BOOKS
// ============================================================================

/// Create a new book
pub fn create_book(new_book: NewBook) -> Result<Book, AppError> {
    info!("Creating new book: {} ({} pages)", new_book.title, new_book.total_pages);
    let mut conn = establish_connection()?;

    diesel::insert_into(books::table)
        .values(&new_book)
        .returning(Book::as_returning())
        .get_result(&mut conn)
        .map(|book: Book| {
            info!("Book created successfully: {} (ID: {})", book.title, book.id);
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
    debug!("Fetching books - collection: {:?}, status: {:?}, favorites: {}", 
           collection_id, status, favorites_only);
    let mut conn = establish_connection()?;

    let mut query = books::table.into_boxed();

    if let Some(cid) = collection_id {
        query = query.filter(books::collection_id.eq(cid));
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

    // TODO: Load collection names and settings for each book
    let result: Vec<BookWithDetails> = books_list
        .into_iter()
        .map(|book| BookWithDetails {
            book,
            collection_name: None,
            settings: None,
            bookmark_count: 0,
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

/// Delete a book
pub fn delete_book(book_id: i32) -> Result<(), AppError> {
    info!("Deleting book ID: {}", book_id);
    let mut conn = establish_connection()?;

    diesel::delete(books::table.find(book_id))
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to delete book {}: {}", book_id, e);
            AppError::new(
                ErrorCode::DatabaseQueryFailed,
                format!("Failed to delete book: {}", e),
            )
        })?;

    info!("Book {} deleted successfully", book_id);
    Ok(())
}

/// Check if a file hash already exists in the database
pub fn find_book_by_hash(file_hash: &str) -> Result<Option<Book>, AppError> {
    let mut conn = establish_connection()?;

    books::table
        .filter(books::file_hash.eq(file_hash))
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

// ============================================================================
// FILE PROCESSING HELPERS
// ============================================================================

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

/// Represents a book found within an archive
struct ArchiveBook {
    /// Folder path within the archive (empty string for root)
    folder_path: String,
    image_files: Vec<String>,
}

/// Parse archive and group images by folder
fn parse_archive_structure<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Vec<ArchiveBook>, AppError> {
    let mut folder_images: HashMap<String, Vec<String>> = HashMap::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to read archive entry: {}", e),
            )
        })?;

        let file_name = file.name().to_string();

        if file.is_dir() || file_name.starts_with('.') || file_name.contains("/.") {
            continue;
        }

        if !is_image_file(&file_name) {
            continue;
        }

        let folder_path = if let Some(pos) = file_name.rfind('/') {
            file_name[..pos].to_string()
        } else {
            String::new()
        };

        folder_images
            .entry(folder_path)
            .or_default()
            .push(file_name);
    }

    // Convert to ArchiveBook structs
    let mut books: Vec<ArchiveBook> = folder_images
        .into_iter()
        .filter(|(_, images)| !images.is_empty())
        .map(|(folder_path, mut image_files)| {
            image_files.sort();
            ArchiveBook {
                folder_path,
                image_files,
            }
        })
        .collect();

    books.sort_by(|a, b| a.folder_path.cmp(&b.folder_path));

    Ok(books)
}

/// Calculate hash for a specific book (folder) within an archive
fn calculate_book_hash<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    image_files: &[String],
) -> Result<String, AppError> {
    let mut hasher = Sha256::new();

    for file_name in image_files {
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

// ============================================================================
// ARCHIVE IMPORT
// ============================================================================

/// Import books from a zip/cbz archive
/// If backup_files is true, copies the archive to library_dir before importing
/// Returns ImportResult with successfully imported books and skipped duplicates
pub fn import_books_from_archive(
    archive_path: &Path,
    collection_id: Option<i32>,
    backup_files: bool,
    library_dir: &Path,
) -> Result<ImportResult, AppError> {
    info!("Starting import from archive: {:?} (backup: {})", archive_path, backup_files);
    // Validate file exists
    if !archive_path.exists() {
        return Err(AppError::new(
            ErrorCode::IoError,
            "Archive file does not exist",
        ));
    }

    // Determine the effective path (original or backup location)
    let effective_path = if backup_files {
        // Ensure library directory exists
        fs::create_dir_all(library_dir).map_err(|e| {
            AppError::new(
                ErrorCode::IoError,
                format!("Failed to create library directory: {}", e),
            )
        })?;

        // Generate destination path
        let archive_filename = archive_path
            .file_name()
            .ok_or_else(|| AppError::new(ErrorCode::IoError, "Invalid archive filename"))?;

        let mut dest_path = library_dir.join(archive_filename);

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

        dest_path
    } else {
        archive_path.to_path_buf()
    };

    let file = fs::File::open(&effective_path).map_err(|e| {
        AppError::new(
            ErrorCode::IoError,
            format!("Failed to open archive: {}", e),
        )
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| {
        AppError::new(
            ErrorCode::IoError,
            format!("Failed to read zip archive: {}", e),
        )
    })?;

    // Get archive metadata
    let file_size: Option<i32> = fs::metadata(&effective_path)
        .ok()
        .and_then(|m| m.len().try_into().ok());

    let archive_filename = effective_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Parse archive structure to find books
    let archive_books = parse_archive_structure(&mut archive)?;
    info!("Found {} book(s) in archive", archive_books.len());

    if archive_books.is_empty() {
        return Err(AppError::new(
            ErrorCode::IoError,
            "No images found in archive",
        ));
    }

    let mut imported: Vec<Book> = Vec::new();
    let mut skipped: Vec<SkippedBook> = Vec::new();

    // Check if it's a single book (all images at root or single folder)
    let is_single_book = archive_books.len() == 1;

    for archive_book in archive_books {
        // Calculate unique hash for this book's content
        let book_hash = calculate_book_hash(&mut archive, &archive_book.image_files)?;

        // Determine book title
        let title = if is_single_book && archive_book.folder_path.is_empty() {
            // Single book at root - use archive name
            extract_title(&archive_filename)
        } else if archive_book.folder_path.is_empty() {
            // Multiple books but this one is at root - use archive name
            extract_title(&archive_filename)
        } else {
            // Use the deepest folder name as title
            let folder_name = archive_book
                .folder_path
                .rsplit('/')
                .next()
                .unwrap_or(&archive_book.folder_path);
            extract_title(folder_name)
        };

        // Build file_path: archive_path for single root book, or archive_path#folder for others
        let file_path = if is_single_book && archive_book.folder_path.is_empty() {
            effective_path.to_string_lossy().to_string()
        } else if archive_book.folder_path.is_empty() {
            // Root images in multi-book archive
            format!("{}#", effective_path.to_string_lossy())
        } else {
            format!(
                "{}#{}",
                effective_path.to_string_lossy(),
                archive_book.folder_path
            )
        };

        // Check for duplicate by hash
        match find_book_by_hash(&book_hash)? {
            Some(existing_book) => {
                warn!("Skipping duplicate book: {} (hash: {}...)", title, &book_hash[..16]);
                skipped.push(SkippedBook {
                    title: title.clone(),
                    reason: format!("Duplicate of existing book '{}'", existing_book.title),
                    existing_book_id: Some(existing_book.id),
                });
            }
            None => {
                // Create the new book
                let new_book = NewBook {
                    file_path,
                    filename: archive_filename.clone(),
                    file_size,
                    file_hash: Some(book_hash),
                    title: title.clone(),
                    total_pages: archive_book.image_files.len() as i32,
                    collection_id,
                };

                match create_book(new_book) {
                    Ok(book) => {
                        debug!("Imported book: {} (ID: {})", book.title, book.id);
                        imported.push(book)
                    }
                    Err(e) => {
                        // Could be a file_path uniqueness violation or other DB error
                        skipped.push(SkippedBook {
                            title,
                            reason: format!("Database error: {}", e),
                            existing_book_id: None,
                        });
                    }
                }
            }
        }
    }

    info!("Import completed: {} imported, {} skipped", imported.len(), skipped.len());
    Ok(ImportResult { imported, skipped })
}

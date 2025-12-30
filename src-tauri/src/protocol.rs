//! Comic image protocol handler
//!
//! Serves images from comic archives (CBZ/ZIP, CBR/RAR) via a custom protocol.
//! URL format: comic://book/{book_id}/page/{page_number}
//! - page 0 is the cover (first image in sorted order)

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::RwLock;

use tauri::http::{Request, Response};
use zip::ZipArchive;

use crate::database::operations::get_book_by_id;

/// Cache for image lists (book_id -> sorted image names)
static IMAGE_LIST_CACHE: RwLock<Option<HashMap<i32, Vec<String>>>> = RwLock::new(None);

/// Maximum cache size (number of books to cache)
const MAX_CACHE_SIZE: usize = 10;

/// Get cached image list or compute and cache it
fn get_cached_image_list(
    book_id: i32,
    archive_path: &Path,
    archive_type: ArchiveType,
) -> Result<Vec<String>, String> {
    // Try to read from cache first
    {
        let cache = IMAGE_LIST_CACHE.read().unwrap();
        if let Some(ref map) = *cache {
            if let Some(list) = map.get(&book_id) {
                return Ok(list.clone());
            }
        }
    }

    let list = get_image_list(archive_path, archive_type)?;

    // Store in cache
    {
        let mut cache = IMAGE_LIST_CACHE.write().unwrap();
        let map = cache.get_or_insert_with(HashMap::new);
        
        // Evict oldest entries if cache is too large
        if map.len() >= MAX_CACHE_SIZE {
            // Remove first entry
            if let Some(key) = map.keys().next().cloned() {
                map.remove(&key);
            }
        }
        
        map.insert(book_id, list.clone());
    }

    Ok(list)
}

/// Invalidate cache for a specific book
#[allow(dead_code)]
pub fn invalidate_image_cache(book_id: i32) {
    let mut cache = IMAGE_LIST_CACHE.write().unwrap();
    if let Some(ref mut map) = *cache {
        map.remove(&book_id);
    }
}

/// Clear entire image cache
#[allow(dead_code)]
pub fn clear_image_cache() {
    let mut cache = IMAGE_LIST_CACHE.write().unwrap();
    *cache = None;
}

/// Check if a file is an image based on extension
fn is_image_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
}

/// Get sorted list of image files from a ZIP/CBZ archive
fn get_zip_image_list(archive_path: &Path) -> Result<Vec<String>, String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer for faster reads

    let mut archive =
        ZipArchive::new(reader).map_err(|e| format!("Failed to read zip archive: {}", e))?;

    let mut image_files: Vec<String> = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        let file = archive
            .by_index_raw(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let file_name = file.name().to_string();
        if !file.is_dir()
            && is_image_file(&file_name)
            && !file_name.starts_with('.')
            && !file_name.contains("/.")
            && !file_name.contains("__MACOSX")
        {
            image_files.push(file_name);
        }
    }

    // Sort naturally (handles "page1", "page2", "page10" correctly)
    image_files.sort_by(|a, b| natord::compare(a, b));

    Ok(image_files)
}

/// Get sorted list of image files from a RAR/CBR archive (desktop only)
#[cfg(not(target_os = "android"))]
fn get_rar_image_list(archive_path: &Path) -> Result<Vec<String>, String> {
    let archive = unrar::Archive::new(archive_path)
        .open_for_listing()
        .map_err(|e| format!("Failed to open RAR archive: {}", e))?;

    let mut image_files: Vec<String> = Vec::new();

    for entry in archive {
        let entry = entry.map_err(|e| format!("Failed to read RAR entry: {}", e))?;
        let file_name = entry.filename.to_string_lossy().to_string();

        if !entry.is_directory()
            && is_image_file(&file_name)
            && !file_name.starts_with('.')
            && !file_name.contains("/.")
        {
            image_files.push(file_name);
        }
    }

    // Sort naturally
    image_files.sort_by(|a, b| natord::compare(a, b));

    Ok(image_files)
}

/// Read a specific image from a ZIP/CBZ archive
fn read_zip_image(archive_path: &Path, image_name: &str) -> Result<(Vec<u8>, String), String> {
    let file = File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer

    let mut archive =
        ZipArchive::new(reader).map_err(|e| format!("Failed to read zip archive: {}", e))?;

    let mut entry = archive
        .by_name(image_name)
        .map_err(|e| format!("Failed to find image '{}': {}", image_name, e))?;

    // Pre-allocate buffer based on uncompressed size for efficiency
    let size_hint = entry.size() as usize;
    let mut buffer = Vec::with_capacity(size_hint.max(1024));
    entry
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read image data: {}", e))?;

    let mime_type = get_mime_type(image_name);

    Ok((buffer, mime_type))
}

/// Read a specific image from a RAR/CBR archive (desktop only)
#[cfg(not(target_os = "android"))]
fn read_rar_image(archive_path: &Path, image_name: &str) -> Result<(Vec<u8>, String), String> {
    let archive = unrar::Archive::new(archive_path)
        .open_for_processing()
        .map_err(|e| format!("Failed to open RAR archive: {}", e))?;

    let mut current_archive = archive;
    loop {
        let header_result = current_archive.read_header();
        match header_result {
            Ok(Some(header)) => {
                let file_name = header.entry().filename.to_string_lossy().to_string();

                if file_name == image_name {
                    let (data, _) = header
                        .read()
                        .map_err(|e| format!("Failed to read RAR entry: {}", e))?;
                    let mime_type = get_mime_type(&file_name);
                    return Ok((data, mime_type));
                } else {
                    current_archive = header
                        .skip()
                        .map_err(|e| format!("Failed to skip RAR entry: {}", e))?;
                }
            }
            Ok(None) => break,
            Err(e) => return Err(format!("Failed to read RAR header: {}", e)),
        }
    }

    Err(format!("Image '{}' not found in archive", image_name))
}

/// Determine MIME type from file extension
fn get_mime_type(filename: &str) -> String {
    let lower = filename.to_lowercase();
    if lower.ends_with(".png") {
        "image/png".to_string()
    } else if lower.ends_with(".gif") {
        "image/gif".to_string()
    } else if lower.ends_with(".webp") {
        "image/webp".to_string()
    } else {
        "image/jpeg".to_string()
    }
}

/// Detect archive type from magic bytes
fn detect_archive_type(path: &Path) -> Result<ArchiveType, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut magic = [0u8; 8];
    file.read(&mut magic)
        .map_err(|e| format!("Failed to read file header: {}", e))?;

    // ZIP: starts with "PK" (0x50 0x4B)
    if magic[0] == 0x50 && magic[1] == 0x4B {
        return Ok(ArchiveType::Zip);
    }

    // RAR: starts with "Rar!" (0x52 0x61 0x72 0x21)
    #[cfg(not(target_os = "android"))]
    if magic[0] == 0x52 && magic[1] == 0x61 && magic[2] == 0x72 && magic[3] == 0x21 {
        return Ok(ArchiveType::Rar);
    }

    #[cfg(target_os = "android")]
    if magic[0] == 0x52 && magic[1] == 0x61 && magic[2] == 0x72 && magic[3] == 0x21 {
        return Err("RAR archives not supported on Android".to_string());
    }

    // Fallback to extension
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("zip") | Some("cbz") => Ok(ArchiveType::Zip),
        #[cfg(not(target_os = "android"))]
        Some("rar") | Some("cbr") => Ok(ArchiveType::Rar),
        _ => Err("Unsupported archive format".to_string()),
    }
}

#[derive(Debug, Clone, Copy)]
enum ArchiveType {
    Zip,
    #[cfg(not(target_os = "android"))]
    Rar,
}

/// Get image list based on archive type
fn get_image_list(archive_path: &Path, archive_type: ArchiveType) -> Result<Vec<String>, String> {
    match archive_type {
        ArchiveType::Zip => get_zip_image_list(archive_path),
        #[cfg(not(target_os = "android"))]
        ArchiveType::Rar => get_rar_image_list(archive_path),
    }
}

/// Read image based on archive type
fn read_image(
    archive_path: &Path,
    image_name: &str,
    archive_type: ArchiveType,
) -> Result<(Vec<u8>, String), String> {
    match archive_type {
        ArchiveType::Zip => read_zip_image(archive_path, image_name),
        #[cfg(not(target_os = "android"))]
        ArchiveType::Rar => read_rar_image(archive_path, image_name),
    }
}

/// Handle comic:// protocol requests
/// URL format: comic://localhost/book/{book_id}/page/{page_number}
pub fn handle_comic_protocol(request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let uri = request.uri().to_string();
    log::debug!("Comic protocol request: {}", uri);

    let path = uri
        .strip_prefix("comic://localhost")
        .or_else(|| uri.strip_prefix("comic://"))
        .unwrap_or(&uri);

    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() < 4 || parts[0] != "book" || parts[2] != "page" {
        log::warn!("Invalid comic URL format: {}", uri);
        return Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body("Invalid URL format. Expected: comic://localhost/book/{id}/page/{number}".as_bytes().to_vec())
            .unwrap();
    }

    let book_id: i32 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => {
            return Response::builder()
                .status(400)
                .header("Content-Type", "text/plain")
                .body("Invalid book ID".as_bytes().to_vec())
                .unwrap();
        }
    };

    let page_number: usize = match parts[3].parse() {
        Ok(num) => num,
        Err(_) => {
            return Response::builder()
                .status(400)
                .header("Content-Type", "text/plain")
                .body("Invalid page number".as_bytes().to_vec())
                .unwrap();
        }
    };

    // Get the book from database
    let book = match get_book_by_id(book_id) {
        Ok(b) => b,
        Err(e) => {
            log::error!("Failed to get book {}: {}", book_id, e);
            return Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(format!("Book not found: {}", e).as_bytes().to_vec())
                .unwrap();
        }
    };

    // Check if book is cloud-only
    if book.file_path.starts_with("cloud://") {
        return Response::builder()
            .status(404)
            .header("Content-Type", "text/plain")
            .body("Book is stored in cloud. Please download first.".as_bytes().to_vec())
            .unwrap();
    }

    let archive_path = Path::new(&book.file_path);

    // Check if file exists
    if !archive_path.exists() {
        log::error!("Archive file not found: {}", book.file_path);
        return Response::builder()
            .status(404)
            .header("Content-Type", "text/plain")
            .body("Archive file not found".as_bytes().to_vec())
            .unwrap();
    }

    // Detect archive type
    let archive_type = match detect_archive_type(archive_path) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to detect archive type: {}", e);
            return Response::builder()
                .status(500)
                .header("Content-Type", "text/plain")
                .body(e.as_bytes().to_vec())
                .unwrap();
        }
    };

    // Get the list of images (cached for performance)
    let image_list = match get_cached_image_list(book_id, archive_path, archive_type) {
        Ok(list) => list,
        Err(e) => {
            log::error!("Failed to get image list: {}", e);
            return Response::builder()
                .status(500)
                .header("Content-Type", "text/plain")
                .body(e.as_bytes().to_vec())
                .unwrap();
        }
    };

    if image_list.is_empty() {
        return Response::builder()
            .status(404)
            .header("Content-Type", "text/plain")
            .body("No images found in archive".as_bytes().to_vec())
            .unwrap();
    }

    // Check page number bounds
    if page_number >= image_list.len() {
        return Response::builder()
            .status(404)
            .header("Content-Type", "text/plain")
            .body(format!("Page {} not found. Archive has {} pages.", page_number, image_list.len()).as_bytes().to_vec())
            .unwrap();
    }

    let image_name = &image_list[page_number];

    // Read the image
    let (image_data, mime_type) = match read_image(archive_path, image_name, archive_type) {
        Ok((data, mime)) => (data, mime),
        Err(e) => {
            log::error!("Failed to read image: {}", e);
            return Response::builder()
                .status(500)
                .header("Content-Type", "text/plain")
                .body(e.as_bytes().to_vec())
                .unwrap();
        }
    };

    log::debug!(
        "Serving page {} ({}) from book {}, {} bytes",
        page_number,
        image_name,
        book_id,
        image_data.len()
    );

    Response::builder()
        .status(200)
        .header("Content-Type", &mime_type)
        .header("Cache-Control", "max-age=31536000, immutable")
        .body(image_data)
        .unwrap()
}

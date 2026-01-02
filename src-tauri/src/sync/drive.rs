//! Google Drive integration for sync
//!
//! Handles reading/writing the sync snapshot to Google Drive's appData folder.

use crate::error::AppError;
use super::types::SyncSnapshot;

const SYNC_FILENAME: &str = "sync_snapshot.json";
const DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";
const DRIVE_UPLOAD_BASE: &str = "https://www.googleapis.com/upload/drive/v3";

/// Google Drive sync operations
pub struct DriveSync {
    access_token: String,
}

impl DriveSync {
    /// Create with a specific access token
    pub fn with_token(access_token: String) -> Self {
        Self { access_token }
    }

    /// Find the sync file in appData folder, returns file ID if found
    /// If a cached_file_id is provided, verifies it still exists before using it
    pub async fn find_sync_file(&self, cached_file_id: Option<&str>) -> Result<Option<String>, AppError> {
        if let Some(id) = cached_file_id {
            if self.verify_file_exists(id).await? {
                log::info!("Using cached sync file ID: {}", id);
                return Ok(Some(id.to_string()));
            }
            log::info!("Cached sync file ID {} no longer valid, searching...", id);
        }
        
        let client = reqwest::Client::new();
        
        let response = client
            .get(format!("{}/files", DRIVE_API_BASE))
            .bearer_auth(&self.access_token)
            .query(&[
                ("spaces", "appDataFolder"),
                ("q", &format!("name = '{}'", SYNC_FILENAME)),
                ("fields", "files(id, name, modifiedTime)"),
            ])
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to search Drive: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::sync_failed(format!(
                "Drive API error {}: {}",
                status, body
            )));
        }

        #[derive(serde::Deserialize)]
        struct FileList {
            files: Vec<FileInfo>,
        }
        
        #[derive(serde::Deserialize)]
        struct FileInfo {
            id: String,
        }

        let file_list: FileList = response.json().await
            .map_err(|e| AppError::sync_failed(format!("Failed to parse file list: {}", e)))?;

        Ok(file_list.files.into_iter().next().map(|f| f.id))
    }

    /// Verify a file ID still exists on Drive
    async fn verify_file_exists(&self, file_id: &str) -> Result<bool, AppError> {
        let client = reqwest::Client::new();
        
        let response = client
            .get(format!("{}/files/{}", DRIVE_API_BASE, file_id))
            .bearer_auth(&self.access_token)
            .query(&[("fields", "id")])
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to verify file: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// Download the sync snapshot from Google Drive
    pub async fn download_snapshot(&self, cached_file_id: Option<&str>) -> Result<Option<SyncSnapshot>, AppError> {
        let file_id = match self.find_sync_file(cached_file_id).await? {
            Some(id) => id,
            None => return Ok(None),
        };

        let client = reqwest::Client::new();
        
        let response = client
            .get(format!("{}/files/{}", DRIVE_API_BASE, file_id))
            .bearer_auth(&self.access_token)
            .query(&[("alt", "media")])
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to download snapshot: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Ok(None);
            }
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::sync_failed(format!(
                "Drive download error {}: {}",
                status, body
            )));
        }

        let snapshot: SyncSnapshot = response.json().await
            .map_err(|e| AppError::sync_failed(format!("Failed to parse snapshot: {}", e)))?;

        log::info!("Downloaded sync snapshot with {} books, {} bookmarks, {} collections",
            snapshot.books.len(),
            snapshot.bookmarks.len(),
            snapshot.collections.len()
        );

        Ok(Some(snapshot))
    }

    /// Upload the sync snapshot to Google Drive
    pub async fn upload_snapshot(&self, snapshot: &SyncSnapshot, existing_file_id: Option<&str>) -> Result<String, AppError> {
        let client = reqwest::Client::new();
        let json_content = serde_json::to_string(snapshot)
            .map_err(|e| AppError::sync_failed(format!("Failed to serialize snapshot: {}", e)))?;

        let file_id = if let Some(id) = existing_file_id {
            // Update existing file
            let response = client
                .patch(format!("{}/files/{}", DRIVE_UPLOAD_BASE, id))
                .bearer_auth(&self.access_token)
                .query(&[("uploadType", "media")])
                .header("Content-Type", "application/json")
                .body(json_content)
                .send()
                .await
                .map_err(|e| AppError::sync_failed(format!("Failed to update snapshot: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::sync_failed(format!(
                    "Drive update error {}: {}",
                    status, body
                )));
            }

            id.to_string()
        } else {
            // Create new file
            #[derive(serde::Serialize)]
            struct FileMetadata {
                name: String,
                parents: Vec<String>,
            }

            let metadata = FileMetadata {
                name: SYNC_FILENAME.to_string(),
                parents: vec!["appDataFolder".to_string()],
            };

            let metadata_json = serde_json::to_string(&metadata)
                .map_err(|e| AppError::sync_failed(format!("Failed to serialize metadata: {}", e)))?;

            // Use multipart upload for creating new file with metadata
            let boundary = "sync_boundary_12345";
            let body = format!(
                "--{boundary}\r\n\
                Content-Type: application/json; charset=UTF-8\r\n\r\n\
                {metadata_json}\r\n\
                --{boundary}\r\n\
                Content-Type: application/json\r\n\r\n\
                {json_content}\r\n\
                --{boundary}--"
            );

            let response = client
                .post(format!("{}/files", DRIVE_UPLOAD_BASE))
                .bearer_auth(&self.access_token)
                .query(&[("uploadType", "multipart")])
                .header("Content-Type", format!("multipart/related; boundary={}", boundary))
                .body(body)
                .send()
                .await
                .map_err(|e| AppError::sync_failed(format!("Failed to create snapshot: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::sync_failed(format!(
                    "Drive create error {}: {}",
                    status, body
                )));
            }

            #[derive(serde::Deserialize)]
            struct CreateResponse {
                id: String,
            }

            let create_response: CreateResponse = response.json().await
                .map_err(|e| AppError::sync_failed(format!("Failed to parse create response: {}", e)))?;

            create_response.id
        };

        log::info!("Uploaded sync snapshot with {} books, {} bookmarks, {} collections",
            snapshot.books.len(),
            snapshot.bookmarks.len(),
            snapshot.collections.len()
        );

        Ok(file_id)
    }

    /// Delete a book file from Google Drive by its hash
    pub async fn delete_book_file(&self, file_hash: &str) -> Result<bool, AppError> {
        let file_id = match self.find_book_file(file_hash).await? {
            Some(id) => id,
            None => {
                log::info!("Book file {} not found in Drive, nothing to delete", file_hash);
                return Ok(false);
            }
        };

        log::info!("Deleting book file {} (Drive ID: {})...", file_hash, file_id);

        let client = reqwest::Client::new();
        
        let response = client
            .delete(format!("{}/files/{}", DRIVE_API_BASE, file_id))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to delete book file: {}", e)))?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            log::info!("Successfully deleted book file {} from Drive", file_hash);
            Ok(true)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(AppError::sync_failed(format!(
                "Drive delete error {}: {}",
                status, body
            )))
        }
    }

    /// Find a comic book file in appData folder by its hash
    pub async fn find_book_file(&self, file_hash: &str) -> Result<Option<String>, AppError> {
        let client = reqwest::Client::new();
        let filename = format!("book_{}.cbz", file_hash);
        
        let response = client
            .get(format!("{}/files", DRIVE_API_BASE))
            .bearer_auth(&self.access_token)
            .query(&[
                ("spaces", "appDataFolder"),
                ("q", &format!("name = '{}'", filename)),
                ("fields", "files(id, name, modifiedTime, size)"),
            ])
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to search for book file: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::sync_failed(format!(
                "Drive API error {}: {}",
                status, body
            )));
        }

        #[derive(serde::Deserialize)]
        struct FileList {
            files: Vec<FileInfo>,
        }
        
        #[derive(serde::Deserialize)]
        struct FileInfo {
            id: String,
        }

        let file_list: FileList = response.json().await
            .map_err(|e| AppError::sync_failed(format!("Failed to parse file list: {}", e)))?;

        Ok(file_list.files.into_iter().next().map(|f| f.id))
    }

    /// Upload a comic book file to Google Drive appData folder
    pub async fn upload_book_file(&self, file_path: &str, file_hash: &str) -> Result<String, AppError> {
        use std::fs;
        
        let client = reqwest::Client::new();
        let filename = format!("book_{}.cbz", file_hash);
        
        // Read file content
        let file_content = fs::read(file_path)
            .map_err(|e| AppError::sync_failed(format!("Failed to read book file: {}", e)))?;
        
        // Check if file already exists
        if let Some(existing_id) = self.find_book_file(file_hash).await? {
            log::info!("Book file {} already exists in Drive, skipping upload", file_hash);
            return Ok(existing_id);
        }
        
        log::info!("Uploading book file {} ({} bytes)...", filename, file_content.len());

        // Create file metadata
        #[derive(serde::Serialize)]
        struct FileMetadata {
            name: String,
            parents: Vec<String>,
        }

        let metadata = FileMetadata {
            name: filename.clone(),
            parents: vec!["appDataFolder".to_string()],
        };

        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| AppError::sync_failed(format!("Failed to serialize metadata: {}", e)))?;

        // Use resumable upload for larger files
        let boundary = format!("book_boundary_{}", uuid::Uuid::new_v4());
        
        // Build multipart body
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(metadata_json.as_bytes());
        body.extend_from_slice(format!("\r\n--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/zip\r\n\r\n");
        body.extend_from_slice(&file_content);
        body.extend_from_slice(format!("\r\n--{}--", boundary).as_bytes());

        let response = client
            .post(format!("{}/files", DRIVE_UPLOAD_BASE))
            .bearer_auth(&self.access_token)
            .query(&[("uploadType", "multipart")])
            .header("Content-Type", format!("multipart/related; boundary={}", boundary))
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to upload book file: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::sync_failed(format!(
                "Drive upload error {}: {}",
                status, body
            )));
        }

        #[derive(serde::Deserialize)]
        struct CreateResponse {
            id: String,
        }

        let create_response: CreateResponse = response.json().await
            .map_err(|e| AppError::sync_failed(format!("Failed to parse upload response: {}", e)))?;

        log::info!("Uploaded book file {} with ID {}", filename, create_response.id);

        Ok(create_response.id)
    }

    /// Download a comic book file from Google Drive
    pub async fn download_book_file(&self, file_hash: &str, target_path: &str) -> Result<(), AppError> {
        use std::fs;
        use std::path::Path;
        
        let file_id = self.find_book_file(file_hash).await?
            .ok_or_else(|| AppError::sync_failed(format!("Book file not found in Drive: {}", file_hash)))?;
        
        log::info!("Downloading book file {} from Drive...", file_hash);
        
        let client = reqwest::Client::new();
        
        let response = client
            .get(format!("{}/files/{}", DRIVE_API_BASE, file_id))
            .bearer_auth(&self.access_token)
            .query(&[("alt", "media")])
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to download book file: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::sync_failed(format!(
                "Drive download error {}: {}",
                status, body
            )));
        }

        let bytes = response.bytes().await
            .map_err(|e| AppError::sync_failed(format!("Failed to read book file bytes: {}", e)))?;

        // Ensure target directory exists
        if let Some(parent) = Path::new(target_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::sync_failed(format!("Failed to create target directory: {}", e)))?;
        }

        fs::write(target_path, &bytes)
            .map_err(|e| AppError::sync_failed(format!("Failed to write book file: {}", e)))?;

        log::info!("Downloaded book file {} to {}", file_hash, target_path);

        Ok(())
    }

    /// List all book files in appData folder
    pub async fn list_book_files(&self) -> Result<Vec<DriveBookFile>, AppError> {
        let client = reqwest::Client::new();
        
        let response = client
            .get(format!("{}/files", DRIVE_API_BASE))
            .bearer_auth(&self.access_token)
            .query(&[
                ("spaces", "appDataFolder"),
                ("q", "name contains 'book_' and name contains '.cbz'"),
                ("fields", "files(id, name, size, modifiedTime)"),
                ("pageSize", "1000"),
            ])
            .send()
            .await
            .map_err(|e| AppError::sync_failed(format!("Failed to list book files: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::sync_failed(format!(
                "Drive API error {}: {}",
                status, body
            )));
        }

        #[derive(serde::Deserialize)]
        struct FileList {
            files: Vec<FileInfo>,
        }
        
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct FileInfo {
            id: String,
            name: String,
            size: Option<String>,
            modified_time: Option<String>,
        }

        let file_list: FileList = response.json().await
            .map_err(|e| AppError::sync_failed(format!("Failed to parse file list: {}", e)))?;

        let book_files = file_list.files.into_iter()
            .filter_map(|f| {
                // Extract hash from filename like "book_abc123.cbz"
                let hash = f.name.strip_prefix("book_")?.strip_suffix(".cbz")?.to_string();
                Some(DriveBookFile {
                    file_hash: hash,
                })
            })
            .collect();

        Ok(book_files)
    }
}

/// Info about a book file stored in Google Drive
#[derive(Debug, Clone)]
pub struct DriveBookFile {
    pub file_hash: String,
}

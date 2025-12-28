//! Merge engine for synchronizing local database with remote snapshot
//!
//! Implements the pull-merge-push algorithm for conflict resolution.

use diesel::prelude::*;
use std::collections::HashMap;
use tauri::AppHandle;

use crate::database::{get_connection, models::*};
use crate::error::AppError;
use crate::schema::{books, bookmarks, collections, book_collections, book_settings, sync_state};
use crate::settings::{load_settings, save_settings};

use super::types::*;

/// Merge engine for syncing local DB with remote snapshot
pub struct MergeEngine {
    device_id: String,
    strategy: ConflictStrategy,
    options: SyncOptions,
}

impl MergeEngine {
    pub fn new(device_id: String, strategy: ConflictStrategy, options: SyncOptions) -> Self {
        Self { device_id, strategy, options }
    }

    /// Execute a full sync: pull remote, merge, push updates
    pub fn sync(
        &self,
        app_handle: &AppHandle,
        remote: Option<SyncSnapshot>,
    ) -> Result<(SyncSnapshot, SyncResult), AppError> {
        let mut conn = get_connection()?;
        let mut result = SyncResult::empty();
        
        // Get or create remote snapshot
        let mut snapshot = remote.unwrap_or_else(SyncSnapshot::new);
        
        // Get last sync timestamp from local state
        let sync_state_record: Option<SyncState> = sync_state::table
            .find(1)
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::database_error(e.to_string()))?;
        
        let last_sync_at = sync_state_record
            .as_ref()
            .and_then(|s| s.last_sync_at)
            .map(|dt| to_timestamp(&dt))
            .unwrap_or(0);

        // Merge each entity type based on options
        // sync_books: Full book metadata sync (creates new books, syncs all fields)
        // sync_progress: Only syncs progress fields for books that already exist locally
        if self.options.sync_books {
            self.merge_books(&mut conn, &mut snapshot, last_sync_at, &mut result, true)?;
            self.merge_collections(&mut conn, &mut snapshot, last_sync_at, &mut result)?;
            self.merge_book_collections(&mut conn, &mut snapshot, last_sync_at, &mut result)?;
        } else if self.options.sync_progress {
            // Only sync progress for existing books
            self.merge_books(&mut conn, &mut snapshot, last_sync_at, &mut result, false)?;
        }
        
        // Bookmarks are part of reading progress
        if self.options.sync_progress {
            self.merge_bookmarks(&mut conn, &mut snapshot, last_sync_at, &mut result)?;
            self.merge_book_settings(&mut conn, &mut snapshot, last_sync_at, &mut result)?;
        }

        // App settings sync (separate from book settings)
        if self.options.sync_settings {
            self.merge_app_settings(app_handle, &mut snapshot, last_sync_at)?;
        }

        // Update snapshot metadata
        snapshot.last_modified_by = Some(self.device_id.clone());
        snapshot.last_modified_at = chrono::Utc::now().timestamp_millis();

        // Update local sync state
        let now = chrono::Utc::now().naive_utc();
        diesel::update(sync_state::table.find(1))
            .set((
                sync_state::last_sync_at.eq(Some(now)),
                sync_state::last_sync_device.eq(Some(&self.device_id)),
            ))
            .execute(&mut conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;

        result.success = result.errors.is_empty();
        result.completed_at = chrono::Utc::now().timestamp_millis();

        Ok((snapshot, result))
    }

    /// Merge books between local DB and remote snapshot
    /// 
    /// - `full_sync`: If true, creates new books from remote and syncs all fields.
    ///                If false, only syncs progress fields (current_page, reading_status, last_read_at)
    ///                for books that already exist locally.
    fn merge_books(
        &self,
        conn: &mut diesel::SqliteConnection,
        snapshot: &mut SyncSnapshot,
        last_sync_at: i64,
        result: &mut SyncResult,
        full_sync: bool,
    ) -> Result<(), AppError> {
        // Load all local books (including soft-deleted)
        let local_books: Vec<Book> = books::table
            .load(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Build UUID -> Book map for local books
        let local_by_uuid: HashMap<String, &Book> = local_books
            .iter()
            .filter_map(|b| b.uuid.as_ref().map(|uuid| (uuid.clone(), b)))
            .collect();
        
        // Build file_hash -> Book map for matching by content
        let local_by_hash: HashMap<String, &Book> = local_books
            .iter()
            .filter_map(|b| b.file_hash.as_ref().map(|hash| (hash.clone(), b)))
            .collect();

        // Process remote books
        for (uuid, remote_book) in snapshot.books.iter() {
            match local_by_uuid.get(uuid) {
                Some(local_book) => {
                    // Both exist - resolve conflict
                    let local_ts = to_timestamp(&local_book.updated_at);
                    let remote_ts = remote_book.updated_at;

                    let action = self.resolve_conflict(
                        local_ts,
                        remote_ts,
                        last_sync_at,
                        remote_book.deleted_at.is_some(),
                        local_book.deleted_at.is_some(),
                    );

                    match action {
                        ConflictAction::UseRemote => {
                            if full_sync {
                                // Full sync - update all fields
                                self.update_local_book(conn, local_book.id, remote_book)?;
                            } else {
                                // Progress only - only update progress fields
                                self.update_local_book_progress(conn, local_book.id, remote_book)?;
                            }
                            result.books_downloaded += 1;
                        }
                        ConflictAction::UseLocal => {
                            // Will update remote below in the "new local" loop
                        }
                        ConflictAction::NoOp => {}
                    }
                }
                None => {
                    // Remote book not in local by UUID
                    if remote_book.deleted_at.is_none() {
                        // Check if a book with the same file_hash exists (same book, different UUID)
                        let existing_by_hash = remote_book.file_hash.as_ref()
                            .and_then(|hash| local_by_hash.get(hash).copied());

                        if let Some(existing) = existing_by_hash {
                            // Book with same hash exists - update its UUID and merge progress
                            log::info!("Found existing book by hash, updating UUID: {} -> {}", 
                                existing.uuid.as_deref().unwrap_or("none"), uuid);
                            
                            if full_sync {
                                diesel::update(books::table.find(existing.id))
                                    .set((
                                        books::uuid.eq(Some(uuid)),
                                        books::title.eq(&remote_book.title),
                                        books::current_page.eq(remote_book.current_page),
                                        books::is_favorite.eq(remote_book.is_favorite),
                                        books::reading_status.eq(&remote_book.reading_status),
                                        books::last_read_at.eq(from_opt_timestamp(remote_book.last_read_at)),
                                        books::updated_at.eq(from_timestamp(remote_book.updated_at)),
                                    ))
                                    .execute(conn)
                                    .map_err(|e| AppError::database_error(e.to_string()))?;
                            } else {
                                // Progress only - just update UUID and progress fields
                                diesel::update(books::table.find(existing.id))
                                    .set((
                                        books::uuid.eq(Some(uuid)),
                                        books::current_page.eq(remote_book.current_page),
                                        books::reading_status.eq(&remote_book.reading_status),
                                        books::last_read_at.eq(from_opt_timestamp(remote_book.last_read_at)),
                                    ))
                                    .execute(conn)
                                    .map_err(|e| AppError::database_error(e.to_string()))?;
                            }
                            result.books_downloaded += 1;
                        } else if full_sync {
                            // Truly new book - only insert if full_sync
                            self.insert_local_book(conn, remote_book)?;
                            result.books_downloaded += 1;
                        }
                        // If not full_sync and book doesn't exist locally, skip it
                    }
                }
            }
        }

        // Process local books that might be new or updated
        for local_book in &local_books {
            let uuid = match &local_book.uuid {
                Some(u) => u.clone(),
                None => continue, // Skip books without UUID (shouldn't happen after migration)
            };

            let local_ts = to_timestamp(&local_book.updated_at);

            match snapshot.books.get(&uuid) {
                Some(remote_book) => {
                    // Already processed above, but check if local is newer
                    let remote_ts = remote_book.updated_at;
                    
                    if local_ts > remote_ts && local_ts > last_sync_at {
                        // Local is newer - update remote
                        if full_sync {
                            snapshot.books.insert(uuid, self.book_to_remote(local_book));
                        } else {
                            // Progress only - only upload progress fields
                            let mut remote = remote_book.clone();
                            remote.current_page = local_book.current_page;
                            remote.reading_status = local_book.reading_status.clone();
                            remote.last_read_at = local_book.last_read_at.as_ref().map(|dt| to_timestamp(dt));
                            remote.updated_at = local_ts;
                            snapshot.books.insert(uuid, remote);
                        }
                        result.books_uploaded += 1;
                    }
                }
                None => {
                    if full_sync {
                        // New local book - add to remote (only if full_sync)
                        snapshot.books.insert(uuid, self.book_to_remote(local_book));
                        result.books_uploaded += 1;
                    }
                    // If not full_sync, don't add new books to remote
                }
            }
        }

        Ok(())
    }
    
    /// Update only progress fields for a local book
    fn update_local_book_progress(
        &self,
        conn: &mut diesel::SqliteConnection,
        book_id: i32,
        remote: &RemoteBookState,
    ) -> Result<(), AppError> {
        diesel::update(books::table.find(book_id))
            .set((
                books::current_page.eq(remote.current_page),
                books::reading_status.eq(&remote.reading_status),
                books::last_read_at.eq(from_opt_timestamp(remote.last_read_at)),
                books::updated_at.eq(from_timestamp(remote.updated_at)),
            ))
            .execute(conn)
            .map_err(|e: diesel::result::Error| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    /// Merge collections
    fn merge_collections(
        &self,
        conn: &mut diesel::SqliteConnection,
        snapshot: &mut SyncSnapshot,
        last_sync_at: i64,
        result: &mut SyncResult,
    ) -> Result<(), AppError> {
        let local_collections: Vec<Collection> = collections::table
            .load(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let local_by_uuid: HashMap<String, &Collection> = local_collections
            .iter()
            .filter_map(|c| c.uuid.as_ref().map(|uuid| (uuid.clone(), c)))
            .collect();

        // Process remote collections
        for (uuid, remote_coll) in snapshot.collections.iter() {
            match local_by_uuid.get(uuid) {
                Some(local_coll) => {
                    let local_ts = to_timestamp(&local_coll.updated_at);
                    let remote_ts = remote_coll.updated_at;

                    let action = self.resolve_conflict(
                        local_ts,
                        remote_ts,
                        last_sync_at,
                        remote_coll.deleted_at.is_some(),
                        local_coll.deleted_at.is_some(),
                    );

                    if matches!(action, ConflictAction::UseRemote) {
                        self.update_local_collection(conn, local_coll.id, remote_coll)?;
                        result.collections_downloaded += 1;
                    }
                }
                None => {
                    if remote_coll.deleted_at.is_none() {
                        self.insert_local_collection(conn, remote_coll)?;
                        result.collections_downloaded += 1;
                    }
                }
            }
        }

        // Process local collections
        for local_coll in &local_collections {
            let uuid = match &local_coll.uuid {
                Some(u) => u.clone(),
                None => continue,
            };

            let local_ts = to_timestamp(&local_coll.updated_at);

            match snapshot.collections.get(&uuid) {
                Some(remote_coll) => {
                    if local_ts > remote_coll.updated_at && local_ts > last_sync_at {
                        snapshot.collections.insert(uuid, self.collection_to_remote(local_coll));
                        result.collections_uploaded += 1;
                    }
                }
                None => {
                    snapshot.collections.insert(uuid, self.collection_to_remote(local_coll));
                    result.collections_uploaded += 1;
                }
            }
        }

        Ok(())
    }

    /// Merge bookmarks
    fn merge_bookmarks(
        &self,
        conn: &mut diesel::SqliteConnection,
        snapshot: &mut SyncSnapshot,
        last_sync_at: i64,
        result: &mut SyncResult,
    ) -> Result<(), AppError> {
        let local_bookmarks: Vec<Bookmark> = bookmarks::table
            .load(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Build book_id -> uuid mapping
        let book_uuid_map: HashMap<i32, String> = books::table
            .select((books::id, books::uuid))
            .load::<(i32, Option<String>)>(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?
            .into_iter()
            .filter_map(|(id, uuid)| uuid.map(|u| (id, u)))
            .collect();

        let local_by_uuid: HashMap<String, &Bookmark> = local_bookmarks
            .iter()
            .filter_map(|b| b.uuid.as_ref().map(|uuid| (uuid.clone(), b)))
            .collect();

        // Process remote bookmarks
        for (uuid, remote_bm) in snapshot.bookmarks.iter() {
            match local_by_uuid.get(uuid) {
                Some(local_bm) => {
                    let local_ts = local_bm.updated_at.map(|dt| to_timestamp(&dt)).unwrap_or(0);
                    let remote_ts = remote_bm.updated_at;

                    let action = self.resolve_conflict(
                        local_ts,
                        remote_ts,
                        last_sync_at,
                        remote_bm.deleted_at.is_some(),
                        local_bm.deleted_at.is_some(),
                    );

                    if matches!(action, ConflictAction::UseRemote) {
                        self.update_local_bookmark(conn, local_bm.id, remote_bm)?;
                        result.bookmarks_downloaded += 1;
                    }
                }
                None => {
                    if remote_bm.deleted_at.is_none() {
                        // Find local book_id for this bookmark's book_uuid
                        if let Some(book_id) = self.find_book_id_by_uuid(conn, &remote_bm.book_uuid)? {
                            self.insert_local_bookmark(conn, remote_bm, book_id)?;
                            result.bookmarks_downloaded += 1;
                        }
                    }
                }
            }
        }

        // Process local bookmarks
        for local_bm in &local_bookmarks {
            let uuid = match &local_bm.uuid {
                Some(u) => u.clone(),
                None => continue,
            };

            let book_uuid = match book_uuid_map.get(&local_bm.book_id) {
                Some(u) => u.clone(),
                None => continue,
            };

            let local_ts = local_bm.updated_at.map(|dt| to_timestamp(&dt)).unwrap_or(0);

            match snapshot.bookmarks.get(&uuid) {
                Some(remote_bm) => {
                    if local_ts > remote_bm.updated_at && local_ts > last_sync_at {
                        snapshot.bookmarks.insert(uuid, self.bookmark_to_remote(local_bm, &book_uuid));
                        result.bookmarks_uploaded += 1;
                    }
                }
                None => {
                    snapshot.bookmarks.insert(uuid, self.bookmark_to_remote(local_bm, &book_uuid));
                    result.bookmarks_uploaded += 1;
                }
            }
        }

        Ok(())
    }

    /// Merge book-collection relationships
    fn merge_book_collections(
        &self,
        conn: &mut diesel::SqliteConnection,
        snapshot: &mut SyncSnapshot,
        _last_sync_at: i64,
        _result: &mut SyncResult,
    ) -> Result<(), AppError> {
        let local_bcs: Vec<BookCollection> = book_collections::table
            .load(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Build ID -> UUID mappings for local data
        let book_uuid_map: HashMap<i32, String> = books::table
            .select((books::id, books::uuid))
            .load::<(i32, Option<String>)>(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?
            .into_iter()
            .filter_map(|(id, uuid)| uuid.map(|u| (id, u)))
            .collect();

        let coll_uuid_map: HashMap<i32, String> = collections::table
            .select((collections::id, collections::uuid))
            .load::<(i32, Option<String>)>(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?
            .into_iter()
            .filter_map(|(id, uuid)| uuid.map(|u| (id, u)))
            .collect();

        // Build reverse mappings: UUID -> ID
        let book_id_map: HashMap<String, i32> = book_uuid_map.iter()
            .map(|(id, uuid)| (uuid.clone(), *id))
            .collect();
        let coll_id_map: HashMap<String, i32> = coll_uuid_map.iter()
            .map(|(id, uuid)| (uuid.clone(), *id))
            .collect();

        // Build set of local book-collection UUIDs
        let local_bc_uuids: std::collections::HashSet<String> = local_bcs
            .iter()
            .filter_map(|bc| bc.uuid.clone())
            .collect();

        // Download: Insert remote book_collections that don't exist locally
        for (uuid, remote_bc) in snapshot.book_collections.iter() {
            if remote_bc.deleted_at.is_some() {
                continue; // Skip deleted
            }
            
            if local_bc_uuids.contains(uuid) {
                continue; // Already exists locally
            }

            // Resolve UUIDs to local IDs
            let book_id = match book_id_map.get(&remote_bc.book_uuid) {
                Some(id) => *id,
                None => {
                    log::debug!("Skipping book_collection {}: book {} not found locally", uuid, remote_bc.book_uuid);
                    continue;
                }
            };
            let coll_id = match coll_id_map.get(&remote_bc.collection_uuid) {
                Some(id) => *id,
                None => {
                    log::debug!("Skipping book_collection {}: collection {} not found locally", uuid, remote_bc.collection_uuid);
                    continue;
                }
            };

            // Check if this book-collection combo already exists (different UUID but same relationship)
            let existing: Option<BookCollection> = book_collections::table
                .filter(book_collections::book_id.eq(book_id))
                .filter(book_collections::collection_id.eq(coll_id))
                .first(conn)
                .optional()
                .map_err(|e| AppError::database_error(e.to_string()))?;

            if existing.is_none() {
                log::info!("Inserting book_collection {} (book {} -> collection {})", uuid, book_id, coll_id);
                diesel::insert_into(book_collections::table)
                    .values((
                        book_collections::uuid.eq(uuid),
                        book_collections::book_id.eq(book_id),
                        book_collections::collection_id.eq(coll_id),
                        book_collections::added_at.eq(from_timestamp(remote_bc.added_at)),
                    ))
                    .execute(conn)
                    .map_err(|e| AppError::database_error(e.to_string()))?;
            }
        }

        // Upload: Add local book_collections to snapshot
        for local_bc in &local_bcs {
            let uuid = match &local_bc.uuid {
                Some(u) => u.clone(),
                None => continue,
            };

            let book_uuid = match book_uuid_map.get(&local_bc.book_id) {
                Some(u) => u.clone(),
                None => continue,
            };

            let coll_uuid = match coll_uuid_map.get(&local_bc.collection_id) {
                Some(u) => u.clone(),
                None => continue,
            };

            if !snapshot.book_collections.contains_key(&uuid) {
                snapshot.book_collections.insert(uuid.clone(), RemoteBookCollectionState {
                    uuid,
                    book_uuid,
                    collection_uuid: coll_uuid,
                    added_at: to_timestamp(&local_bc.added_at),
                    updated_at: local_bc.updated_at.map(|dt| to_timestamp(&dt)).unwrap_or_else(|| to_timestamp(&local_bc.added_at)),
                    deleted_at: local_bc.deleted_at.map(|dt| to_timestamp(&dt)),
                });
            }
        }

        Ok(())
    }

    /// Merge book settings
    fn merge_book_settings(
        &self,
        conn: &mut diesel::SqliteConnection,
        snapshot: &mut SyncSnapshot,
        _last_sync_at: i64,
        _result: &mut SyncResult,
    ) -> Result<(), AppError> {
        let local_settings: Vec<BookSettings> = book_settings::table
            .load(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let book_uuid_map: HashMap<i32, String> = books::table
            .select((books::id, books::uuid))
            .load::<(i32, Option<String>)>(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?
            .into_iter()
            .filter_map(|(id, uuid)| uuid.map(|u| (id, u)))
            .collect();

        // Build reverse mapping: UUID -> ID
        let book_id_map: HashMap<String, i32> = book_uuid_map.iter()
            .map(|(id, uuid)| (uuid.clone(), *id))
            .collect();

        // Build set of local book_settings UUIDs
        let local_bs_uuids: std::collections::HashSet<String> = local_settings
            .iter()
            .filter_map(|bs| bs.uuid.clone())
            .collect();

        // Download: Insert remote book_settings that don't exist locally
        for (uuid, remote_bs) in snapshot.book_settings.iter() {
            if remote_bs.deleted_at.is_some() {
                continue;
            }

            if local_bs_uuids.contains(uuid) {
                continue;
            }

            let book_id = match book_id_map.get(&remote_bs.book_uuid) {
                Some(id) => *id,
                None => {
                    log::debug!("Skipping book_settings {}: book {} not found locally", uuid, remote_bs.book_uuid);
                    continue;
                }
            };

            // Check if settings already exist for this book (different UUID)
            let existing: Option<BookSettings> = book_settings::table
                .filter(book_settings::book_id.eq(book_id))
                .first(conn)
                .optional()
                .map_err(|e| AppError::database_error(e.to_string()))?;

            if existing.is_none() {
                log::info!("Inserting book_settings {} for book {}", uuid, book_id);
                diesel::insert_into(book_settings::table)
                    .values((
                        book_settings::uuid.eq(uuid),
                        book_settings::book_id.eq(book_id),
                        book_settings::reading_direction.eq(&remote_bs.reading_direction),
                        book_settings::page_display_mode.eq(&remote_bs.page_display_mode),
                        book_settings::image_fit_mode.eq(&remote_bs.image_fit_mode),
                        book_settings::reader_background.eq(&remote_bs.reader_background),
                        book_settings::sync_progress.eq(remote_bs.sync_progress),
                    ))
                    .execute(conn)
                    .map_err(|e| AppError::database_error(e.to_string()))?;
            }
        }

        // Upload: Add local book_settings to snapshot
        for local_bs in &local_settings {
            let uuid = match &local_bs.uuid {
                Some(u) => u.clone(),
                None => continue,
            };

            let book_uuid = match book_uuid_map.get(&local_bs.book_id) {
                Some(u) => u.clone(),
                None => continue,
            };

            if !snapshot.book_settings.contains_key(&uuid) {
                snapshot.book_settings.insert(uuid.clone(), RemoteBookSettingsState {
                    uuid,
                    book_uuid,
                    reading_direction: local_bs.reading_direction.clone(),
                    page_display_mode: local_bs.page_display_mode.clone(),
                    image_fit_mode: local_bs.image_fit_mode.clone(),
                    reader_background: local_bs.reader_background.clone(),
                    sync_progress: local_bs.sync_progress,
                    updated_at: to_timestamp(&local_bs.updated_at),
                    deleted_at: local_bs.deleted_at.map(|dt| to_timestamp(&dt)),
                });
            }
        }

        Ok(())
    }

    /// Merge app settings (the settings.json file)
    fn merge_app_settings(
        &self,
        app_handle: &AppHandle,
        snapshot: &mut SyncSnapshot,
        _last_sync_at: i64,
    ) -> Result<(), AppError> {
        use crate::settings::SettingValue;

        // Load local settings
        let local_settings = load_settings(app_handle)?;
        
        // Convert local settings to JSON map
        let mut local_map: HashMap<String, serde_json::Value> = HashMap::new();
        for category in &local_settings.categories {
            for setting in &category.settings {
                // Skip sync settings themselves to avoid circular issues
                if setting.key.starts_with("sync.") {
                    continue;
                }
                let value = match &setting.value {
                    SettingValue::Bool(b) => serde_json::Value::Bool(*b),
                    SettingValue::String(s) => serde_json::Value::String(s.clone()),
                    SettingValue::Number(n) => serde_json::json!(*n),
                    SettingValue::Float(f) => serde_json::json!(*f),
                };
                local_map.insert(setting.key.clone(), value);
            }
        }

        // Determine which settings to use based on timestamps
        let local_updated_at = local_settings.updated_at;
        let remote_updated_at = snapshot.app_settings_updated_at;

        if snapshot.app_settings.is_empty() || local_updated_at > remote_updated_at {
            // Local is newer or remote is empty - upload local settings
            log::info!("Uploading local app settings to remote");
            snapshot.app_settings = local_map;
            snapshot.app_settings_updated_at = local_updated_at;
        } else if remote_updated_at > 0 {
            // Remote is newer - download remote settings
            log::info!("Downloading remote app settings to local");
            let mut settings = local_settings;
            
            for (key, value) in &snapshot.app_settings {
                let setting_value = match value {
                    serde_json::Value::Bool(b) => SettingValue::Bool(*b),
                    serde_json::Value::String(s) => SettingValue::String(s.clone()),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            SettingValue::Number(i)
                        } else if let Some(f) = n.as_f64() {
                            SettingValue::Float(f)
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };
                settings.set(key, setting_value);
            }
            
            save_settings(app_handle, &settings)?;
        }

        Ok(())
    }

    // ========================================================================
    // CONFLICT RESOLUTION
    // ========================================================================

    fn resolve_conflict(
        &self,
        local_ts: i64,
        remote_ts: i64,
        _last_sync_at: i64,
        remote_deleted: bool,
        local_deleted: bool,
    ) -> ConflictAction {
        // Deletion always wins (if either side deleted, it stays deleted)
        if remote_deleted && !local_deleted {
            return ConflictAction::UseRemote;
        }
        if local_deleted && !remote_deleted {
            return ConflictAction::UseLocal;
        }

        match self.strategy {
            ConflictStrategy::RemoteWins => ConflictAction::UseRemote,
            ConflictStrategy::LocalWins => ConflictAction::UseLocal,
            ConflictStrategy::LastWriteWins => {
                if remote_ts > local_ts {
                    ConflictAction::UseRemote
                } else if local_ts > remote_ts {
                    ConflictAction::UseLocal
                } else {
                    ConflictAction::NoOp
                }
            }
        }
    }

    // ========================================================================
    // LOCAL DB UPDATE HELPERS
    // ========================================================================

    fn update_local_book(
        &self,
        conn: &mut diesel::SqliteConnection,
        book_id: i32,
        remote: &RemoteBookState,
    ) -> Result<(), AppError> {
        diesel::update(books::table.find(book_id))
            .set((
                books::title.eq(&remote.title),
                books::current_page.eq(remote.current_page),
                books::total_pages.eq(remote.total_pages),
                books::is_favorite.eq(remote.is_favorite),
                books::reading_status.eq(&remote.reading_status),
                books::last_read_at.eq(from_opt_timestamp(remote.last_read_at)),
                books::updated_at.eq(from_timestamp(remote.updated_at)),
                books::deleted_at.eq(from_opt_timestamp(remote.deleted_at)),
            ))
            .execute(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    fn insert_local_book(
        &self,
        conn: &mut diesel::SqliteConnection,
        remote: &RemoteBookState,
    ) -> Result<(), AppError> {
        // Generate a unique placeholder path for cloud-only books
        // This will be replaced with the actual path when the file is downloaded
        let placeholder_path = format!("cloud://{}", remote.uuid);
        
        diesel::insert_into(books::table)
            .values((
                books::uuid.eq(&remote.uuid),
                books::file_path.eq(&placeholder_path),
                books::filename.eq(&remote.filename),
                books::file_hash.eq(&remote.file_hash),
                books::title.eq(&remote.title),
                books::current_page.eq(remote.current_page),
                books::total_pages.eq(remote.total_pages),
                books::is_favorite.eq(remote.is_favorite),
                books::reading_status.eq(&remote.reading_status),
                books::last_read_at.eq(from_opt_timestamp(remote.last_read_at)),
                books::added_at.eq(from_timestamp(remote.added_at)),
                books::updated_at.eq(from_timestamp(remote.updated_at)),
            ))
            .execute(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    fn update_local_collection(
        &self,
        conn: &mut diesel::SqliteConnection,
        collection_id: i32,
        remote: &RemoteCollectionState,
    ) -> Result<(), AppError> {
        diesel::update(collections::table.find(collection_id))
            .set((
                collections::name.eq(&remote.name),
                collections::description.eq(&remote.description),
                collections::updated_at.eq(from_timestamp(remote.updated_at)),
                collections::deleted_at.eq(from_opt_timestamp(remote.deleted_at)),
            ))
            .execute(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    fn insert_local_collection(
        &self,
        conn: &mut diesel::SqliteConnection,
        remote: &RemoteCollectionState,
    ) -> Result<(), AppError> {
        diesel::insert_into(collections::table)
            .values((
                collections::uuid.eq(&remote.uuid),
                collections::name.eq(&remote.name),
                collections::description.eq(&remote.description),
                collections::created_at.eq(from_timestamp(remote.created_at)),
                collections::updated_at.eq(from_timestamp(remote.updated_at)),
            ))
            .execute(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    fn update_local_bookmark(
        &self,
        conn: &mut diesel::SqliteConnection,
        bookmark_id: i32,
        remote: &RemoteBookmarkState,
    ) -> Result<(), AppError> {
        diesel::update(bookmarks::table.find(bookmark_id))
            .set((
                bookmarks::name.eq(&remote.name),
                bookmarks::description.eq(&remote.description),
                bookmarks::page.eq(remote.page),
                bookmarks::updated_at.eq(Some(from_timestamp(remote.updated_at))),
                bookmarks::deleted_at.eq(from_opt_timestamp(remote.deleted_at)),
            ))
            .execute(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    fn insert_local_bookmark(
        &self,
        conn: &mut diesel::SqliteConnection,
        remote: &RemoteBookmarkState,
        book_id: i32,
    ) -> Result<(), AppError> {
        diesel::insert_into(bookmarks::table)
            .values((
                bookmarks::uuid.eq(&remote.uuid),
                bookmarks::book_id.eq(book_id),
                bookmarks::name.eq(&remote.name),
                bookmarks::description.eq(&remote.description),
                bookmarks::page.eq(remote.page),
                bookmarks::created_at.eq(from_timestamp(remote.created_at)),
                bookmarks::updated_at.eq(Some(from_timestamp(remote.updated_at))),
            ))
            .execute(conn)
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    fn find_book_id_by_uuid(
        &self,
        conn: &mut diesel::SqliteConnection,
        book_uuid: &str,
    ) -> Result<Option<i32>, AppError> {
        let result: Option<i32> = books::table
            .filter(books::uuid.eq(book_uuid))
            .select(books::id)
            .first(conn)
            .optional()
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(result)
    }

    // ========================================================================
    // LOCAL -> REMOTE CONVERSION HELPERS
    // ========================================================================

    fn book_to_remote(&self, book: &Book) -> RemoteBookState {
        RemoteBookState {
            uuid: book.uuid.clone().unwrap_or_default(),
            file_hash: book.file_hash.clone(),
            title: book.title.clone(),
            filename: book.filename.clone(),
            current_page: book.current_page,
            total_pages: book.total_pages,
            is_favorite: book.is_favorite,
            reading_status: book.reading_status.clone(),
            last_read_at: to_opt_timestamp(&book.last_read_at),
            added_at: to_timestamp(&book.added_at),
            updated_at: to_timestamp(&book.updated_at),
            deleted_at: to_opt_timestamp(&book.deleted_at),
        }
    }

    fn collection_to_remote(&self, collection: &Collection) -> RemoteCollectionState {
        RemoteCollectionState {
            uuid: collection.uuid.clone().unwrap_or_default(),
            name: collection.name.clone(),
            description: collection.description.clone(),
            created_at: to_timestamp(&collection.created_at),
            updated_at: to_timestamp(&collection.updated_at),
            deleted_at: to_opt_timestamp(&collection.deleted_at),
        }
    }

    fn bookmark_to_remote(&self, bookmark: &Bookmark, book_uuid: &str) -> RemoteBookmarkState {
        RemoteBookmarkState {
            uuid: bookmark.uuid.clone().unwrap_or_default(),
            book_uuid: book_uuid.to_string(),
            name: bookmark.name.clone(),
            description: bookmark.description.clone(),
            page: bookmark.page,
            created_at: to_timestamp(&bookmark.created_at),
            updated_at: bookmark.updated_at.map(|dt| to_timestamp(&dt)).unwrap_or_else(|| to_timestamp(&bookmark.created_at)),
            deleted_at: to_opt_timestamp(&bookmark.deleted_at),
        }
    }
}

/// Result of conflict resolution
#[derive(Debug, Clone, Copy)]
enum ConflictAction {
    UseRemote,
    UseLocal,
    NoOp,
}

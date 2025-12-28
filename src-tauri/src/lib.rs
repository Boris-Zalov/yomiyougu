//! yomiyougu - A cross-platform manga/comic reader
//!
//! ## Module Structure
//! - `auth/` - Google OAuth token management
//! - `commands/` - Tauri commands exposed to frontend
//! - `database/` - Diesel ORM models and connection management
//! - `settings/` - Configuration management with UI schema generation
//! - `sync/` - Google Drive synchronization
//! - `error` - Application-wide error types
//! - `schema` - Auto-generated Diesel schema

pub mod auth;
mod commands;
mod database;
mod error;
mod schema;
mod settings;
mod sync;

pub use database::{establish_connection, DbPool};
pub use error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger - respect RUST_LOG env var, default to Info
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting yomiyougu application");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            database::connection::init_pool(app.handle())?;
            log::info!("Database connection pool initialized");
            log::info!("Stronghold secure storage available for credential management");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            commands::get_auth_status,
            commands::google_sign_in,
            commands::refresh_google_token,
            commands::google_logout,
            commands::set_auth_token,
            commands::save_google_auth_token,
            // Settings commands
            commands::check_settings_exists,
            commands::get_settings,
            commands::get_settings_schema,
            commands::get_setting,
            commands::save_settings_from_schema,
            commands::update_setting,
            commands::complete_setup,
            commands::reset_all_settings,
            commands::reset_setting,
            // Library commands - collections
            commands::create_collection,
            commands::get_collections,
            commands::get_collection,
            commands::update_collection,
            commands::delete_collection,
            // Library commands - books
            commands::get_books,
            commands::get_book,
            commands::update_book,
            commands::delete_book,
            commands::import_book_from_archive,
            // Library commands - book-collection management
            commands::set_book_collections,
            commands::add_book_to_collection,
            commands::remove_book_from_collection,
            // Library commands - book settings
            commands::get_book_settings,
            commands::update_book_settings,
            // Sync commands
            commands::get_sync_status,
            commands::sync_now,
        ])
        .run(tauri::generate_context!())
        .expect("Critical error while running tauri application");
}

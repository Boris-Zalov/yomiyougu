//! yomiyougu - A cross-platform manga/comic reader
//!
//! ## Module Structure
//! - `commands/` - Tauri commands exposed to frontend
//! - `database/` - Diesel ORM models and connection management
//! - `settings/` - Configuration management with UI schema generation
//! - `error` - Application-wide error types
//! - `schema` - Auto-generated Diesel schema

mod commands;
mod database;
mod error;
mod schema;
mod settings;

pub use database::{establish_connection, DbPool};
pub use error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting yomiyougu application");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            database::connection::init_pool(app.handle())?;
            log::info!("Database connection pool initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            commands::import_books_from_archive,
        ])
        .run(tauri::generate_context!())
        .expect("Critical error while running tauri application");
}

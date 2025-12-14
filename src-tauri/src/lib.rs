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
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            database::connection::init_pool(app.handle())?;
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
        ])
        .run(tauri::generate_context!())
        .expect("Critical error while running tauri application");
}

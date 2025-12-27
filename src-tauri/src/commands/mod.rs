//! Tauri commands module - exposes Rust functionality to the frontend
//!
//! Commands are organized by feature area. All commands should:
//! - Use Result<T, String> return type for Tauri compatibility
//! - Convert AppError to String for frontend consumption
//! - Follow snake_case naming (invoked as camelCase from JS)

mod auth;
mod library;
mod settings;

pub use auth::*;
pub use library::*;
pub use settings::*;

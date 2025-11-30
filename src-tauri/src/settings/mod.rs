//! Settings module - handles application configuration with UI schema generation
//!
//! This module provides:
//! - Type-safe settings storage with categories
//! - Self-describing schema for dynamic UI rendering
//! - Default values appropriate for manga/comic reading

mod schema;
mod storage;
mod types;

pub use schema::*;
pub use storage::*;
pub use types::*;

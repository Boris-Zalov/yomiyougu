//! Database module for Diesel ORM integration
//!
//! ## Structure
//! - `models` - Diesel model structs for database tables
//! - `connection` - Connection pool management
//! - `operations` - CRUD operations for books and collections

pub mod connection;
pub mod models;
pub mod operations;

#[cfg(test)]
mod tests;

pub use connection::{establish_connection, DbPool};
pub use models::*;
pub use operations::*;

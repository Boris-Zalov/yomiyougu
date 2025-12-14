//! Database module for Diesel ORM integration
//!
//! ## Structure
//! - `models` - Diesel model structs for database tables
//! - `connection` - Connection pool management

pub mod connection;
pub mod models;

pub use connection::{establish_connection, DbPool};
pub use models::*;

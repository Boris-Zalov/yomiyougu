//! Authentication module for Google OAuth token management
//!
//! Handles storage and retrieval of OAuth tokens for Google Drive sync.
//! Uses secure file storage in the app's config directory.

mod storage;
mod types;

pub use storage::*;
pub use types::*;

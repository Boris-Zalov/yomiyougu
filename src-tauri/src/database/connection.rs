//! Database connection pool management
//!
//! Uses r2d2 for connection pooling with SQLite

use diesel::connection::SimpleConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use std::sync::OnceLock;
use tauri::AppHandle;
use tauri::Manager;

use crate::error::{AppError, ErrorCode};

/// Type alias for the connection pool
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Global database pool instance
static DB_POOL: OnceLock<DbPool> = OnceLock::new();

/// Get the path to the database file
fn get_database_path(app: &AppHandle) -> Result<String, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::new(
            ErrorCode::DatabasePathError,
            format!("Failed to get app data directory: {}", e),
        )
    })?;

    // Ensure directory exists
    std::fs::create_dir_all(&app_data_dir).map_err(|e| {
        AppError::new(
            ErrorCode::DatabasePathError,
            format!("Failed to create app data directory: {}", e),
        )
    })?;

    let db_path = app_data_dir.join("yomiyougu.db");
    Ok(db_path.to_string_lossy().to_string())
}

/// Custom connection initializer to configure SQLite for concurrent access
#[derive(Debug)]
struct SqliteConnectionCustomizer;

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for SqliteConnectionCustomizer
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        // Enable WAL mode
        conn.batch_execute("PRAGMA journal_mode = WAL;")
            .map_err(|e| diesel::r2d2::Error::QueryError(e))?;
        // Set busy timeout to 10 seconds
        conn.batch_execute("PRAGMA busy_timeout = 10000;")
            .map_err(|e| diesel::r2d2::Error::QueryError(e))?;
        // Enable foreign keys
        conn.batch_execute("PRAGMA foreign_keys = ON;")
            .map_err(|e| diesel::r2d2::Error::QueryError(e))?;
        Ok(())
    }
}

/// Initialize the database connection pool
pub fn init_pool(app: &AppHandle) -> Result<(), AppError> {
    let database_url = get_database_path(app)?;

    let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .max_size(10)
        .connection_customizer(Box::new(SqliteConnectionCustomizer))
        .build(manager)
        .map_err(|e| {
            AppError::new(
                ErrorCode::DatabaseConnectionFailed,
                format!("Failed to create pool: {}", e),
            )
        })?;

    // Run pending migrations
    run_migrations(&pool)?;

    DB_POOL.set(pool).map_err(|_| {
        AppError::new(
            ErrorCode::DatabaseConnectionFailed,
            "Database pool already initialized",
        )
    })?;

    Ok(())
}

/// Run pending database migrations
fn run_migrations(pool: &DbPool) -> Result<(), AppError> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    let mut conn = pool.get().map_err(|e| {
        AppError::new(
            ErrorCode::DatabaseConnectionFailed,
            format!("Failed to get connection for migrations: {}", e),
        )
    })?;

    conn.run_pending_migrations(MIGRATIONS).map_err(|e| {
        AppError::new(
            ErrorCode::DatabaseMigrationFailed,
            format!("Failed to run migrations: {}", e),
        )
    })?;

    Ok(())
}

/// Get a connection from the pool
pub fn establish_connection(
) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, AppError> {
    let pool = DB_POOL.get().ok_or_else(|| {
        AppError::new(
            ErrorCode::DatabaseNotInitialized,
            "Database not initialized",
        )
    })?;

    pool.get().map_err(|e| {
        AppError::new(
            ErrorCode::DatabaseConnectionFailed,
            format!("Failed to get database connection: {}", e),
        )
    })
}

/// Alias for establish_connection (for backwards compatibility)
pub fn get_connection(
) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, AppError> {
    establish_connection()
}

-- Books table for manga/comics
CREATE TABLE books (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    -- File information
    file_path TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    file_size INTEGER,
    file_hash TEXT,
    -- Display information
    title TEXT NOT NULL,
    -- Reading progress
    current_page INTEGER NOT NULL DEFAULT 1,
    total_pages INTEGER NOT NULL DEFAULT 0,
    -- Timestamps
    last_read_at TIMESTAMP,
    added_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Organization
    collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    -- Reading status
    reading_status TEXT NOT NULL DEFAULT 'unread' CHECK(reading_status IN ('unread', 'reading', 'completed', 'on_hold', 'dropped'))
);

-- Indexes for common queries
CREATE INDEX idx_books_collection ON books(collection_id);
CREATE INDEX idx_books_favorite ON books(is_favorite);
CREATE INDEX idx_books_last_read ON books(last_read_at);
CREATE INDEX idx_books_status ON books(reading_status);
CREATE INDEX idx_books_title ON books(title);

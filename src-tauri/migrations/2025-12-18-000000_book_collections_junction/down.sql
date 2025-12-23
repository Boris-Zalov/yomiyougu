-- Revert: Add cover_path back to collections
CREATE TABLE collections_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    cover_path TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO collections_old (id, name, description, created_at, updated_at)
SELECT id, name, description, created_at, updated_at FROM collections;

DROP TABLE collections;
ALTER TABLE collections_old RENAME TO collections;
CREATE INDEX idx_collections_name ON collections(name);

-- Revert: Add collection_id back to books
CREATE TABLE books_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    file_size INTEGER,
    file_hash TEXT,
    title TEXT NOT NULL,
    current_page INTEGER NOT NULL DEFAULT 1,
    total_pages INTEGER NOT NULL DEFAULT 0,
    last_read_at TIMESTAMP,
    added_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    reading_status TEXT NOT NULL DEFAULT 'unread' CHECK(reading_status IN ('unread', 'reading', 'completed', 'on_hold', 'dropped'))
);

-- Copy data back, taking the first collection_id from junction table if exists
INSERT INTO books_old (id, file_path, filename, file_size, file_hash, title, current_page, total_pages, last_read_at, added_at, updated_at, collection_id, is_favorite, reading_status)
SELECT b.id, b.file_path, b.filename, b.file_size, b.file_hash, b.title, b.current_page, b.total_pages, b.last_read_at, b.added_at, b.updated_at,
       (SELECT bc.collection_id FROM book_collections bc WHERE bc.book_id = b.id LIMIT 1),
       b.is_favorite, b.reading_status
FROM books b;

DROP TABLE books;
ALTER TABLE books_old RENAME TO books;

CREATE INDEX idx_books_collection ON books(collection_id);
CREATE INDEX idx_books_favorite ON books(is_favorite);
CREATE INDEX idx_books_last_read ON books(last_read_at);
CREATE INDEX idx_books_status ON books(reading_status);
CREATE INDEX idx_books_title ON books(title);

-- Drop the junction table
DROP TABLE book_collections;

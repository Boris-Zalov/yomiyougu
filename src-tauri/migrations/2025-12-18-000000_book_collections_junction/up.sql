-- Junction table for many-to-many book-collection relationships
CREATE TABLE book_collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    added_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(book_id, collection_id)
);

CREATE INDEX idx_book_collections_book ON book_collections(book_id);
CREATE INDEX idx_book_collections_collection ON book_collections(collection_id);

-- Migrate existing collection_id data from books to junction table
INSERT INTO book_collections (book_id, collection_id)
SELECT id, collection_id FROM books WHERE collection_id IS NOT NULL;

-- Remove collection_id from books table (SQLite requires table rebuild)
CREATE TABLE books_new (
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
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    reading_status TEXT NOT NULL DEFAULT 'unread' CHECK(reading_status IN ('unread', 'reading', 'completed', 'on_hold', 'dropped'))
);

INSERT INTO books_new (id, file_path, filename, file_size, file_hash, title, current_page, total_pages, last_read_at, added_at, updated_at, is_favorite, reading_status)
SELECT id, file_path, filename, file_size, file_hash, title, current_page, total_pages, last_read_at, added_at, updated_at, is_favorite, reading_status FROM books;

DROP TABLE books;
ALTER TABLE books_new RENAME TO books;

CREATE INDEX idx_books_favorite ON books(is_favorite);
CREATE INDEX idx_books_last_read ON books(last_read_at);
CREATE INDEX idx_books_status ON books(reading_status);
CREATE INDEX idx_books_title ON books(title);

-- Remove cover_path from collections table (SQLite requires table rebuild)
CREATE TABLE collections_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO collections_new (id, name, description, created_at, updated_at)
SELECT id, name, description, created_at, updated_at FROM collections;

DROP TABLE collections;
ALTER TABLE collections_new RENAME TO collections;

CREATE INDEX idx_collections_name ON collections(name);

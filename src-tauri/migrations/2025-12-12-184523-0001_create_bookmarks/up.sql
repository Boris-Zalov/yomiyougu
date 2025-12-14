-- Bookmarks for saving specific pages in books
CREATE TABLE bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    page INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for faster book bookmark lookups
CREATE INDEX idx_bookmarks_book ON bookmarks(book_id);

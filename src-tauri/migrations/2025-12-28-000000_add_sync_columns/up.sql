-- Add sync-related columns to all tables that need to be synchronized

ALTER TABLE books ADD COLUMN uuid TEXT;
ALTER TABLE books ADD COLUMN deleted_at TIMESTAMP;

UPDATE books SET uuid = lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))) WHERE uuid IS NULL;

CREATE UNIQUE INDEX idx_books_uuid ON books(uuid);

ALTER TABLE bookmarks ADD COLUMN uuid TEXT;
ALTER TABLE bookmarks ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE bookmarks ADD COLUMN deleted_at TIMESTAMP;

UPDATE bookmarks SET uuid = lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))) WHERE uuid IS NULL;

CREATE UNIQUE INDEX idx_bookmarks_uuid ON bookmarks(uuid);

ALTER TABLE collections ADD COLUMN uuid TEXT;
ALTER TABLE collections ADD COLUMN deleted_at TIMESTAMP;

UPDATE collections SET uuid = lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))) WHERE uuid IS NULL;

CREATE UNIQUE INDEX idx_collections_uuid ON collections(uuid);

ALTER TABLE book_settings ADD COLUMN uuid TEXT;
ALTER TABLE book_settings ADD COLUMN deleted_at TIMESTAMP;

UPDATE book_settings SET uuid = lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))) WHERE uuid IS NULL;

CREATE UNIQUE INDEX idx_book_settings_uuid ON book_settings(uuid);

ALTER TABLE book_collections ADD COLUMN uuid TEXT;
ALTER TABLE book_collections ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE book_collections ADD COLUMN deleted_at TIMESTAMP;

UPDATE book_collections SET uuid = lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))) WHERE uuid IS NULL;

CREATE UNIQUE INDEX idx_book_collections_uuid ON book_collections(uuid);

CREATE TABLE sync_state (
    id INTEGER PRIMARY KEY NOT NULL DEFAULT 1,
    last_sync_at TIMESTAMP,
    last_sync_device TEXT,
    sync_file_id TEXT,
    CHECK (id = 1)  -- Ensure only one row exists
);

INSERT INTO sync_state (id) VALUES (1);

-- Book-specific settings overrides
-- These override the global settings for individual books
CREATE TABLE book_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    book_id INTEGER NOT NULL UNIQUE REFERENCES books(id) ON DELETE CASCADE,
    -- Reading settings overrides (NULL means use global setting)
    reading_direction TEXT CHECK(reading_direction IN ('ltr', 'rtl', 'vertical')),
    page_display_mode TEXT CHECK(page_display_mode IN ('single', 'double', 'auto')),
    image_fit_mode TEXT CHECK(image_fit_mode IN ('fit_width', 'fit_height', 'fit_screen', 'original')),
    reader_background TEXT CHECK(reader_background IN ('white', 'black', 'gray', 'sepia')),
    -- Sync settings
    sync_progress BOOLEAN,
    -- Timestamps
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for book lookup
CREATE INDEX idx_book_settings_book ON book_settings(book_id);

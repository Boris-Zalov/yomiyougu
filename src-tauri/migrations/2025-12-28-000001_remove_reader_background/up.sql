-- Remove reader_background column from book_settings
CREATE TABLE book_settings_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    book_id INTEGER NOT NULL UNIQUE REFERENCES books(id) ON DELETE CASCADE,
    reading_direction TEXT CHECK(reading_direction IN ('ltr', 'rtl', 'vertical')),
    page_display_mode TEXT CHECK(page_display_mode IN ('single', 'double', 'continuous')),
    image_fit_mode TEXT CHECK(image_fit_mode IN ('fit_width', 'fit_height', 'fit_screen', 'original')),
    sync_progress BOOLEAN,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    uuid TEXT UNIQUE,
    deleted_at TIMESTAMP
);

INSERT INTO book_settings_new (id, book_id, reading_direction, page_display_mode, image_fit_mode, sync_progress, updated_at, uuid, deleted_at)
SELECT id, book_id, reading_direction, page_display_mode, image_fit_mode, sync_progress, updated_at, uuid, deleted_at
FROM book_settings;

DROP TABLE book_settings;

ALTER TABLE book_settings_new RENAME TO book_settings;

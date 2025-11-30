// @generated automatically by Diesel CLI.

diesel::table! {
    book_settings (id) {
        id -> Integer,
        book_id -> Integer,
        reading_direction -> Nullable<Text>,
        page_display_mode -> Nullable<Text>,
        image_fit_mode -> Nullable<Text>,
        reader_background -> Nullable<Text>,
        sync_progress -> Nullable<Bool>,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    bookmarks (id) {
        id -> Integer,
        book_id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        page -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    books (id) {
        id -> Integer,
        file_path -> Text,
        filename -> Text,
        file_size -> Nullable<Integer>,
        file_hash -> Nullable<Text>,
        title -> Text,
        current_page -> Integer,
        total_pages -> Integer,
        last_read_at -> Nullable<Timestamp>,
        added_at -> Timestamp,
        updated_at -> Timestamp,
        collection_id -> Nullable<Integer>,
        is_favorite -> Bool,
        reading_status -> Text,
    }
}

diesel::table! {
    collections (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        cover_path -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(book_settings -> books (book_id));
diesel::joinable!(bookmarks -> books (book_id));
diesel::joinable!(books -> collections (collection_id));

diesel::allow_tables_to_appear_in_same_query!(book_settings, bookmarks, books, collections,);

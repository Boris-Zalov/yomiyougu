//! Database unit tests
//!
//! Uses in-memory SQLite database for isolation.
//! Each test gets a fresh database with migrations applied.

#[cfg(test)]
mod database_tests {
    use diesel::prelude::*;
    use diesel::r2d2::{ConnectionManager, Pool};
    use diesel::sqlite::SqliteConnection;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    use crate::database::models::*;
    use crate::schema::*;

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    type TestPool = Pool<ConnectionManager<SqliteConnection>>;

    /// Generate a test UUID
    fn test_uuid() -> Option<String> {
        Some(uuid::Uuid::new_v4().to_string())
    }

    /// Create a fresh in-memory database with migrations applied
    fn setup_test_db() -> TestPool {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("Failed to create test pool");

        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        pool
    }

    // ========================================================================
    // COLLECTION TESTS
    // ========================================================================

    mod collection_tests {
        use super::*;

        #[test]
        fn test_create_collection() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let new_collection = NewCollection {
                uuid: test_uuid(),
                name: "Manga".to_string(),
                description: Some("Japanese comics".to_string()),
            };

            let result = diesel::insert_into(collections::table)
                .values(&new_collection)
                .returning(Collection::as_returning())
                .get_result(&mut conn);

            assert!(result.is_ok());
            let collection = result.unwrap();
            assert_eq!(collection.name, "Manga");
            assert_eq!(collection.description, Some("Japanese comics".to_string()));
            assert!(collection.id > 0);
        }

        #[test]
        fn test_collection_unique_name_constraint() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let collection1 = NewCollection {
                uuid: test_uuid(),
                name: "Manga".to_string(),
                description: None,
            };

            let collection2 = NewCollection {
                uuid: test_uuid(),
                name: "Manga".to_string(), // Same name
                description: Some("Duplicate".to_string()),
            };

            // First insert should succeed
            diesel::insert_into(collections::table)
                .values(&collection1)
                .execute(&mut conn)
                .expect("First insert should succeed");

            // Second insert with same name should fail
            let result = diesel::insert_into(collections::table)
                .values(&collection2)
                .execute(&mut conn);

            assert!(result.is_err());
        }

        #[test]
        fn test_update_collection() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            // Create collection
            let new_collection = NewCollection {
                uuid: test_uuid(),
                name: "Old Name".to_string(),
                description: None,
            };

            let collection: Collection = diesel::insert_into(collections::table)
                .values(&new_collection)
                .returning(Collection::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Update collection
            let update = UpdateCollection {
                name: Some("New Name".to_string()),
                description: Some(Some("Updated description".to_string())),
                updated_at: Some(chrono::Utc::now().naive_utc()),
            };

            diesel::update(collections::table.find(collection.id))
                .set(&update)
                .execute(&mut conn)
                .expect("Update should succeed");

            // Verify update
            let updated: Collection = collections::table
                .find(collection.id)
                .first(&mut conn)
                .unwrap();

            assert_eq!(updated.name, "New Name");
            assert_eq!(updated.description, Some("Updated description".to_string()));
        }

        #[test]
        fn test_delete_collection() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let new_collection = NewCollection {
                uuid: test_uuid(),
                name: "To Delete".to_string(),
                description: None,
            };

            let collection: Collection = diesel::insert_into(collections::table)
                .values(&new_collection)
                .returning(Collection::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Delete
            diesel::delete(collections::table.find(collection.id))
                .execute(&mut conn)
                .expect("Delete should succeed");

            // Verify deletion
            let result: Result<Collection, _> =
                collections::table.find(collection.id).first(&mut conn);

            assert!(result.is_err());
        }

        #[test]
        fn test_list_collections() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            // Create multiple collections
            for i in 1..=3 {
                let new_collection = NewCollection {
                    uuid: test_uuid(),
                    name: format!("Collection {}", i),
                    description: None,
                };
                diesel::insert_into(collections::table)
                    .values(&new_collection)
                    .execute(&mut conn)
                    .unwrap();
            }

            let all_collections: Vec<Collection> = collections::table
                .order(collections::name.asc())
                .load(&mut conn)
                .unwrap();

            assert_eq!(all_collections.len(), 3);
            assert_eq!(all_collections[0].name, "Collection 1");
            assert_eq!(all_collections[2].name, "Collection 3");
        }
    }

    // ========================================================================
    // BOOK TESTS
    // ========================================================================

    mod book_tests {
        use super::*;

        fn create_test_book(conn: &mut SqliteConnection, title: &str) -> Book {
            let new_book = NewBook {
                uuid: test_uuid(),
                file_path: format!("/path/to/{}.cbz", title.to_lowercase().replace(' ', "_")),
                filename: format!("{}.cbz", title),
                file_size: Some(1024000),
                file_hash: Some("abc123".to_string()),
                title: title.to_string(),
                total_pages: 100,
            };

            diesel::insert_into(books::table)
                .values(&new_book)
                .returning(Book::as_returning())
                .get_result(conn)
                .unwrap()
        }

        #[test]
        fn test_create_book() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn, "Test Manga");

            assert_eq!(book.title, "Test Manga");
            assert_eq!(book.current_page, 1); // Default
            assert_eq!(book.total_pages, 100);
            assert!(!book.is_favorite); // Default false
            assert_eq!(book.reading_status, "unread"); // Default
        }

        #[test]
        fn test_book_unique_file_path() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book1 = NewBook {
                uuid: test_uuid(),
                file_path: "/path/to/manga.cbz".to_string(),
                filename: "manga.cbz".to_string(),
                file_size: None,
                file_hash: None,
                title: "Manga 1".to_string(),
                total_pages: 50,
            };

            let book2 = NewBook {
                uuid: test_uuid(),
                file_path: "/path/to/manga.cbz".to_string(), // Same path
                filename: "manga.cbz".to_string(),
                file_size: None,
                file_hash: None,
                title: "Manga 2".to_string(),
                total_pages: 60,
            };

            diesel::insert_into(books::table)
                .values(&book1)
                .execute(&mut conn)
                .expect("First insert should succeed");

            let result = diesel::insert_into(books::table)
                .values(&book2)
                .execute(&mut conn);

            assert!(result.is_err(), "Duplicate file_path should fail");
        }

        #[test]
        fn test_update_book_progress() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn, "Reading Progress Test");

            // Update reading progress
            let update = UpdateBook {
                current_page: Some(50),
                reading_status: Some("reading".to_string()),
                last_read_at: Some(Some(chrono::Utc::now().naive_utc())),
                ..Default::default()
            };

            diesel::update(books::table.find(book.id))
                .set(&update)
                .execute(&mut conn)
                .unwrap();

            let updated: Book = books::table.find(book.id).first(&mut conn).unwrap();

            assert_eq!(updated.current_page, 50);
            assert_eq!(updated.reading_status, "reading");
            assert!(updated.last_read_at.is_some());
        }

        #[test]
        fn test_book_progress_calculation() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let mut book = create_test_book(&mut conn, "Progress Calc Test");
            book.current_page = 25;
            book.total_pages = 100;

            assert_eq!(book.progress(), 25.0);

            book.current_page = 100;
            assert_eq!(book.progress(), 100.0);

            book.total_pages = 0;
            assert_eq!(book.progress(), 0.0); // Avoid division by zero
        }

        #[test]
        fn test_book_status_enum() {
            assert_eq!(ReadingStatus::Unread.as_str(), "unread");
            assert_eq!(ReadingStatus::Reading.as_str(), "reading");
            assert_eq!(ReadingStatus::Completed.as_str(), "completed");
            assert_eq!(ReadingStatus::OnHold.as_str(), "on_hold");
            assert_eq!(ReadingStatus::Dropped.as_str(), "dropped");

            assert_eq!(
                ReadingStatus::from_str("reading"),
                Some(ReadingStatus::Reading)
            );
            assert_eq!(ReadingStatus::from_str("invalid"), None);
        }

        #[test]
        fn test_toggle_favorite() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn, "Favorite Test");
            assert!(!book.is_favorite);

            // Toggle to favorite
            diesel::update(books::table.find(book.id))
                .set(books::is_favorite.eq(true))
                .execute(&mut conn)
                .unwrap();

            let updated: Book = books::table.find(book.id).first(&mut conn).unwrap();
            assert!(updated.is_favorite);
        }

        #[test]
        fn test_book_with_collection() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            // Create collection
            let collection: Collection = diesel::insert_into(collections::table)
                .values(&NewCollection {
                    uuid: test_uuid(),
                    name: "Shonen".to_string(),
                    description: None,
                })
                .returning(Collection::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Create book
            let new_book = NewBook {
                uuid: test_uuid(),
                file_path: "/manga/naruto.cbz".to_string(),
                filename: "naruto.cbz".to_string(),
                file_size: None,
                file_hash: None,
                title: "Naruto".to_string(),
                total_pages: 200,
            };

            let book: Book = diesel::insert_into(books::table)
                .values(&new_book)
                .returning(Book::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Add book to collection via junction table
            diesel::insert_into(book_collections::table)
                .values(&NewBookCollection {
                    uuid: test_uuid(),
                    book_id: book.id,
                    collection_id: collection.id,
                })
                .execute(&mut conn)
                .unwrap();

            // Query books in collection via junction table
            let books_in_collection: Vec<Book> = books::table
                .inner_join(book_collections::table.on(book_collections::book_id.eq(books::id)))
                .filter(book_collections::collection_id.eq(collection.id))
                .select(Book::as_select())
                .load(&mut conn)
                .unwrap();

            assert_eq!(books_in_collection.len(), 1);
        }

        #[test]
        fn test_collection_deletion_removes_junction() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            // Create collection
            let collection: Collection = diesel::insert_into(collections::table)
                .values(&NewCollection {
                    uuid: test_uuid(),
                    name: "To Delete".to_string(),
                    description: None,
                })
                .returning(Collection::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Create book
            let book: Book = diesel::insert_into(books::table)
                .values(&NewBook {
                    uuid: test_uuid(),
                    file_path: "/manga/test.cbz".to_string(),
                    filename: "test.cbz".to_string(),
                    file_size: None,
                    file_hash: None,
                    title: "Test".to_string(),
                    total_pages: 50,
                })
                .returning(Book::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Add book to collection via junction table
            diesel::insert_into(book_collections::table)
                .values(&NewBookCollection {
                    uuid: test_uuid(),
                    book_id: book.id,
                    collection_id: collection.id,
                })
                .execute(&mut conn)
                .unwrap();

            let junction_count: i64 = book_collections::table
                .filter(book_collections::book_id.eq(book.id))
                .count()
                .get_result(&mut conn)
                .unwrap();
            assert_eq!(junction_count, 1);

            // Delete collection
            diesel::delete(collections::table.find(collection.id))
                .execute(&mut conn)
                .unwrap();

            // Book should still exist
            let book_exists: bool = books::table.find(book.id).first::<Book>(&mut conn).is_ok();
            assert!(
                book_exists,
                "Book should still exist after collection deletion"
            );

            // Junction entry should be deleted (CASCADE)
            let remaining: i64 = book_collections::table
                .filter(book_collections::book_id.eq(book.id))
                .count()
                .get_result(&mut conn)
                .unwrap();

            assert_eq!(
                remaining, 0,
                "Junction entries should be deleted after collection deletion"
            );
        }

        #[test]
        fn test_filter_by_reading_status() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            // Create books with different statuses
            for (i, status) in ["unread", "reading", "completed", "reading"]
                .iter()
                .enumerate()
            {
                let book = NewBook {
                    uuid: test_uuid(),
                    file_path: format!("/manga/book{}.cbz", i),
                    filename: format!("book{}.cbz", i),
                    file_size: None,
                    file_hash: None,
                    title: format!("Book {}", i),
                    total_pages: 100,
                };

                let inserted: Book = diesel::insert_into(books::table)
                    .values(&book)
                    .returning(Book::as_returning())
                    .get_result(&mut conn)
                    .unwrap();

                diesel::update(books::table.find(inserted.id))
                    .set(books::reading_status.eq(*status))
                    .execute(&mut conn)
                    .unwrap();
            }

            let reading_books: Vec<Book> = books::table
                .filter(books::reading_status.eq("reading"))
                .load(&mut conn)
                .unwrap();

            assert_eq!(reading_books.len(), 2);
        }
    }

    // ========================================================================
    // BOOKMARK TESTS
    // ========================================================================

    mod bookmark_tests {
        use super::*;

        fn create_test_book(conn: &mut SqliteConnection) -> Book {
            let new_book = NewBook {
                uuid: test_uuid(),
                file_path: "/path/to/test.cbz".to_string(),
                filename: "test.cbz".to_string(),
                file_size: None,
                file_hash: None,
                title: "Test Book".to_string(),
                total_pages: 100,
            };

            diesel::insert_into(books::table)
                .values(&new_book)
                .returning(Book::as_returning())
                .get_result(conn)
                .unwrap()
        }

        #[test]
        fn test_create_bookmark() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn);

            let new_bookmark = NewBookmark {
                uuid: test_uuid(),
                book_id: book.id,
                name: "Cool Scene".to_string(),
                description: Some("The hero's entrance".to_string()),
                page: 42,
            };

            let bookmark: Bookmark = diesel::insert_into(bookmarks::table)
                .values(&new_bookmark)
                .returning(Bookmark::as_returning())
                .get_result(&mut conn)
                .unwrap();

            assert_eq!(bookmark.name, "Cool Scene");
            assert_eq!(bookmark.page, 42);
            assert_eq!(bookmark.book_id, book.id);
        }

        #[test]
        fn test_multiple_bookmarks_per_book() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn);

            for i in 1..=5 {
                diesel::insert_into(bookmarks::table)
                    .values(&NewBookmark {
                        uuid: test_uuid(),
                        book_id: book.id,
                        name: format!("Bookmark {}", i),
                        description: None,
                        page: i * 10,
                    })
                    .execute(&mut conn)
                    .unwrap();
            }

            let book_bookmarks: Vec<Bookmark> = bookmarks::table
                .filter(bookmarks::book_id.eq(book.id))
                .order(bookmarks::page.asc())
                .load(&mut conn)
                .unwrap();

            assert_eq!(book_bookmarks.len(), 5);
            assert_eq!(book_bookmarks[0].page, 10);
            assert_eq!(book_bookmarks[4].page, 50);
        }

        #[test]
        fn test_cascade_delete_bookmarks() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn);

            // Add bookmarks
            diesel::insert_into(bookmarks::table)
                .values(&NewBookmark {
                    uuid: test_uuid(),
                    book_id: book.id,
                    name: "Bookmark".to_string(),
                    description: None,
                    page: 1,
                })
                .execute(&mut conn)
                .unwrap();

            let bookmark_count: i64 = bookmarks::table
                .filter(bookmarks::book_id.eq(book.id))
                .count()
                .get_result(&mut conn)
                .unwrap();

            assert_eq!(bookmark_count, 1);

            // Delete book
            diesel::delete(books::table.find(book.id))
                .execute(&mut conn)
                .unwrap();

            // Bookmarks should be deleted too (CASCADE)
            let remaining: i64 = bookmarks::table.count().get_result(&mut conn).unwrap();

            assert_eq!(remaining, 0, "Bookmarks should be cascade deleted");
        }
    }

    // ========================================================================
    // BOOK SETTINGS TESTS
    // ========================================================================

    mod book_settings_tests {
        use super::*;

        fn create_test_book(conn: &mut SqliteConnection) -> Book {
            let new_book = NewBook {
                uuid: test_uuid(),
                file_path: "/path/to/settings_test.cbz".to_string(),
                filename: "settings_test.cbz".to_string(),
                file_size: None,
                file_hash: None,
                title: "Settings Test Book".to_string(),
                total_pages: 100,
            };

            diesel::insert_into(books::table)
                .values(&new_book)
                .returning(Book::as_returning())
                .get_result(conn)
                .unwrap()
        }

        #[test]
        fn test_create_book_settings() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn);

            let new_settings = NewBookSettings {
                uuid: test_uuid(),
                book_id: book.id,
                reading_direction: Some("rtl".to_string()),
                page_display_mode: Some("double".to_string()),
                image_fit_mode: None,
                reader_background: Some("black".to_string()),
                sync_progress: Some(true),
            };

            let settings: BookSettings = diesel::insert_into(book_settings::table)
                .values(&new_settings)
                .returning(BookSettings::as_returning())
                .get_result(&mut conn)
                .unwrap();

            assert_eq!(settings.book_id, book.id);
            assert_eq!(settings.reading_direction, Some("rtl".to_string()));
            assert_eq!(settings.page_display_mode, Some("double".to_string()));
            assert!(settings.image_fit_mode.is_none());
            assert_eq!(settings.sync_progress, Some(true));
        }

        #[test]
        fn test_unique_settings_per_book() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn);

            let settings1 = NewBookSettings {
                uuid: test_uuid(),
                book_id: book.id,
                reading_direction: Some("ltr".to_string()),
                page_display_mode: None,
                image_fit_mode: None,
                reader_background: None,
                sync_progress: None,
            };

            diesel::insert_into(book_settings::table)
                .values(&settings1)
                .execute(&mut conn)
                .expect("First settings should succeed");

            let settings2 = NewBookSettings {
                uuid: test_uuid(),
                book_id: book.id, // Same book
                reading_direction: Some("rtl".to_string()),
                page_display_mode: None,
                image_fit_mode: None,
                reader_background: None,
                sync_progress: None,
            };

            let result = diesel::insert_into(book_settings::table)
                .values(&settings2)
                .execute(&mut conn);

            assert!(result.is_err(), "Duplicate book_id should fail");
        }

        #[test]
        fn test_cascade_delete_settings() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn);

            diesel::insert_into(book_settings::table)
                .values(&NewBookSettings {
                    uuid: test_uuid(),
                    book_id: book.id,
                    reading_direction: Some("rtl".to_string()),
                    page_display_mode: None,
                    image_fit_mode: None,
                    reader_background: None,
                    sync_progress: None,
                })
                .execute(&mut conn)
                .unwrap();

            // Delete book
            diesel::delete(books::table.find(book.id))
                .execute(&mut conn)
                .unwrap();

            // Settings should be deleted too
            let remaining: i64 = book_settings::table.count().get_result(&mut conn).unwrap();

            assert_eq!(remaining, 0, "Book settings should be cascade deleted");
        }

        #[test]
        fn test_reading_direction_enum() {
            assert_eq!(ReadingDirection::Ltr.as_str(), "ltr");
            assert_eq!(ReadingDirection::Rtl.as_str(), "rtl");
            assert_eq!(ReadingDirection::Vertical.as_str(), "vertical");

            assert_eq!(
                ReadingDirection::from_str("rtl"),
                Some(ReadingDirection::Rtl)
            );
            assert_eq!(ReadingDirection::from_str("invalid"), None);
        }

        #[test]
        fn test_update_book_settings() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let book = create_test_book(&mut conn);

            let settings: BookSettings = diesel::insert_into(book_settings::table)
                .values(&NewBookSettings {
                    uuid: test_uuid(),
                    book_id: book.id,
                    reading_direction: Some("ltr".to_string()),
                    page_display_mode: None,
                    image_fit_mode: None,
                    reader_background: None,
                    sync_progress: None,
                })
                .returning(BookSettings::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Update settings
            let update = UpdateBookSettings {
                reading_direction: Some(Some("rtl".to_string())),
                sync_progress: Some(Some(false)),
                ..Default::default()
            };

            diesel::update(book_settings::table.find(settings.id))
                .set(&update)
                .execute(&mut conn)
                .unwrap();

            let updated: BookSettings = book_settings::table
                .find(settings.id)
                .first(&mut conn)
                .unwrap();

            assert_eq!(updated.reading_direction, Some("rtl".to_string()));
            assert_eq!(updated.sync_progress, Some(false));
        }
    }

    // ========================================================================
    // QUERY TESTS
    // ========================================================================

    mod query_tests {
        use super::*;

        #[test]
        fn test_books_ordered_by_last_read() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let now = chrono::Utc::now().naive_utc();

            for i in 0..3 {
                let book: Book = diesel::insert_into(books::table)
                    .values(&NewBook {
                        uuid: test_uuid(),
                        file_path: format!("/manga/book{}.cbz", i),
                        filename: format!("book{}.cbz", i),
                        file_size: None,
                        file_hash: None,
                        title: format!("Book {}", i),
                        total_pages: 100,
                    })
                    .returning(Book::as_returning())
                    .get_result(&mut conn)
                    .unwrap();

                // Set last_read_at with different times
                let read_time = now - chrono::Duration::hours(i as i64);
                diesel::update(books::table.find(book.id))
                    .set(books::last_read_at.eq(Some(read_time)))
                    .execute(&mut conn)
                    .unwrap();
            }

            let recent_books: Vec<Book> = books::table
                .filter(books::last_read_at.is_not_null())
                .order(books::last_read_at.desc())
                .load(&mut conn)
                .unwrap();

            assert_eq!(recent_books.len(), 3);
            // Book 0 was read most recently
            assert_eq!(recent_books[0].title, "Book 0");
        }

        #[test]
        fn test_count_books_in_collection() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let collection: Collection = diesel::insert_into(collections::table)
                .values(&NewCollection {
                    uuid: test_uuid(),
                    name: "Test Collection".to_string(),
                    description: None,
                })
                .returning(Collection::as_returning())
                .get_result(&mut conn)
                .unwrap();

            // Add books and link to collection via junction table
            for i in 0..5 {
                let book: Book = diesel::insert_into(books::table)
                    .values(&NewBook {
                        uuid: test_uuid(),
                        file_path: format!("/manga/coll_book{}.cbz", i),
                        filename: format!("coll_book{}.cbz", i),
                        file_size: None,
                        file_hash: None,
                        title: format!("Collection Book {}", i),
                        total_pages: 50,
                    })
                    .returning(Book::as_returning())
                    .get_result(&mut conn)
                    .unwrap();

                diesel::insert_into(book_collections::table)
                    .values(&NewBookCollection {
                        uuid: test_uuid(),
                        book_id: book.id,
                        collection_id: collection.id,
                    })
                    .execute(&mut conn)
                    .unwrap();
            }

            let count: i64 = book_collections::table
                .filter(book_collections::collection_id.eq(collection.id))
                .count()
                .get_result(&mut conn)
                .unwrap();

            assert_eq!(count, 5);
        }

        #[test]
        fn test_search_books_by_title() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            let titles = ["One Piece", "Naruto", "One Punch Man", "Bleach"];

            for (i, title) in titles.iter().enumerate() {
                diesel::insert_into(books::table)
                    .values(&NewBook {
                        uuid: test_uuid(),
                        file_path: format!("/manga/search{}.cbz", i),
                        filename: format!("search{}.cbz", i),
                        file_size: None,
                        file_hash: None,
                        title: title.to_string(),
                        total_pages: 100,
                    })
                    .execute(&mut conn)
                    .unwrap();
            }

            let results: Vec<Book> = books::table
                .filter(books::title.like("%One%"))
                .load(&mut conn)
                .unwrap();

            assert_eq!(results.len(), 2);
        }

        #[test]
        fn test_favorites_query() {
            let pool = setup_test_db();
            let mut conn = pool.get().unwrap();

            for i in 0..5 {
                let book: Book = diesel::insert_into(books::table)
                    .values(&NewBook {
                        uuid: test_uuid(),
                        file_path: format!("/manga/fav{}.cbz", i),
                        filename: format!("fav{}.cbz", i),
                        file_size: None,
                        file_hash: None,
                        title: format!("Book {}", i),
                        total_pages: 100,
                    })
                    .returning(Book::as_returning())
                    .get_result(&mut conn)
                    .unwrap();

                // Make every other book a favorite
                if i % 2 == 0 {
                    diesel::update(books::table.find(book.id))
                        .set(books::is_favorite.eq(true))
                        .execute(&mut conn)
                        .unwrap();
                }
            }

            let favorites: Vec<Book> = books::table
                .filter(books::is_favorite.eq(true))
                .load(&mut conn)
                .unwrap();

            assert_eq!(favorites.len(), 3);
        }
    }
}

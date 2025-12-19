/**
 * TypeScript types matching the Rust library API
 * These mirror structures in src-tauri/src/database/models.rs
 */

/**
 * Reading status enum matching Rust ReadingStatus
 */
export type ReadingStatus = 'unread' | 'reading' | 'completed' | 'on_hold' | 'dropped';

/**
 * Interface mirroring the Rust 'Book' struct.
 * Note: NaiveDateTime types are represented as strings in TypeScript.
 */
export interface Book {
	id: number;
	file_path: string;
	filename: string;
	file_size: number | null;
	file_hash: string | null;
	title: string;
	current_page: number;
	total_pages: number;
	last_read_at: string | null;
	added_at: string;
	updated_at: string;
	is_favorite: boolean;
	reading_status: ReadingStatus;
}

/**
 * Book-specific settings overrides
 */
export interface BookSettings {
	id: number;
	book_id: number;
	reading_direction: string | null;
	page_display_mode: string | null;
	image_fit_mode: string | null;
	reader_background: string | null;
	sync_progress: boolean | null;
	updated_at: string;
}

/**
 * Interface mirroring the Rust 'BookWithDetails' struct.
 * Note: Uses #[serde(flatten)] so book fields are at the top level
 */
export interface BookWithDetails extends Book {
	collection_names: string[];
	collection_ids: number[];
	settings: BookSettings | null;
	bookmark_count: number;
}

/**
 * Collection model
 */
export interface Collection {
	id: number;
	name: string;
	description: string | null;
	created_at: string;
	updated_at: string;
}

/**
 * Collection with book count
 */
export interface CollectionWithCount extends Collection {
	book_count: number;
}

/**
 * Information about a skipped book during import
 */
export interface SkippedBook {
	title: string;
	reason: string;
	existing_book_id: number | null;
}

/**
 * Result of importing books from an archive
 */
export interface ImportResult {
	imported: Book[];
	skipped: SkippedBook[];
}

/**
 * Temporary function for generating the cover image path.
 * TODO: Replace with actual cover extraction/caching logic
 * @param bookId - The ID of the book.
 * @returns The URL for the cover image.
 */
export function getCoverPath(bookId: number): string {
	return `/api/covers/${bookId}.jpg`;
}

/**
 * Calculate reading progress percentage
 */
export function calculateProgress(book: Book): number {
	if (book.total_pages === 0) return 0;
	return Math.round((book.current_page / book.total_pages) * 100);
}
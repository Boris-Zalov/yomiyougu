/**
 * TypeScript types matching the Rust library API
 * These mirror structures in src-tauri/src/database/models.rs
 */

/**
 * Reading status enum matching Rust ReadingStatus
 */
export type ReadingStatus = "unread" | "reading" | "completed" | "on_hold" | "dropped";

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
	sync_progress: boolean | null;
	updated_at: string;
}

/**
 * Bookmark model for saving specific pages
 */
export interface Bookmark {
	id: number;
	book_id: number;
	name: string;
	description: string | null;
	page: number;
	created_at: string;
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
 * Check if a book is in RAR/CBR format (unsupported on Android)
 * Works for both local paths and cloud books (checks filename)
 */
export function isRarFormat(book: Book | BookWithDetails): boolean {
	const filename = book.filename.toLowerCase();
	return filename.endsWith(".rar") || filename.endsWith(".cbr");
}

/**
 * Get the protocol URL prefix based on platform.
 * On Android/Windows, custom protocols use http://<scheme>.localhost format.
 * On macOS/Linux/iOS, they use <scheme>://localhost format.
 */
let cachedIsAndroid: boolean | null = null;

export function setIsAndroid(value: boolean): void {
	cachedIsAndroid = value;
}

export function getIsAndroid(): boolean {
	return cachedIsAndroid === true;
}

function getComicProtocolPrefix(): string {
	// On Android, use http://comic.localhost format
	if (cachedIsAndroid === true) {
		return "http://comic.localhost";
	}
	// On desktop (macOS/Linux), use comic://localhost format
	return "comic://localhost";
}

/**
 * Get the cover image path for a book.
 * Uses the comic:// custom protocol to serve the first page as cover.
 * @param bookId - The ID of the book.
 * @returns The URL for the cover image via custom protocol.
 */
export function getCoverPath(bookId: number): string {
	return `${getComicProtocolPrefix()}/book/${bookId}/page/0`;
}

/**
 * Get the image path for a specific page of a book.
 * Uses the comic:// custom protocol to serve images from archives.
 * @param bookId - The ID of the book.
 * @param pageNumber - The page number (0-indexed).
 * @returns The URL for the page image via custom protocol.
 */
export function getPagePath(bookId: number, pageNumber: number): string {
	return `${getComicProtocolPrefix()}/book/${bookId}/page/${pageNumber}`;
}

/**
 * Calculate reading progress percentage
 * Note: current_page is 0-indexed, so 1 is added for display
 */
export function calculateProgress(book: Book): number {
	if (book.total_pages === 0) return 0;
	return Math.round(((book.current_page + 1) / book.total_pages) * 100);
}

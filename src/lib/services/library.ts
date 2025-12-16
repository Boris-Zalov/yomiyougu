/**
 * Library API service
 * Centralized Tauri command invocations for library management (books and collections)
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  Book,
  BookWithDetails,
  Collection,
  CollectionWithCount,
  ImportResult,
  ReadingStatus,
} from "$lib/types/library";

// ============================================================================
// COLLECTION COMMANDS
// ============================================================================

/**
 * Create a new collection
 */
export async function createCollection(
  name: string,
  description?: string,
  coverPath?: string
): Promise<Collection> {
  return invoke<Collection>("create_collection", {
    name,
    description: description ?? null,
    coverPath: coverPath ?? null,
  });
}

/**
 * Get all collections with book counts
 */
export async function getCollections(): Promise<CollectionWithCount[]> {
  return invoke<CollectionWithCount[]>("get_collections");
}

/**
 * Get a single collection by ID
 */
export async function getCollection(collectionId: number): Promise<Collection> {
  return invoke<Collection>("get_collection", { collectionId });
}

/**
 * Update a collection
 */
export async function updateCollection(
  collectionId: number,
  updates: {
    name?: string;
    description?: string | null;
    coverPath?: string | null;
  }
): Promise<Collection> {
  return invoke<Collection>("update_collection", {
    collectionId,
    name: updates.name,
    description: updates.description,
    coverPath: updates.coverPath,
  });
}

/**
 * Delete a collection
 */
export async function deleteCollection(collectionId: number): Promise<void> {
  return invoke<void>("delete_collection", { collectionId });
}

// ============================================================================
// BOOK COMMANDS
// ============================================================================

/**
 * Get all books with optional filtering
 */
export async function getBooks(options?: {
  collectionId?: number;
  status?: ReadingStatus;
  favoritesOnly?: boolean;
}): Promise<BookWithDetails[]> {
  return invoke<BookWithDetails[]>("get_books", {
    collectionId: options?.collectionId ?? null,
    status: options?.status ?? null,
    favoritesOnly: options?.favoritesOnly ?? false,
  });
}

/**
 * Get a single book by ID
 */
export async function getBook(bookId: number): Promise<Book> {
  return invoke<Book>("get_book", { bookId });
}

/**
 * Update a book
 */
export async function updateBook(
  bookId: number,
  updates: {
    title?: string;
    currentPage?: number;
    collectionId?: number | null;
    isFavorite?: boolean;
    readingStatus?: ReadingStatus;
  }
): Promise<Book> {
  return invoke<Book>("update_book", {
    bookId,
    title: updates.title,
    currentPage: updates.currentPage,
    collectionId: updates.collectionId,
    isFavorite: updates.isFavorite,
    readingStatus: updates.readingStatus,
  });
}

/**
 * Delete a book
 */
export async function deleteBook(bookId: number): Promise<void> {
  return invoke<void>("delete_book", { bookId });
}

/**
 * Import books from a zip/cbz archive file
 * @param filePath - Path to the archive file
 * @param collectionId - Optional collection to add imported books to
 * @returns ImportResult with imported books and skipped duplicates
 */
export async function importBooksFromArchive(
  filePath: string,
  collectionId?: number
): Promise<ImportResult> {
  return invoke<ImportResult>("import_books_from_archive", {
    filePath,
    collectionId: collectionId ?? null,
  });
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/**
 * Toggle a book's favorite status
 */
export async function toggleFavorite(book: Book): Promise<Book> {
  return updateBook(book.id, { isFavorite: !book.is_favorite });
}

/**
 * Update reading progress (also updates last_read_at automatically)
 */
export async function updateReadingProgress(
  bookId: number,
  currentPage: number
): Promise<Book> {
  return updateBook(bookId, { currentPage });
}

/**
 * Mark a book as completed (sets status and current page to total)
 */
export async function markAsCompleted(book: Book): Promise<Book> {
  return updateBook(book.id, {
    readingStatus: "completed",
    currentPage: book.total_pages,
  });
}

/**
 * Start reading a book (sets status to reading)
 */
export async function startReading(bookId: number): Promise<Book> {
  return updateBook(bookId, { readingStatus: "reading" });
}

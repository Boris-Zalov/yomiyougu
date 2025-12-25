/**
 * Library API service
 * Centralized Tauri command invocations for library management (books and collections)
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  Book,
  BookWithDetails,
  BookSettings,
  Collection,
  CollectionWithCount,
  ReadingStatus,
} from "$lib/types/library";

/**
 * Extract filename from a file path or Android content URI
 * @param path - File path or content URI
 * @returns Extracted filename or undefined
 */
function extractFilename(path: string): string | undefined {
  // For Android content URIs
  if (path.startsWith('content://')) {
    try {
      const decoded = decodeURIComponent(path);
      
      // "primary:Download/filename.cbz" pattern
      const primaryMatch = decoded.match(/primary:[^/]+\/([^/]+)$/);
      if (primaryMatch) return primaryMatch[1];
      
      // raw:/storage/.../filename.cbz" pattern
      const rawMatch = decoded.match(/raw:\/.*\/([^/]+)$/);
      if (rawMatch) return rawMatch[1];
      
      // Look for anything after the last slash that looks like a filename
      const parts = decoded.split('/');
      const lastPart = parts[parts.length - 1];
      if (lastPart && /\.(cbz|cbr|zip|rar)$/i.test(lastPart)) {
        return lastPart;
      }
    } catch (e) {
      console.warn('Failed to parse content URI:', e);
    }
    return undefined;
  }
  
  // For regular file paths, extract the filename
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1];
}

// ============================================================================
// COLLECTION COMMANDS
// ============================================================================

/**
 * Create a new collection
 */
export async function createCollection(
  name: string,
  description?: string
): Promise<Collection> {
  return invoke<Collection>("create_collection", {
    name,
    description: description ?? null,
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
  }
): Promise<Collection> {
  return invoke<Collection>("update_collection", {
    collectionId,
    name: updates.name,
    description: updates.description,
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
    isFavorite?: boolean;
    readingStatus?: ReadingStatus;
  }
): Promise<Book> {
  return invoke<Book>("update_book", {
    bookId,
    title: updates.title,
    currentPage: updates.currentPage,
    isFavorite: updates.isFavorite,
    readingStatus: updates.readingStatus,
  });
}

/**
 * Set the collections for a book (replaces existing)
 */
export async function setBookCollections(
  bookId: number,
  collectionIds: number[]
): Promise<void> {
  return invoke<void>("set_book_collections", { bookId, collectionIds });
}

/**
 * Add a book to a collection
 */
export async function addBookToCollection(
  bookId: number,
  collectionId: number
): Promise<void> {
  return invoke<void>("add_book_to_collection", { bookId, collectionId });
}

/**
 * Remove a book from a collection
 */
export async function removeBookFromCollection(
  bookId: number,
  collectionId: number
): Promise<void> {
  return invoke<void>("remove_book_from_collection", { bookId, collectionId });
}

/**
 * Delete a book
 */
export async function deleteBook(bookId: number): Promise<void> {
  return invoke<void>("delete_book", { bookId });
}

/**
 * Import a single book from a zip/cbz/rar/cbr archive file
 * !! RAR/CBR support is desktop-only (native unrar crate doesn't compile for Android) !!
 * @param filePath - Path to the archive file
 * @param collectionId - Optional collection to add the imported book to
 * @returns The imported Book
 */
export async function importBookFromArchive(
  filePath: string,
  collectionId?: number
): Promise<Book> {
  const originalFilename = extractFilename(filePath);
  
  return invoke<Book>("import_book_from_archive", {
    filePath,
    collectionId: collectionId ?? null,
    originalFilename: originalFilename ?? null,
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

// ============================================================================
// BOOK SETTINGS COMMANDS
// ============================================================================

/**
 * Get book-specific settings
 */
export async function getBookSettings(bookId: number): Promise<BookSettings | null> {
  return invoke<BookSettings | null>("get_book_settings", { bookId });
}

/**
 * Update book-specific settings (creates if not exists)
 */
export async function updateBookSettings(
  bookId: number,
  settings: {
    readingDirection?: string | null;
    pageDisplayMode?: string | null;
    imageFitMode?: string | null;
    syncProgress?: boolean | null;
  }
): Promise<BookSettings> {
  return invoke<BookSettings>("update_book_settings", {
    bookId,
    readingDirection: settings.readingDirection !== undefined ? settings.readingDirection : null,
    pageDisplayMode: settings.pageDisplayMode !== undefined ? settings.pageDisplayMode : null,
    imageFitMode: settings.imageFitMode !== undefined ? settings.imageFitMode : null,
    syncProgress: settings.syncProgress !== undefined ? settings.syncProgress : null,
  });
}

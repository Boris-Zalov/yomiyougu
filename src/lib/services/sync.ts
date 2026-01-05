/**
 * Sync API service
 * Provides methods for syncing data with Google Drive
 */

import { invoke } from "@tauri-apps/api/core";

export interface SyncResult {
	success: boolean;
	books_uploaded: number;
	books_downloaded: number;
	bookmarks_uploaded: number;
	bookmarks_downloaded: number;
	collections_uploaded: number;
	collections_downloaded: number;
	conflicts_resolved: number;
	errors: string[];
	completed_at: number;
}

export type SyncStatus =
	| { never_synced: null }
	| { syncing: null }
	| { synced: { last_sync_at: number } }
	| { failed: { error: string; last_attempt_at: number } }
	| { disabled: null };

export async function getSyncStatus(): Promise<SyncStatus> {
	return invoke<SyncStatus>("get_sync_status");
}

/**
 * Trigger a manual sync operation
 */
export async function syncNow(): Promise<SyncResult> {
	return invoke<SyncResult>("sync_now");
}

/**
 * Download a cloud-only book from Google Drive
 * Called when user tries to read a book with cloud:// file path
 */
export async function downloadCloudBook(
	bookId: number
): Promise<import("$lib/types/library").Book> {
	return invoke<import("$lib/types/library").Book>("download_cloud_book", { bookId });
}

/**
 * Parse sync status into a human-readable string
 */
export function formatSyncStatus(status: SyncStatus): string {
	if ("never_synced" in status) {
		return "Never synced";
	}
	if ("syncing" in status) {
		return "Syncing...";
	}
	if ("synced" in status) {
		const date = new Date(status.synced.last_sync_at);
		return `Last synced: ${date.toLocaleString()}`;
	}
	if ("failed" in status) {
		return `Sync failed: ${status.failed.error}`;
	}
	if ("disabled" in status) {
		return "Sync disabled";
	}
	return "Unknown";
}

/**
 * Check if sync is currently in progress
 */
export function isSyncing(status: SyncStatus): boolean {
	return "syncing" in status;
}

/**
 * Check if sync is enabled (user is authenticated)
 */
export function isSyncEnabled(status: SyncStatus): boolean {
	return !("disabled" in status);
}

<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { platform } from "@tauri-apps/plugin-os";
	import { Heading, P, Card, Button, Badge, Modal, Spinner } from "flowbite-svelte";
	import {
		BookOpenOutline,
		ClockOutline,
		ArrowRightOutline,
		PlusOutline,
		DownloadOutline,
		ExclamationCircleOutline,
	} from "flowbite-svelte-icons";
	import { DashboardSkeleton } from "$skeletons";
	import { BookItem } from "$components/library";
	import { getBooks } from "$lib/services/library";
	import { syncApi, isRarFormat } from "$lib";
	import type { BookWithDetails } from "$lib/types/library";

	const currentPlatform = platform();
	const isAndroid = currentPlatform === "android";

	let isLoading = $state(true);
	let allBooks = $state<BookWithDetails[]>([]);

	// The most recently read book (featured)
	let featuredBook = $derived.by(() => {
		const readingBooks = allBooks.filter((b) => b.reading_status === "reading" && b.last_read_at);
		if (readingBooks.length === 0) return null;
		return readingBooks.sort(
			(a, b) => new Date(b.last_read_at!).getTime() - new Date(a.last_read_at!).getTime()
		)[0];
	});

	// Recently read books (excluding the featured one)
	let recentlyReading = $derived.by(() => {
		const readingBooks = allBooks.filter(
			(b) => b.reading_status === "reading" && b.last_read_at && b.id !== featuredBook?.id
		);
		return readingBooks
			.sort((a, b) => new Date(b.last_read_at!).getTime() - new Date(a.last_read_at!).getTime())
			.slice(0, 5);
	});

	// Books not read in a while (more than 7 days)
	let neglectedBooks = $derived.by(() => {
		const sevenDaysAgo = new Date();
		sevenDaysAgo.setDate(sevenDaysAgo.getDate() - 7);

		const neglected = allBooks.filter((b) => {
			if (b.reading_status !== "reading") return false;
			if (b.id === featuredBook?.id) return false;
			if (recentlyReading.some((r) => r.id === b.id)) return false;
			if (!b.last_read_at) return true; // Never read but marked as reading
			return new Date(b.last_read_at) < sevenDaysAgo;
		});

		return neglected
			.sort((a, b) => {
				if (!a.last_read_at) return 1;
				if (!b.last_read_at) return -1;
				return new Date(a.last_read_at).getTime() - new Date(b.last_read_at).getTime();
			})
			.slice(0, 10);
	});

	// Stats
	let totalBooks = $derived(allBooks.length);
	let readingCount = $derived(allBooks.filter((b) => b.reading_status === "reading").length);

	// Modal state for cloud download and unsupported format
	let showCloudDownloadModal = $state(false);
	let showUnsupportedFormatModal = $state(false);
	let pendingBook = $state<BookWithDetails | null>(null);
	let isDownloading = $state(false);

	function formatLastRead(dateStr: string | null): string {
		if (!dateStr) return "Not started";
		const date = new Date(dateStr);
		const now = new Date();
		const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

		if (diffDays === 0) return "Today";
		if (diffDays === 1) return "Yesterday";
		if (diffDays < 7) return `${diffDays} days ago`;
		if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
		if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
		return date.toLocaleDateString();
	}

	function openBook(book: BookWithDetails) {
		const isCloudOnly = book.file_path.startsWith("cloud://");
		const isRar = isRarFormat(book);

		// Check for unsupported format on Android
		if (isAndroid && isRar) {
			pendingBook = book;
			showUnsupportedFormatModal = true;
			return;
		}

		if (isCloudOnly) {
			pendingBook = book;
			showCloudDownloadModal = true;
			return;
		}

		goto(`/reader/${book.id}`);
	}

	async function handleDownloadConfirm() {
		if (!pendingBook) return;

		isDownloading = true;
		try {
			const updatedBook = await syncApi.downloadCloudBook(pendingBook.id);

			// Update the book in our local list
			allBooks = allBooks.map((b) =>
				b.id === updatedBook.id ? { ...b, file_path: updatedBook.file_path } : b
			);

			// Close modal and navigate to reader
			showCloudDownloadModal = false;
			const bookId = pendingBook.id;
			pendingBook = null;
			goto(`/reader/${bookId}`);
		} catch (error) {
			console.error("Failed to download book:", error);
			showCloudDownloadModal = false;
			pendingBook = null;
		} finally {
			isDownloading = false;
		}
	}

	onMount(async () => {
		try {
			allBooks = await getBooks();
		} catch (error) {
			console.error("Failed to load books:", error);
		} finally {
			isLoading = false;
		}
	});
</script>

{#if isLoading}
	<DashboardSkeleton />
{:else}
	<div class="page-container space-y-8">
		<!-- Header with stats -->
		<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
			<Heading tag="h2">Dashboard</Heading>
			<div class="flex gap-3">
				<Badge color="blue" large class="flex items-center gap-1.5">
					<BookOpenOutline class="w-4 h-4" />
					{readingCount} reading
				</Badge>
				<Badge color="gray" large class="flex items-center gap-1.5">
					{totalBooks} total
				</Badge>
			</div>
		</div>

		{#if totalBooks === 0}
			<!-- Empty state: No books at all -->
			<Card class="text-center py-12 px-6">
				<div class="flex flex-col items-center gap-4">
					<div class="p-4 rounded-full bg-gray-100 dark:bg-gray-800">
						<BookOpenOutline class="w-12 h-12 text-gray-400" />
					</div>
					<div>
						<Heading tag="h3" class="mb-2">Your library is empty</Heading>
						<P class="text-gray-500 dark:text-gray-400 max-w-md mx-auto">
							Import some manga or comics to get started. Your reading progress will be tracked
							here.
						</P>
					</div>
					<Button href="/library" class="mt-2">
						<PlusOutline class="w-4 h-4 me-2" />
						Go to Library
					</Button>
				</div>
			</Card>
		{:else if !featuredBook}
			<!-- Has books but none currently reading -->
			<Card class="text-center py-12 px-6">
				<div class="flex flex-col items-center gap-4">
					<div class="p-4 rounded-full bg-primary-100 dark:bg-primary-900/30">
						<ClockOutline class="w-12 h-12 text-primary-500" />
					</div>
					<div>
						<Heading tag="h3" class="mb-2">Start reading something!</Heading>
						<P class="text-gray-500 dark:text-gray-400 max-w-md mx-auto">
							You have {totalBooks}
							{totalBooks === 1 ? "book" : "books"} in your library. Pick one to start reading!
						</P>
					</div>
					<Button href="/library" class="mt-2">
						<BookOpenOutline class="w-4 h-4 me-2" />
						Browse Library
					</Button>
				</div>
			</Card>
		{:else}
			<!-- Featured: Continue Reading -->
			<section>
				<div class="flex items-center justify-between mb-4">
					<Heading tag="h3">Continue Reading</Heading>
				</div>

				<div class="featured-book-wrapper">
					<BookItem book={featuredBook} onclick={() => openBook(featuredBook!)} />
				</div>
			</section>
		{/if}

		{#if recentlyReading.length > 0}
			<!-- Recently Reading Row -->
			<section>
				<div class="flex items-center justify-between mb-4">
					<Heading tag="h3">Recently Reading</Heading>
					<Button
						href="/library?status=reading"
						size="sm"
						color="alternative"
						class="hidden sm:flex"
					>
						View All
						<ArrowRightOutline class="w-3.5 h-3.5 ms-1.5" />
					</Button>
				</div>

				<div class="horizontal-scroll-container">
					<div class="horizontal-scroll-row">
						{#each recentlyReading as book (book.id)}
							<div class="book-card-wrapper">
								<BookItem {book} onclick={() => openBook(book)} />
							</div>
						{/each}
					</div>
				</div>
			</section>
		{/if}

		{#if neglectedBooks.length > 0}
			<!-- Neglected Books Row -->
			<section>
				<div class="flex items-center justify-between mb-4">
					<div>
						<Heading tag="h3">Pick Up Where You Left Off</Heading>
						<P class="text-sm text-gray-500 dark:text-gray-400 mt-1">
							Books you haven't read in a while
						</P>
					</div>
				</div>

				<div class="horizontal-scroll-container">
					<div class="horizontal-scroll-row">
						{#each neglectedBooks as book (book.id)}
							<div class="book-card-wrapper">
								<BookItem {book} onclick={() => openBook(book)} />
							</div>
						{/each}
					</div>
				</div>
			</section>
		{/if}

		{#if totalBooks > 0 && readingCount === 0}
			<!-- Has books but nothing is being read -->
			<section>
				<Card class="text-center p-8">
					<P class="text-gray-500 dark:text-gray-400 mb-4">
						You have {totalBooks}
						{totalBooks === 1 ? "book" : "books"} waiting to be read!
					</P>
					<Button href="/library" color="alternative">
						<BookOpenOutline class="w-4 h-4 me-2" />
						Browse Library
					</Button>
				</Card>
			</section>
		{/if}
	</div>

	<!-- Cloud Download Modal -->
	{#if showCloudDownloadModal}
		<Modal bind:open={showCloudDownloadModal} size="md">
			<div class="text-center">
				<DownloadOutline class="mx-auto mb-4 w-12 h-12 text-blue-500 dark:text-blue-400" />
				<Heading tag="h3" class="mb-2 text-lg font-medium">Download Required</Heading>
				<P size="sm" class="mb-5 text-gray-500 dark:text-gray-400">
					"<strong>{pendingBook?.title}</strong>" is stored in the cloud. Would you like to download
					it to read?
				</P>
				<div class="flex gap-3">
					<Button
						color="alternative"
						class="flex-1"
						onclick={() => {
							showCloudDownloadModal = false;
							pendingBook = null;
						}}
						disabled={isDownloading}
					>
						Cancel
					</Button>
					<Button
						color="primary"
						class="flex-1"
						onclick={handleDownloadConfirm}
						disabled={isDownloading}
					>
						{#if isDownloading}
							<Spinner size="4" class="mr-2" />
							Downloading...
						{:else}
							Download
						{/if}
					</Button>
				</div>
			</div>
		</Modal>
	{/if}

	<!-- Unsupported Format Modal -->
	{#if showUnsupportedFormatModal}
		<Modal bind:open={showUnsupportedFormatModal} size="md">
			<div class="text-center">
				<ExclamationCircleOutline class="mx-auto mb-4 w-12 h-12 text-red-500 dark:text-red-400" />
				<Heading tag="h3" class="mb-2 text-lg font-medium">Unsupported Format</Heading>
				<P size="sm" class="mb-5 text-gray-500 dark:text-gray-400">
					RAR/CBR files are not supported on Android. Please convert "<strong
						>{pendingBook?.title}</strong
					>" to ZIP/CBZ format to read it on this device.
				</P>
				<Button
					color="alternative"
					class="w-full"
					onclick={() => {
						showUnsupportedFormatModal = false;
						pendingBook = null;
					}}
				>
					OK
				</Button>
			</div>
		</Modal>
	{/if}
{/if}

<style>
	.featured-book-wrapper {
		width: 200px;
	}

	@media (min-width: 640px) {
		.featured-book-wrapper {
			width: 240px;
		}
	}

	@media (min-width: 768px) {
		.featured-book-wrapper {
			width: 280px;
		}
	}

	.horizontal-scroll-container {
		margin-left: -1rem;
		margin-right: -1rem;
		padding-left: 1rem;
		padding-right: 1rem;
		overflow-x: auto;
		scrollbar-width: thin;
		-webkit-overflow-scrolling: touch;
	}

	.horizontal-scroll-row {
		display: flex;
		gap: 0.75rem;
		padding-bottom: 0.5rem;
	}

	.book-card-wrapper {
		flex-shrink: 0;
		width: 140px;
	}

	@media (min-width: 640px) {
		.book-card-wrapper {
			width: 160px;
		}
	}

	@media (min-width: 768px) {
		.horizontal-scroll-container {
			margin-left: 0;
			margin-right: 0;
			padding-left: 0;
			padding-right: 0;
		}

		.book-card-wrapper {
			width: 180px;
		}
	}
</style>

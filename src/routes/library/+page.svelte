<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { platform } from "@tauri-apps/plugin-os";
  import {
    Heading,
    Button,
    Search,
    Hr,
    Toast,
    Modal,
    Spinner,
    Label,
    Input,
    Textarea,
    Helper,
    Tooltip,
    P,
    Dropdown,
    DropdownItem,
  } from "flowbite-svelte";
  import {
    PlusOutline,
    CheckCircleSolid,
    CloseCircleSolid,
    RefreshOutline,
    TrashBinOutline,
    ChevronDownOutline,
    ArrowUpOutline,
    ArrowDownOutline,
    ArrowSortLettersOutline,
    DownloadOutline,
    ExclamationCircleOutline,
  } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";
  import { BookItem, CollectionItem } from "$components/library";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    libraryApi,
    syncApi,
    settingsApi,
    applyTheme,
    isRarFormat,
    type ThemeMode,
    type BookWithDetails,
    type Book,
    type CollectionWithCount,
  } from "$lib";
  import type Fuse from "fuse.js";

  const currentPlatform = platform();
  const isAndroid = currentPlatform === "android";

  let isLoading = $state(true);
  let isImporting = $state(false);
  let isSyncing = $state(false);
  let syncStatusText = $state("");
  let search = $state("");
  let books = $state<BookWithDetails[]>([]);
  let collections = $state<CollectionWithCount[]>([]);

  // Book open state
  let showCloudDownloadModal = $state(false);
  let showUnsupportedFormatModal = $state(false);
  let pendingBook = $state<BookWithDetails | null>(null);

  // Sorting state
  type SortField = "name" | "date_added" | "book_count";
  type BookSortField = "title" | "date_added" | "last_read" | "progress";
  type SortDirection = "asc" | "desc";

  let collectionSortField = $state<SortField>("name");
  let collectionSortDirection = $state<SortDirection>("asc");
  let bookSortField = $state<BookSortField>("title");
  let bookSortDirection = $state<SortDirection>("asc");

  const collectionSortOptions: { value: SortField; label: string }[] = [
    { value: "name", label: "Name" },
    { value: "date_added", label: "Date Added" },
    { value: "book_count", label: "Book Count" },
  ];

  const bookSortOptions: { value: BookSortField; label: string }[] = [
    { value: "title", label: "Title" },
    { value: "date_added", label: "Date Added" },
    { value: "last_read", label: "Last Read" },
    { value: "progress", label: "Progress" },
  ];

  function getCollectionSortLabel(): string {
    return collectionSortOptions.find((o) => o.value === collectionSortField)?.label ?? "Sort";
  }

  function getBookSortLabel(): string {
    return bookSortOptions.find((o) => o.value === bookSortField)?.label ?? "Sort";
  }

  function toggleCollectionSortDirection() {
    collectionSortDirection = collectionSortDirection === "asc" ? "desc" : "asc";
  }

  function toggleBookSortDirection() {
    bookSortDirection = bookSortDirection === "asc" ? "desc" : "asc";
  }

  function sortCollections(items: CollectionWithCount[]): CollectionWithCount[] {
    const sorted = [...items];
    const dir = collectionSortDirection === "asc" ? 1 : -1;
    
    switch (collectionSortField) {
      case "name":
        sorted.sort((a, b) => dir * a.name.localeCompare(b.name));
        break;
      case "date_added":
        sorted.sort((a, b) => dir * (new Date(a.created_at).getTime() - new Date(b.created_at).getTime()));
        break;
      case "book_count":
        sorted.sort((a, b) => dir * (a.book_count - b.book_count));
        break;
    }
    return sorted;
  }

  function sortBooks(items: BookWithDetails[]): BookWithDetails[] {
    const sorted = [...items];
    const dir = bookSortDirection === "asc" ? 1 : -1;
    
    switch (bookSortField) {
      case "title":
        sorted.sort((a, b) => dir * a.title.localeCompare(b.title));
        break;
      case "date_added":
        sorted.sort((a, b) => dir * (new Date(a.added_at).getTime() - new Date(b.added_at).getTime()));
        break;
      case "last_read":
        sorted.sort((a, b) => {
          const aTime = a.last_read_at ? new Date(a.last_read_at).getTime() : 0;
          const bTime = b.last_read_at ? new Date(b.last_read_at).getTime() : 0;
          return dir * (aTime - bTime);
        });
        break;
      case "progress":
        sorted.sort((a, b) => {
          const aProgress = a.total_pages > 0 ? a.current_page / a.total_pages : 0;
          const bProgress = b.total_pages > 0 ? b.current_page / b.total_pages : 0;
          return dir * (aProgress - bProgress);
        });
        break;
    }
    return sorted;
  }

  // Strip punctuation for search normalization
  function stripPunctuation(str: string): string {
    return str.replace(/[^\w\s]|_/g, "").replace(/\s+/g, " ");
  }

  const fuseOptions = {
    threshold: 0.3,
    ignoreLocation: true,
    includeScore: true,
    getFn: (obj: object, path: string | string[]) => {
      const key = Array.isArray(path) ? path[0] : path;
      const value = (obj as Record<string, unknown>)[key];
      if (typeof value === "string") {
        return stripPunctuation(value);
      }
      return value as string;
    },
  };

  let FuseClass: typeof Fuse | null = null;
  let fuseLoadPromise: Promise<typeof Fuse> | null = null;

  async function loadFuse(): Promise<typeof Fuse> {
    if (FuseClass) return FuseClass;
    if (!fuseLoadPromise) {
      fuseLoadPromise = import("fuse.js").then((m) => {
        FuseClass = m.default;
        return FuseClass;
      });
    }
    return fuseLoadPromise;
  }

  let collectionsFuseCache: { data: CollectionWithCount[]; fuse: Fuse<CollectionWithCount> } | null = null;
  let booksFuseCache: { data: BookWithDetails[]; fuse: Fuse<BookWithDetails> } | null = null;

  function getCollectionsFuse(): Fuse<CollectionWithCount> | null {
    if (!FuseClass) return null;
    if (!collectionsFuseCache || collectionsFuseCache.data !== collections) {
      collectionsFuseCache = {
        data: collections,
        fuse: new FuseClass(collections, { ...fuseOptions, keys: ["name", "description"] }),
      };
    }
    return collectionsFuseCache.fuse;
  }

  function getBooksFuse(): Fuse<BookWithDetails> | null {
    if (!FuseClass) return null;
    if (!booksFuseCache || booksFuseCache.data !== books) {
      booksFuseCache = {
        data: books,
        fuse: new FuseClass(books, { ...fuseOptions, keys: ["title", "filename"] }),
      };
    }
    return booksFuseCache.fuse;
  }

  let filteredCollections = $derived.by(() => {
    const query = stripPunctuation(search.trim());
    if (!query) {
      return sortCollections(collections);
    }
    const fuse = getCollectionsFuse();
    if (!fuse) return sortCollections(collections);
    return sortCollections(fuse.search(query).map((result) => result.item));
  });

  let filteredBooks = $derived.by(() => {
    const query = stripPunctuation(search.trim());
    if (!query) {
      return sortBooks(books);
    }
    const fuse = getBooksFuse();
    if (!fuse) return sortBooks(books);
    return sortBooks(fuse.search(query).map((result) => result.item));
  });

  $effect(() => {
    if (search.trim() && !FuseClass) {
      loadFuse();
    }
  });

  let showResultModal = $state(false);
  let showErrorModal = $state(false);
  let errorMessage = $state("");
  let importedBook = $state<Book | null>(null);

  let showCollectionModal = $state(false);
  let isCreatingCollection = $state(false);
  let newCollectionName = $state("");
  let newCollectionDescription = $state("");
  let collectionNameError = $state("");

  // Delete confirmation state
  let showDeleteBookModal = $state(false);
  let showDeleteCollectionModal = $state(false);
  let bookToDelete = $state<BookWithDetails | null>(null);
  let collectionToDelete = $state<CollectionWithCount | null>(null);
  let isDeleting = $state(false);

  function parseError(error: unknown): string {
    const errorStr = String(error);
    try {
      const parsed = JSON.parse(errorStr);
      if (parsed.message) {
        return parsed.message;
      }
    } catch {
      // Not JSON, return as-is
    }
    return errorStr;
  }

  function showError(message: string) {
    errorMessage = message;
    showErrorModal = true;
  }

  function openCollectionModal() {
    newCollectionName = "";
    newCollectionDescription = "";
    collectionNameError = "";
    showCollectionModal = true;
  }

  async function createCollection() {
    // Validate
    if (!newCollectionName.trim()) {
      collectionNameError = "Collection name is required";
      return;
    }

    if (
      collections.some(
        (c) => c.name.toLowerCase() === newCollectionName.trim().toLowerCase(),
      )
    ) {
      collectionNameError = "A collection with this name already exists";
      return;
    }

    isCreatingCollection = true;
    try {
      await libraryApi.createCollection(
        newCollectionName.trim(),
        newCollectionDescription.trim() || undefined,
      );
      await loadCollections();
      showCollectionModal = false;
    } catch (error) {
      console.error("Failed to create collection:", error);
      collectionNameError = parseError(error);
    } finally {
      isCreatingCollection = false;
    }
  }

  async function addBook() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Archive Files",
          extensions: ["zip", "cbz", "rar", "cbr"],
        },
      ],
    });

    if (selected) {
      isImporting = true;
      try {
        importedBook = await libraryApi.importBookFromArchive(selected);
        await loadBooks();
        showResultModal = true;
      } catch (error) {
        console.error("Failed to import book:", error);
        showError(parseError(error));
      } finally {
        isImporting = false;
      }
    }
  }

  async function loadBooks() {
    try {
      books = await libraryApi.getBooks();
    } catch (error) {
      console.error("Failed to load books:", error);
      showError(parseError(error));
    }
  }

  async function loadCollections() {
    try {
      collections = await libraryApi.getCollections();
    } catch (error) {
      console.error("Failed to load collections:", error);
    }
  }

  async function loadSyncStatus() {
    try {
      const status = await syncApi.getSyncStatus();
      syncStatusText = syncApi.formatSyncStatus(status);
    } catch (error) {
      console.error("Failed to load sync status:", error);
      syncStatusText = "";
    }
  }

  // Toggle favorite for a book
  async function handleToggleFavorite(book: BookWithDetails) {
    try {
      const updatedBook = await libraryApi.toggleFavorite(book);
      // Update the book in the local state
      books = books.map((b) =>
        b.id === updatedBook.id ? { ...b, is_favorite: updatedBook.is_favorite } : b
      );
    } catch (error) {
      console.error("Failed to toggle favorite:", error);
      showError(parseError(error));
    }
  }

  // Show delete book confirmation
  function confirmDeleteBook(book: BookWithDetails) {
    bookToDelete = book;
    showDeleteBookModal = true;
  }

  // Check if a book can be opened and navigate to reader
  function handleBookClick(book: BookWithDetails) {
    const isCloudOnly = book.file_path.startsWith("cloud://");
    const isRar = isRarFormat(book);
    
    // Check for unsupported format on Android (check filename for cloud books too)
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

  let isDownloading = $state(false);
  
  async function handleDownloadConfirm() {
    if (!pendingBook) return;
    
    isDownloading = true;
    try {
      const updatedBook = await syncApi.downloadCloudBook(pendingBook.id);
      
      // Update the book in our local list
      books = books.map(b => b.id === updatedBook.id ? { ...b, file_path: updatedBook.file_path } : b);
      
      // Close modal and navigate to reader
      showCloudDownloadModal = false;
      const bookId = pendingBook.id;
      pendingBook = null;
      goto(`/reader/${bookId}`);
    } catch (error) {
      console.error("Failed to download book:", error);
      showError(parseError(error));
      showCloudDownloadModal = false;
      pendingBook = null;
    } finally {
      isDownloading = false;
    }
  }

  // Delete a book
  async function handleDeleteBook() {
    if (!bookToDelete) return;
    
    isDeleting = true;
    try {
      await libraryApi.deleteBook(bookToDelete.id);
      books = books.filter((b) => b.id !== bookToDelete!.id);
      showDeleteBookModal = false;
      bookToDelete = null;
    } catch (error) {
      console.error("Failed to delete book:", error);
      showError(parseError(error));
    } finally {
      isDeleting = false;
    }
  }

  // Show delete collection confirmation
  function confirmDeleteCollection(collection: CollectionWithCount) {
    collectionToDelete = collection;
    showDeleteCollectionModal = true;
  }

  // Delete a collection
  async function handleDeleteCollection() {
    if (!collectionToDelete) return;
    
    isDeleting = true;
    try {
      await libraryApi.deleteCollection(collectionToDelete.id);
      collections = collections.filter((c) => c.id !== collectionToDelete!.id);
      showDeleteCollectionModal = false;
      collectionToDelete = null;
    } catch (error) {
      console.error("Failed to delete collection:", error);
      showError(parseError(error));
    } finally {
      isDeleting = false;
    }
  }

  async function handleSync() {
    isSyncing = true;
    syncStatusText = "Syncing...";
    try {
      const result = await syncApi.syncNow();
      if (result.success) {
        syncStatusText = `Synced: ${result.books_uploaded}↑ ${result.books_downloaded}↓`;
        // Reload books in case any were synced
        await loadBooks();
        // Reload settings and reapply theme in case it changed
        try {
          const settings = await settingsApi.getSettings();
          const theme = (settings.categories
            .find((c) => c.id === "appearance")
            ?.settings.find((s) => s.key === "appearance.theme")
            ?.value || "system") as ThemeMode;
          applyTheme(theme);
        } catch (e) {
          console.error("Failed to reload theme after sync:", e);
        }
      } else {
        syncStatusText = `Sync had errors: ${result.errors.join(", ")}`;
        showError(result.errors.join("\n"));
      }
    } catch (error) {
      console.error("Sync failed:", error);
      syncStatusText = "Sync failed";
      showError(parseError(error));
    } finally {
      isSyncing = false;
      // Reload sync status after a delay
      setTimeout(loadSyncStatus, 2000);
    }
  }

  onMount(async () => {
    await Promise.all([loadBooks(), loadCollections(), loadSyncStatus()]);
    isLoading = false;
  });
</script>

{#if isLoading}
  <LibrarySkeleton />
{:else}
  <div class="page-container p-4">
    <!-- Search and Sync Row -->
    <div class="mb-6 flex items-center gap-3">
      <Search
        clearable
        clearableOnClick={() => { search = ""; }}
        class="flex-1"
        bind:value={search}
        placeholder="Search books and collections..."
      ></Search>
      
      <Button
        id="sync-btn"
        onclick={handleSync}
        disabled={isSyncing}
        color="alternative"
        class="shrink-0"
      >
        {#if isSyncing}
          <Spinner size="4" class="mr-2" />
        {:else}
          <RefreshOutline class="w-4 h-4 mr-2" />
        {/if}
        Sync
      </Button>
      <Tooltip triggeredBy="#sync-btn" placement="bottom">
        {syncStatusText || "Sync with Google Drive"}
      </Tooltip>
    </div>

    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h5">Collections</Heading>
      <div class="flex items-center gap-1">
        <Button color="alternative" size="sm" class="collection-sort-btn">
          <ArrowSortLettersOutline class="me-1.5 h-3.5 w-3.5" />
          {getCollectionSortLabel()}
          <ChevronDownOutline class="ms-1.5 h-3 w-3" />
        </Button>
        <Dropdown simple triggeredBy=".collection-sort-btn" placement="bottom-end">
          {#each collectionSortOptions as option (option.value)}
            <DropdownItem
              onclick={() => (collectionSortField = option.value)}
              class={collectionSortField === option.value ? "bg-gray-100 dark:bg-gray-600" : ""}
            >
              {option.label}
            </DropdownItem>
          {/each}
        </Dropdown>
        <Button
          color="alternative"
          size="sm"
          class="px-2!"
          onclick={toggleCollectionSortDirection}
          aria-label="Toggle sort direction"
        >
          {#if collectionSortDirection === "asc"}
            <ArrowUpOutline class="h-4 w-4" />
          {:else}
            <ArrowDownOutline class="h-4 w-4" />
          {/if}
        </Button>
      </div>
    </div>

    <div
      class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3"
    >
      {#if !search.trim()}
        <Button
          onclick={openCollectionModal}
          class="
            flex items-center justify-center w-full h-auto aspect-2/3 
            border-2 rounded-lg transition-colors
            bg-red-50 border-red-200 text-red-500 hover:bg-red-100
            dark:bg-gray-900 dark:border-red-900 dark:text-red-400 dark:hover:bg-gray-700
          "
        >
          <PlusOutline class="w-6 h-6" />
        </Button>
      {/if}

      {#each filteredCollections as collection (collection.id)}
        <CollectionItem 
          {collection} 
          ondelete={confirmDeleteCollection}
        />
      {:else}
        {#if search.trim()}
          <p
            class="col-span-full text-center text-gray-500 dark:text-gray-400 py-4"
          >
            No collections match "{search}"
          </p>
        {/if}
      {/each}
    </div>

    <Hr class="my-8" />

    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h5">All volumes</Heading>
      <div class="flex items-center gap-1">
        <Button color="alternative" size="sm" class="book-sort-btn">
          <ArrowSortLettersOutline class="me-1.5 h-3.5 w-3.5" />
          {getBookSortLabel()}
          <ChevronDownOutline class="ms-1.5 h-3 w-3" />
        </Button>
        <Dropdown simple triggeredBy=".book-sort-btn" placement="bottom-end">
          {#each bookSortOptions as option (option.value)}
            <DropdownItem
              onclick={() => (bookSortField = option.value)}
              class={bookSortField === option.value ? "bg-gray-100 dark:bg-gray-600" : ""}
            >
              {option.label}
            </DropdownItem>
          {/each}
        </Dropdown>
        <Button
          color="alternative"
          size="sm"
          class="px-2!"
          onclick={toggleBookSortDirection}
          aria-label="Toggle sort direction"
        >
          {#if bookSortDirection === "asc"}
            <ArrowUpOutline class="h-4 w-4" />
          {:else}
            <ArrowDownOutline class="h-4 w-4" />
          {/if}
        </Button>
      </div>
    </div>

    <div
      class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3"
    >
      {#if !search.trim()}
        <Button
          onclick={addBook}
          disabled={isImporting}
          class="
            flex items-center justify-center w-full h-auto aspect-2/3 
            border-2 rounded-lg transition-colors
            bg-red-50 border-red-200 text-red-500 hover:bg-red-100
            dark:bg-gray-900 dark:border-red-900 dark:text-red-400 dark:hover:bg-gray-700
            disabled:opacity-50 disabled:cursor-not-allowed
          "
        >
          {#if isImporting}
            <Spinner color="primary" size="6" />
          {:else}
            <PlusOutline class="w-6 h-6" />
          {/if}
        </Button>
      {/if}

      {#each filteredBooks as book (book.id)}
        <BookItem 
          {book} 
          onclick={() => handleBookClick(book)}
          ontogglefavorite={handleToggleFavorite}
          ondelete={confirmDeleteBook}
        />
      {:else}
        {#if search.trim()}
          <p
            class="col-span-full text-center text-gray-500 dark:text-gray-400 py-4"
          >
            No books match "{search}"
          </p>
        {/if}
      {/each}
    </div>
  </div>

  {#if isImporting}
    <Toast
      position="top-right"
      dismissable={false}
      class="mt-10 mr-4 fixed z-50 bg-white dark:bg-gray-800 shadow-lg border border-gray-200 dark:border-gray-700"
    >
      <div class="flex items-center gap-3">
        <Spinner size="5" color="primary" />
        <div class="text-sm font-normal">Importing book...</div>
      </div>
    </Toast>
  {/if}

  <!-- Create Collection Modal -->
  {#if showCollectionModal}
  <Modal bind:open={showCollectionModal} size="md" class="w-full">
    <form
      onsubmit={(e) => {
        e.preventDefault();
        createCollection();
      }}
      class="space-y-4"
    >
      <Heading tag="h3" class="text-lg font-medium">
        Create New Collection
      </Heading>

      <div>
        <Label for="collection-name" class="mb-2">Name</Label>
        <Input
          id="collection-name"
          bind:value={newCollectionName}
          placeholder="Enter collection name"
          color={collectionNameError ? "red" : undefined}
          disabled={isCreatingCollection}
        />
        {#if collectionNameError}
          <Helper class="mt-1" color="red">{collectionNameError}</Helper>
        {/if}
      </div>

      <div>
        <Label for="collection-description" class="mb-2"
          >Description (optional)</Label
        >
        <Textarea
          id="collection-description"
          bind:value={newCollectionDescription}
          placeholder="Enter a description for this collection"
          rows={3}
          disabled={isCreatingCollection}
          class="resize-none w-full"
        />
      </div>

      <div class="flex gap-3 pt-2">
        <Button
          type="button"
          color="alternative"
          class="flex-1"
          onclick={() => (showCollectionModal = false)}
          disabled={isCreatingCollection}
        >
          Cancel
        </Button>
        <Button
          type="submit"
          color="primary"
          class="flex-1"
          disabled={isCreatingCollection}
        >
          {#if isCreatingCollection}
            <Spinner size="4" class="mr-2" />
            Creating...
          {:else}
            Create
          {/if}
        </Button>
      </div>
    </form>
  </Modal>
  {/if}

  <!-- Import Success Modal -->
  {#if showResultModal}
  <Modal bind:open={showResultModal} size="md" autoclose>
    <div class="text-center">
      <CheckCircleSolid
        class="mx-auto mb-4 w-12 h-12 text-green-500 dark:text-green-400"
      />
      <Heading tag="h3" class="mb-5 text-lg font-normal">
        Import Complete
      </Heading>
      <div class="mb-6 text-sm text-gray-600 dark:text-gray-300">
        {#if importedBook}
          <P>Successfully added "<strong>{importedBook.title}</strong>"</P>
          <P class="mt-2 text-gray-500">{importedBook.total_pages} pages</P>
        {/if}
      </div>
      <Button color="red" class="w-full">Close</Button>
    </div>
  </Modal>
  {/if}

  <!-- Error Modal -->
  {#if showErrorModal}
  <Modal bind:open={showErrorModal} size="md" autoclose>
    <div class="text-center">
      <CloseCircleSolid
        class="mx-auto mb-4 w-12 h-12 text-red-500 dark:text-red-400"
      />
      <Heading tag="h3" class="mb-5 text-lg font-normal">
        Error
      </Heading>
      <P size="sm" class="mb-6 text-gray-600 dark:text-gray-300">
        {errorMessage}
      </P>
      <Button color="red" class="w-full">Close</Button>
    </div>
  </Modal>
  {/if}

  <!-- Delete Book Confirmation Modal -->
  {#if showDeleteBookModal}
  <Modal bind:open={showDeleteBookModal} size="md">
    <div class="text-center">
      <TrashBinOutline
        class="mx-auto mb-4 w-12 h-12 text-red-500 dark:text-red-400"
      />
      <Heading tag="h3" class="mb-2 text-lg font-medium">
        Delete Book
      </Heading>
      <P size="sm" class="mb-5 text-gray-500 dark:text-gray-400">
        Are you sure you want to delete "<strong>{bookToDelete?.title}</strong>"? This action cannot be undone.
      </P>
      <div class="flex gap-3">
        <Button
          color="alternative"
          class="flex-1"
          onclick={() => { showDeleteBookModal = false; bookToDelete = null; }}
          disabled={isDeleting}
        >
          Cancel
        </Button>
        <Button
          color="red"
          class="flex-1"
          onclick={handleDeleteBook}
          disabled={isDeleting}
        >
          {#if isDeleting}
            <Spinner size="4" class="mr-2" />
            Deleting...
          {:else}
            Delete
          {/if}
        </Button>
      </div>
    </div>
  </Modal>
  {/if}

  <!-- Delete Collection Confirmation Modal -->
  {#if showDeleteCollectionModal}
  <Modal bind:open={showDeleteCollectionModal} size="md">
    <div class="text-center">
      <TrashBinOutline
        class="mx-auto mb-4 w-12 h-12 text-red-500 dark:text-red-400"
      />
      <Heading tag="h3" class="mb-2 text-lg font-medium">
        Delete Collection
      </Heading>
      <P size="sm" class="mb-5 text-gray-500 dark:text-gray-400">
        Are you sure you want to delete the collection "<strong>{collectionToDelete?.name}</strong>"? Books in this collection will not be deleted.
      </P>
      <div class="flex gap-3">
        <Button
          color="alternative"
          class="flex-1"
          onclick={() => { showDeleteCollectionModal = false; collectionToDelete = null; }}
          disabled={isDeleting}
        >
          Cancel
        </Button>
        <Button
          color="red"
          class="flex-1"
          onclick={handleDeleteCollection}
          disabled={isDeleting}
        >
          {#if isDeleting}
            <Spinner size="4" class="mr-2" />
            Deleting...
          {:else}
            Delete
          {/if}
        </Button>
      </div>
    </div>
  </Modal>
  {/if}

  <!-- Cloud Download Modal -->
  {#if showCloudDownloadModal}
  <Modal bind:open={showCloudDownloadModal} size="md">
    <div class="text-center">
      <DownloadOutline
        class="mx-auto mb-4 w-12 h-12 text-blue-500 dark:text-blue-400"
      />
      <Heading tag="h3" class="mb-2 text-lg font-medium">
        Download Required
      </Heading>
      <P size="sm" class="mb-5 text-gray-500 dark:text-gray-400">
        "<strong>{pendingBook?.title}</strong>" is stored in the cloud. Would you like to download it to read?
      </P>
      <div class="flex gap-3">
        <Button
          color="alternative"
          class="flex-1"
          onclick={() => { showCloudDownloadModal = false; pendingBook = null; }}
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
      <ExclamationCircleOutline
        class="mx-auto mb-4 w-12 h-12 text-red-500 dark:text-red-400"
      />
      <Heading tag="h3" class="mb-2 text-lg font-medium">
        Unsupported Format
      </Heading>
      <P size="sm" class="mb-5 text-gray-500 dark:text-gray-400">
        RAR/CBR files are not supported on Android. Please convert "<strong>{pendingBook?.title}</strong>" to ZIP/CBZ format to read it on this device.
      </P>
      <Button
        color="alternative"
        class="w-full"
        onclick={() => { showUnsupportedFormatModal = false; pendingBook = null; }}
      >
        OK
      </Button>
    </div>
  </Modal>
  {/if}
{/if}

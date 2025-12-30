<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { platform } from "@tauri-apps/plugin-os";
  import {
    Heading,
    Button,
    Search,
    Hr,
    Modal,
    Spinner,
    Toast,
    P,
  } from "flowbite-svelte";
  import {
    ArrowLeftOutline,
    CloseCircleSolid,
    PlusOutline,
    CheckCircleSolid,
    BookSolid,
    UploadOutline,
    TrashBinOutline,
    DownloadOutline,
    ExclamationCircleOutline,
  } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";
  import { BookItem } from "$components/library";
  import { libraryApi, syncApi, isRarFormat, type BookWithDetails, type Collection, type Book } from "$lib";
  import Fuse from "fuse.js";
  import { open } from "@tauri-apps/plugin-dialog";

  let collectionId = $derived(page.params.id);

  const currentPlatform = platform();
  const isAndroid = currentPlatform === "android";

  let isLoading = $state(true);
  let search = $state("");
  let isImporting = $state(false);
  
  let collection = $state<Collection | null>(null);
  let books = $state<BookWithDetails[]>([]);
  let error = $state("");

  let showAddModal = $state(false);
  let showResultModal = $state(false);
  let showErrorModal = $state(false);
  let errorMessage = $state("");
  let importedBook = $state<Book | null>(null);

  let showDeleteBookModal = $state(false);
  let bookToDelete = $state<BookWithDetails | null>(null);
  let isDeleting = $state(false);

  let showRemoveFromCollectionModal = $state(false);
  let bookToRemove = $state<BookWithDetails | null>(null);
  let isRemoving = $state(false);

  let showCloudDownloadModal = $state(false);
  let showUnsupportedFormatModal = $state(false);
  let pendingBook = $state<BookWithDetails | null>(null);

  function stripPunctuation(str: string): string {
    return str.replace(/[^\w\s]|_/g, "").replace(/\s+/g, " ");
  }

  const fuseOptions = {
    threshold: 0.4,
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

  let booksFuse = $derived(
    new Fuse(books, {
      ...fuseOptions,
      keys: ["title", "filename"],
    }),
  );

  let filteredBooks = $derived.by(() => {
    const query = stripPunctuation(search.trim());
    if (!query) {
      return [...books].sort((a, b) => a.title.localeCompare(b.title));
    }
    return booksFuse
      .search(query)
      .map((result) => result.item)
      .sort((a, b) => a.title.localeCompare(b.title));
  });

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

  async function loadCollectionDetails() {
    try {
      const [collectionData, booksData] = await Promise.all([
        libraryApi.getCollection(Number(collectionId)),
        libraryApi.getBooks({ collectionId: Number(collectionId) })
      ]);
      
      collection = collectionData;
      books = booksData;
      
    } catch (err) {
      console.error("Failed to load collection:", err);
      error = "Failed to load collection details.";
    }
  }

  async function importFromDevice() {
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
      showAddModal = false;
      isImporting = true;
      try {
        importedBook = await libraryApi.importBookFromArchive(selected, Number(collectionId));
        await loadCollectionDetails();
        showResultModal = true;
      } catch (error) {
        console.error("Failed to import book:", error);
        showError(parseError(error));
      } finally {
        isImporting = false;
      }
    }
  }

  onMount(async () => {
    if (collectionId) {
      await loadCollectionDetails();
    }
    isLoading = false;
  });

  // Toggle favorite for a book
  async function handleToggleFavorite(book: BookWithDetails) {
    try {
      const updatedBook = await libraryApi.toggleFavorite(book);
      // Update the book in the local state
      books = books.map((b) =>
        b.id === updatedBook.id ? { ...b, is_favorite: updatedBook.is_favorite } : b
      );
    } catch (err) {
      console.error("Failed to toggle favorite:", err);
      showError(parseError(err));
    }
  }

  function confirmDeleteBook(book: BookWithDetails) {
    bookToDelete = book;
    showDeleteBookModal = true;
  }

  function handleBookClick(book: BookWithDetails) {
    const isCloudOnly = book.file_path.startsWith("cloud://");
    const isRar = isRarFormat(book);
    
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
    
    // Navigate to reader
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
    } catch (err) {
      console.error("Failed to download book:", err);
      showError(parseError(err));
      showCloudDownloadModal = false;
      pendingBook = null;
    } finally {
      isDownloading = false;
    }
  }

  async function handleDeleteBook() {
    if (!bookToDelete) return;
    isDeleting = true;
    try {
      await libraryApi.deleteBook(bookToDelete.id);
      books = books.filter((b) => b.id !== bookToDelete!.id);
      showDeleteBookModal = false;
      bookToDelete = null;
    } catch (err) {
      console.error("Failed to delete book:", err);
      showError(parseError(err));
    } finally {
      isDeleting = false;
    }
  }

  function confirmRemoveFromCollection(book: BookWithDetails) {
    bookToRemove = book;
    showRemoveFromCollectionModal = true;
  }

  async function handleRemoveFromCollection() {
    if (!bookToRemove || !collection) return;
    isRemoving = true;
    try {
      await libraryApi.removeBookFromCollection(bookToRemove.id, collection.id);
      books = books.filter((b) => b.id !== bookToRemove!.id);
      showRemoveFromCollectionModal = false;
      bookToRemove = null;
    } catch (err) {
      console.error("Failed to remove book from collection:", err);
      showError(parseError(err));
    } finally {
      isRemoving = false;
    }
  }
</script>
{#if isLoading}
  <LibrarySkeleton />
{:else if error}
  <div class="h-full flex flex-col items-center justify-center p-10 text-center">
    <CloseCircleSolid class="w-12 h-12 text-red-500 mb-4" />
    <h3 class="text-xl font-medium text-gray-900 dark:text-white">Error</h3>
    <p class="text-gray-500 mt-2">{error}</p>
    <Button href="/library" class="mt-4" color="light">Return to Library</Button>
  </div>
{:else if collection}
  <div class="page-container p-4">
    
    <div class="mb-6">
      <div class="flex items-center gap-4 mb-4">
        <Button 
          href="/library" 
          color="alternative" 
          class="p-2.5!" 
          aria-label="Back to library"
        >
          <ArrowLeftOutline class="w-5 h-5" />
        </Button>
        
        <div>
          <Heading tag="h4">{collection.name}</Heading>
          {#if collection.description}
            <p class="text-gray-500 dark:text-gray-400 mt-2 max-w-3xl">
              {collection.description}
            </p>
          {/if}
        </div>
      </div>
      
      <Search
        clearable
        bind:value={search}
        placeholder={`Search in ${collection.name}...`}
      />
    </div>

    <Hr class="my-6" />

    <div class="flex items-center justify-between mb-4">
      <span class="text-sm text-gray-500 dark:text-gray-400">
        {filteredBooks.length} {filteredBooks.length === 1 ? 'volume' : 'volumes'}
      </span>
    </div>

    <div
      class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3"
    >
      {#if !search.trim()}
        <Button
          onclick={() => showAddModal = true}
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
          onremovefromcollection={confirmRemoveFromCollection}
          collectionName={collection.name}
        />
      {:else}
        {#if search.trim()}
          <div class="col-span-full py-12 text-center">
            <p class="text-gray-500 dark:text-gray-400 text-lg">
              No books found matching "{search}"
            </p>
          </div>
        {/if}
      {/each}
    </div>
  </div>
{:else}
  <div class="p-8 text-center">
    <p class="text-gray-500">Collection not found.</p>
    <Button href="/library" class="mt-4">Go Back</Button>
  </div>
{/if}

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

<!-- Add Book Options Modal -->
<Modal bind:open={showAddModal} size="sm" class="w-full">
  <div class="space-y-4">
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Add Book to Collection
    </h3>

    <Button
      color="light"
      class="gap-3 p-4 w-full"
    >
    <a href={`/library/collections/${collectionId}/add-existing`}
        class="w-full flex items-center justify-start"
    >
      <BookSolid class="w-5 h-5" />
      <div class="text-left">
        <div class="font-medium">From Existing Books</div>
        <div class="text-xs text-gray-500 dark:text-gray-400">
          Select books already in your library
        </div>
      </div>
    </a>
    </Button>

    <Button
      onclick={importFromDevice}
      color="light"
      class="w-full flex items-center justify-start gap-3 p-4"
    >
      <UploadOutline class="w-5 h-5" />
      <div class="text-left">
        <div class="font-medium">Import from Device</div>
        <div class="text-xs text-gray-500 dark:text-gray-400">
          Import a new archive file
        </div>
      </div>
    </Button>
  </div>
</Modal>

<!-- Success Modal -->
<Modal bind:open={showResultModal} size="md" class="w-full">
  <div class="text-center">
    <CheckCircleSolid class="mx-auto mb-4 h-12 w-12 text-green-500" />
    <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
      Book imported successfully!
    </h3>
    {#if importedBook}
      <p class="mb-5 text-sm text-gray-600 dark:text-gray-300">
        <strong>{importedBook.title}</strong> has been added to {collection?.name}.
      </p>
    {/if}
    <Button color="green" onclick={() => (showResultModal = false)}>
      Continue
    </Button>
  </div>
</Modal>

<!-- Error Modal -->
<Modal bind:open={showErrorModal} size="md" class="w-full">
  <div class="text-center">
    <CloseCircleSolid class="mx-auto mb-4 h-12 w-12 text-red-500" />
    <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
      Import Failed
    </h3>
    <p class="mb-5 text-sm text-gray-600 dark:text-gray-300">
      {errorMessage}
    </p>
    <Button color="red" onclick={() => (showErrorModal = false)}>
      Close
    </Button>
  </div>
</Modal>

<!-- Delete Book Confirmation Modal -->
<Modal bind:open={showDeleteBookModal} size="md" autoclose={false}>
  <div class="text-center">
    <TrashBinOutline class="mx-auto mb-4 w-12 h-12 text-red-500 dark:text-red-400" />
    <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
      Delete Book
    </h3>
    {#if bookToDelete}
      <P class="mb-6 text-sm text-gray-600 dark:text-gray-300">
        Are you sure you want to delete <strong>{bookToDelete.title}</strong>? This action cannot be undone.
      </P>
    {/if}
    <div class="flex justify-center gap-4">
      <Button
        color="red"
        onclick={handleDeleteBook}
        disabled={isDeleting}
      >
        {#if isDeleting}
          <Spinner size="4" class="mr-2" />
        {/if}
        Delete
      </Button>
      <Button
        color="alternative"
        onclick={() => {
          showDeleteBookModal = false;
          bookToDelete = null;
        }}
        disabled={isDeleting}
      >
        Cancel
      </Button>
    </div>
  </div>
</Modal>

<!-- Remove from Collection Confirmation Modal -->
<Modal bind:open={showRemoveFromCollectionModal} size="md" autoclose={false}>
  <div class="text-center">
    <CloseCircleSolid class="mx-auto mb-4 w-12 h-12 text-orange-500 dark:text-orange-400" />
    <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
      Remove from Collection
    </h3>
    {#if bookToRemove && collection}
      <P class="mb-6 text-sm text-gray-600 dark:text-gray-300">
        Remove <strong>{bookToRemove.title}</strong> from <strong>{collection.name}</strong>? The book will remain in your library.
      </P>
    {/if}
    <div class="flex justify-center gap-4">
      <Button
        color="yellow"
        onclick={handleRemoveFromCollection}
        disabled={isRemoving}
      >
        {#if isRemoving}
          <Spinner size="4" class="mr-2" />
        {/if}
        Remove
      </Button>
      <Button
        color="alternative"
        onclick={() => {
          showRemoveFromCollectionModal = false;
          bookToRemove = null;
        }}
        disabled={isRemoving}
      >
        Cancel
      </Button>
    </div>
  </div>
</Modal>

<!-- Cloud Download Modal -->
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

<!-- Unsupported Format Modal -->
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
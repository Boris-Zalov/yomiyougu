<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import {
    Heading,
    Button,
    Search,
    Hr,
    Spinner,
    Toast,
  } from "flowbite-svelte";
  import {
    ArrowLeftOutline,
    FileCheckSolid,
    CheckCircleSolid,
    CloseCircleSolid,
  } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";
  import { libraryApi, type BookWithDetails, type Collection } from "$lib";
  import { getCoverPath } from "$lib/types/library";
  import Fuse from "fuse.js";

  let collectionId = $derived(page.params.id);
  let collectionIdNum = $derived(Number(collectionId));

  let isLoading = $state(true);
  let isAdding = $state(false);
  let search = $state("");
  
  let collection = $state<Collection | null>(null);
  let allBooks = $state<BookWithDetails[]>([]);
  let selectedBookIds = $state<Set<number>>(new Set());

  let showSuccessToast = $state(false);
  let showErrorToast = $state(false);
  let errorMessage = $state("");
  let addedCount = $state(0);
  let navigationTimeoutId = $state<number | null>(null);

  // Books not already in this collection
  let availableBooks = $derived(
    allBooks.filter(book => !book.collection_ids.includes(collectionIdNum))
  );

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
    new Fuse(availableBooks, {
      ...fuseOptions,
      keys: ["title", "filename"],
    }),
  );

  let filteredBooks = $derived.by(() => {
    const query = stripPunctuation(search.trim());
    if (!query) {
      return [...availableBooks].sort((a, b) => a.title.localeCompare(b.title));
    }
    return booksFuse
      .search(query)
      .map((result) => result.item)
      .sort((a, b) => a.title.localeCompare(b.title));
  });

  function toggleBookSelection(bookId: number) {
    const newSet = new Set(selectedBookIds);
    if (newSet.has(bookId)) {
      newSet.delete(bookId);
    } else {
      newSet.add(bookId);
    }
    selectedBookIds = newSet;
  }

  function isSelected(bookId: number): boolean {
    return selectedBookIds.has(bookId);
  }

  async function addSelectedBooks() {
    if (selectedBookIds.size === 0) return;
    
    isAdding = true;
    try {
      const bookIds = Array.from(selectedBookIds);
      for (const bookId of bookIds) {
        await libraryApi.addBookToCollection(bookId, collectionIdNum);
      }
      
      addedCount = selectedBookIds.size;
      showSuccessToast = true;
      
      // Auto-dismiss toast and navigate after 3 seconds
      navigationTimeoutId = setTimeout(() => {
        navigateToCollection();
      }, 3000) as unknown as number;
    } catch (error) {
      console.error("Failed to add books:", error);
      errorMessage = String(error);
      showErrorToast = true;
    } finally {
      isAdding = false;
    }
  }
  
  function navigateToCollection() {
    // Clear any pending auto-navigation timeout
    if (navigationTimeoutId !== null) {
      clearTimeout(navigationTimeoutId);
      navigationTimeoutId = null;
    }
    showSuccessToast = false;
    goto(`/library/collections/${collectionId}`);
  }

  async function loadData() {
    try {
      const [collectionData, booksData] = await Promise.all([
        libraryApi.getCollection(collectionIdNum),
        libraryApi.getBooks()
      ]);
      
      collection = collectionData;
      allBooks = booksData;
    } catch (err) {
      console.error("Failed to load data:", err);
    }
  }

  onMount(async () => {
    await loadData();
    isLoading = false;
  });
</script>

{#if isLoading}
  <LibrarySkeleton />
{:else}
  <div class="page-container p-4">
    <div class="mb-6">
      <div class="flex items-center gap-4 mb-4">
        <Button 
          href={`/library/collections/${collectionId}`}
          color="alternative" 
          class="p-2.5!" 
          aria-label="Back to collection"
        >
          <ArrowLeftOutline class="w-5 h-5" />
        </Button>
        
        <div class="flex-1">
          <Heading tag="h4">Add Existing Books</Heading>
          <p class="text-gray-500 dark:text-gray-400 mt-1">
            Select books to add to <strong>{collection?.name}</strong>
          </p>
        </div>

        {#if selectedBookIds.size > 0}
          <Button
            onclick={addSelectedBooks}
            disabled={isAdding}
            color="primary"
          >
            {#if isAdding}
              <Spinner size="4" class="mr-2" />
            {/if}
            Add {selectedBookIds.size} {selectedBookIds.size === 1 ? 'book' : 'books'}
          </Button>
        {/if}
      </div>

      <Search
        clearable
        bind:value={search}
        placeholder="Search available books..."
      />
    </div>

    <Hr class="my-6" />

    <div class="flex items-center justify-between mb-4">
      <span class="text-sm text-gray-500 dark:text-gray-400">
        {availableBooks.length} {availableBooks.length === 1 ? 'book' : 'books'} available
        {#if selectedBookIds.size > 0}
          · {selectedBookIds.size} selected
        {/if}
      </span>
    </div>

    <div
      class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3"
    >
      {#each filteredBooks as book (book.id)}
        <button
          type="button"
          onclick={() => toggleBookSelection(book.id)}
          class="relative group w-full h-auto aspect-2/3 overflow-hidden rounded-lg shadow-sm hover:shadow-md border-2 transition-all
            {isSelected(book.id) 
              ? 'border-primary-500 dark:border-primary-600 border-3' 
              : 'border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-700'
            } bg-gray-200 dark:bg-gray-700"
        >
          {#if isSelected(book.id)}
            <div class="absolute top-2 left-2 z-20 p-1.5 rounded-full bg-primary-500 text-white shadow-lg">
              <FileCheckSolid class="w-4 h-4" />
            </div>
          {/if}
          
          <img 
            src={getCoverPath(book.id)} 
            alt={book.title}
            class="absolute inset-0 w-full h-full object-cover transition-transform duration-300 group-hover:scale-105
              {isSelected(book.id) ? 'opacity-80' : ''}"
            loading="lazy"
          />
          
          <div class="absolute bottom-0 left-0 right-0 z-10 p-2 bg-white/90 dark:bg-gray-800/90 backdrop-blur-md border-t border-gray-100 dark:border-gray-700">
            <h5 class="text-xs font-bold text-gray-900 dark:text-white line-clamp-2 leading-tight text-left" title={book.title}>
              {book.title}
            </h5>
          </div>
        </button>
      {:else}
        {#if search.trim()}
          <div class="col-span-full py-12 text-center">
            <p class="text-gray-500 dark:text-gray-400 text-lg">
              No books found matching "{search}"
            </p>
          </div>
        {:else}
          <div class="col-span-full py-12 text-center bg-gray-50 dark:bg-gray-800 rounded-lg border border-dashed border-gray-300 dark:border-gray-700">
            <p class="text-gray-500 dark:text-gray-400">
              All books are already in this collection.
            </p>
            <Button href={`/library/collections/${collectionId}`} class="mt-4">
              Back to Collection
            </Button>
          </div>
        {/if}
      {/each}
    </div>
  </div>

  {#if showSuccessToast}
    <Toast
      position="top-right"
      dismissable={false}
      class="mt-10 mr-4 fixed z-50 bg-white dark:bg-gray-800 shadow-lg border border-gray-200 dark:border-gray-700"
    >
      <div class="flex items-center gap-3">
        <CheckCircleSolid class="w-5 h-5 text-green-500" />
        <div class="flex-1">
          <div class="text-sm font-normal">
            Added {addedCount} {addedCount === 1 ? 'book' : 'books'} to collection!
          </div>
        </div>
        <Button
          size="xs"
          color="green"
          onclick={navigateToCollection}
          class="ml-2"
        >
          View Collection
        </Button>
      </div>
    </Toast>
  {/if}

  {#if showErrorToast}
    <Toast
      position="top-right"
      color="red"
      class="mt-10 mr-4 fixed z-50 bg-white dark:bg-gray-800 shadow-lg border border-red-200 dark:border-red-700"
    >
      <div class="flex items-center gap-3">
        <CloseCircleSolid class="w-5 h-5 text-red-500" />
        <div class="text-sm font-normal">Failed to add books</div>
      </div>
    </Toast>
  {/if}
{/if}

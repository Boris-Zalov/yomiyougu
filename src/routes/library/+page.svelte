<script lang="ts">
  import { onMount } from "svelte";
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
  } from "flowbite-svelte";
  import {
    PlusOutline,
    CheckCircleSolid,
    CloseCircleSolid,
  } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";
  import { BookItem } from "$components/library";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    libraryApi,
    type BookWithDetails,
    type Book,
    type CollectionWithCount,
    stripPunctuation,
    fuseOptions,
  } from "$lib";
  import Fuse from "fuse.js";

  let isLoading = $state(true);
  let isImporting = $state(false);
  let search = $state("");
  let books = $state<BookWithDetails[]>([]);
  let collections = $state<CollectionWithCount[]>([]);

  let collectionsFuse = $derived(
    new Fuse(collections, {
      ...fuseOptions,
      keys: ["name", "description"],
    }),
  );

  let booksFuse = $derived(
    new Fuse(books, {
      ...fuseOptions,
      keys: ["title", "filename"],
    }),
  );

  let filteredCollections = $derived.by(() => {
    const query = stripPunctuation(search.trim());
    if (!query) {
      return [...collections].sort((a, b) => a.name.localeCompare(b.name));
    }
    return collectionsFuse
      .search(query)
      .map((result) => result.item)
      .sort((a, b) => a.name.localeCompare(b.name));
  });

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

  let showResultModal = $state(false);
  let showErrorModal = $state(false);
  let errorMessage = $state("");
  let importedBook = $state<Book | null>(null);

  let showCollectionModal = $state(false);
  let isCreatingCollection = $state(false);
  let newCollectionName = $state("");
  let newCollectionDescription = $state("");
  let collectionNameError = $state("");

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

  onMount(async () => {
    await Promise.all([loadBooks(), loadCollections()]);
    isLoading = false;
  });
</script>

{#if isLoading}
  <LibrarySkeleton />
{:else}
  <div class="page-container p-4">
    <Search
      clearable
      class="mb-6"
      bind:value={search}
      placeholder="Search books and collections..."
    ></Search>

    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h5">Collections</Heading>
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
        <a href={`/library/collections/${collection.id}`}>
          <div class="relative group">
            <div
              class="w-full aspect-2/3 rounded-lg bg-linear-to-br from-primary-500 to-primary-700 dark:from-primary-600 dark:to-primary-800 flex items-center justify-center cursor-pointer hover:opacity-90 transition-opacity"
            >
              <span
                class="text-white text-center px-2 font-medium text-sm line-clamp-3"
                >{collection.name}</span
              >
            </div>
            <div
              class="absolute bottom-0 left-0 right-0 bg-black/60 text-white text-xs text-center py-1 rounded-b-lg"
            >
              {collection.book_count}
              {collection.book_count === 1 ? "book" : "books"}
            </div>
          </div>
          </a>
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
        <BookItem {book} />
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
  <Modal bind:open={showCollectionModal} size="md" class="w-full">
    <form
      onsubmit={(e) => {
        e.preventDefault();
        createCollection();
      }}
      class="space-y-4"
    >
      <h3 class="text-lg font-medium text-gray-900 dark:text-white">
        Create New Collection
      </h3>

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

  <!-- Import Success Modal -->
  <Modal bind:open={showResultModal} size="xs" autoclose>
    <div class="text-center">
      <CheckCircleSolid
        class="mx-auto mb-4 w-12 h-12 text-green-500 dark:text-green-400"
      />
      <h3 class="mb-5 text-lg font-normal text-gray-900 dark:text-white">
        Import Complete
      </h3>
      <div class="mb-6 text-sm text-gray-600 dark:text-gray-300">
        {#if importedBook}
          <p>Successfully added "<strong>{importedBook.title}</strong>"</p>
          <p class="mt-2 text-gray-500">{importedBook.total_pages} pages</p>
        {/if}
      </div>
      <Button color="red" class="w-full">Close</Button>
    </div>
  </Modal>

  <!-- Error Modal -->
  <Modal bind:open={showErrorModal} size="xs" autoclose>
    <div class="text-center">
      <CloseCircleSolid
        class="mx-auto mb-4 w-12 h-12 text-red-500 dark:text-red-400"
      />
      <h3 class="mb-5 text-lg font-normal text-gray-900 dark:text-white">
        Error
      </h3>
      <div class="mb-6 text-sm text-gray-600 dark:text-gray-300">
        <p>{errorMessage}</p>
      </div>
      <Button color="red" class="w-full">Close</Button>
    </div>
  </Modal>
{/if}

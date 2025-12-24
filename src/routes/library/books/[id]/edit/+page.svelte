<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import {
    Heading,
    Button,
    Label,
    Input,
    Toggle,
    Helper,
    Spinner,
    Modal,
    Badge,
    Checkbox,
    Card,
    P,
  } from "flowbite-svelte";
  import {
    ArrowLeftOutline,
    CloseCircleSolid,
    HeartSolid,
  } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";
  import { RadioDropdown } from "$components/settings";
  import { 
    libraryApi, 
    type Book, 
    type ReadingStatus,
    type CollectionWithCount,
    getCoverPath,
  } from "$lib";

  let bookId = $derived(Number(page.params.id));

  let isLoading = $state(true);
  let isSaving = $state(false);
  let book = $state<Book | null>(null);
  let allCollections = $state<CollectionWithCount[]>([]);
  let bookCollectionIds = $state<number[]>([]);
  
  // Form fields
  let title = $state("");
  let readingStatus = $state<ReadingStatus>("unread");
  let isFavorite = $state(false);
  let selectedCollectionIds = $state<number[]>([]);
  
  // Validation
  let titleError = $state("");
  
  // Error modal
  let showErrorModal = $state(false);
  let errorMessage = $state("");

  const readingStatusOptions: { value: ReadingStatus; label: string; description?: string }[] = [
    { value: "unread", label: "Unread", description: "Not started yet" },
    { value: "reading", label: "Reading", description: "Currently reading" },
    { value: "completed", label: "Completed", description: "Finished reading" },
    { value: "on_hold", label: "On Hold", description: "Paused for now" },
    { value: "dropped", label: "Dropped", description: "Stopped reading" },
  ];

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

  async function loadBook() {
    try {
      const [bookData, collectionsData, booksWithDetails] = await Promise.all([
        libraryApi.getBook(bookId),
        libraryApi.getCollections(),
        libraryApi.getBooks(),
      ]);
      
      book = bookData;
      allCollections = collectionsData;
      
      // Find the book's current collections from the full details
      const bookDetails = booksWithDetails.find(b => b.id === bookId);
      bookCollectionIds = bookDetails?.collection_ids ?? [];
      
      // Initialize form fields
      title = book.title;
      readingStatus = book.reading_status;
      isFavorite = book.is_favorite;
      selectedCollectionIds = [...bookCollectionIds];
    } catch (error) {
      console.error("Failed to load book:", error);
      showError(parseError(error));
    } finally {
      isLoading = false;
    }
  }

  function validate(): boolean {
    titleError = "";
    
    if (!title.trim()) {
      titleError = "Title is required";
      return false;
    }
    
    return true;
  }

  function toggleCollection(collectionId: number) {
    if (selectedCollectionIds.includes(collectionId)) {
      selectedCollectionIds = selectedCollectionIds.filter(id => id !== collectionId);
    } else {
      selectedCollectionIds = [...selectedCollectionIds, collectionId];
    }
  }

  async function handleSave() {
    if (!validate()) return;
    
    isSaving = true;
    try {
      // Update book details
      await libraryApi.updateBook(bookId, {
        title: title.trim(),
        readingStatus,
        isFavorite,
      });
      
      // Update collections if changed
      const collectionsChanged = 
        selectedCollectionIds.length !== bookCollectionIds.length ||
        !selectedCollectionIds.every(id => bookCollectionIds.includes(id));
      
      if (collectionsChanged) {
        await libraryApi.setBookCollections(bookId, selectedCollectionIds);
      }
      
      goto("/library");
    } catch (error) {
      console.error("Failed to update book:", error);
      showError(parseError(error));
    } finally {
      isSaving = false;
    }
  }

  function handleCancel() {
    goto("/library");
  }

  onMount(() => {
    loadBook();
  });

  let imageLoadFailed = $state(false);
  let coverPath = $derived(book ? getCoverPath(book.id) : "");
</script>

{#if isLoading}
  <LibrarySkeleton />
{:else if book}
  <div class="page-container p-4">
    <!-- Header -->
    <div class="mb-6 flex items-center gap-3">
      <Button
        color="alternative"
        size="sm"
        onclick={handleCancel}
        class="shrink-0"
      >
        <ArrowLeftOutline class="w-4 h-4 mr-1" />
        Back
      </Button>
      <Heading tag="h5" class="flex-1">Edit Book</Heading>
    </div>

    <form
      onsubmit={(e) => {
        e.preventDefault();
        handleSave();
      }}
      class="max-w-2xl space-y-6"
    >
      <!-- Book preview card -->
      <Card class="flex gap-4 p-4"  size="xl">
        <div class="w-20 h-28 shrink-0 rounded overflow-hidden bg-gray-200 dark:bg-gray-700">
          {#if imageLoadFailed}
            <div class="flex items-center justify-center w-full h-full text-gray-400">
              <span class="text-xs">No cover</span>
            </div>
          {:else}
            <img 
              src={coverPath} 
              alt={book.title}
              class="w-full h-full object-cover"
              onerror={() => imageLoadFailed = true}
            />
          {/if}
        </div>
        <div class="flex-1 min-w-0">
          <P size="sm" class="text-gray-500 dark:text-gray-400 mb-1">Original filename</P>
          <P size="sm" weight="medium" class="truncate mb-2">{book.filename}</P>
          <P size="sm" class="text-gray-500 dark:text-gray-400">
            {book.total_pages} pages · Added {new Date(book.added_at).toLocaleDateString()}
          </P>
        </div>
      </Card>

      <!-- Title -->
      <div>
        <Label for="book-title" class="mb-2">Title</Label>
        <Input
          id="book-title"
          bind:value={title}
          placeholder="Enter book title"
          color={titleError ? "red" : undefined}
          disabled={isSaving}
        />
        {#if titleError}
          <Helper class="mt-1" color="red">{titleError}</Helper>
        {/if}
      </div>

      <!-- Reading Status -->
      <div>
        <Label class="mb-2">Reading Status</Label>
        <RadioDropdown
          bind:value={readingStatus}
          options={readingStatusOptions}
        />
      </div>

      <!-- Favorite Toggle -->
      <div class="flex items-center gap-3">
        <Toggle bind:checked={isFavorite} disabled={isSaving} />
        <div class="flex items-center gap-2">
          <HeartSolid class="w-4 h-4 {isFavorite ? 'text-red-500' : 'text-gray-400'}" />
          <span class="text-sm text-gray-700 dark:text-gray-300">Favorite</span>
        </div>
      </div>

      <!-- Collections -->
      {#if allCollections.length > 0}
        <div>
          <Label class="mb-2">Collections</Label>
          <div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 space-y-2 max-h-48 overflow-y-auto">
            {#each allCollections as collection (collection.id)}
              <button
                type="button"
                class="flex items-center gap-3 w-full cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 p-2 rounded text-left"
                onclick={() => toggleCollection(collection.id)}
                disabled={isSaving}
              >
                <Checkbox 
                  checked={selectedCollectionIds.includes(collection.id)}
                  disabled={isSaving}
                />
                <span class="text-sm text-gray-700 dark:text-gray-300">{collection.name}</span>
                <Badge color="gray" class="ml-auto">{collection.book_count}</Badge>
              </button>
            {/each}
          </div>
          <Helper class="mt-1">Select collections this book belongs to</Helper>
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-3 pt-4">
        <Button
          type="button"
          color="alternative"
          class="flex-1"
          onclick={handleCancel}
          disabled={isSaving}
        >
          Cancel
        </Button>
        <Button
          type="submit"
          color="primary"
          class="flex-1"
          disabled={isSaving}
        >
          {#if isSaving}
            <Spinner size="4" class="mr-2" />
            Saving...
          {:else}
            Save Changes
          {/if}
        </Button>
      </div>
    </form>
  </div>

  <!-- Error Modal -->
  <Modal bind:open={showErrorModal} size="xs" autoclose>
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
{:else}
  <div class="page-container p-4">
    <P class="text-center text-gray-500 dark:text-gray-400">
      Book not found
    </P>
  </div>
{/if}

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
  } from "flowbite-svelte";
  import { PlusOutline, CheckCircleSolid, CloseCircleSolid } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";
  import { BookItem } from "$components/library";
  import { open } from '@tauri-apps/plugin-dialog';
  import { libraryApi, type BookWithDetails, type ImportResult } from "$lib";

  let isLoading = $state(true);
  let isImporting = $state(false);
  let search = $state("");
  let books = $state<BookWithDetails[]>([]);
  
  // Toast notifications
  let toastVisible = $state(false);
  let toastMessage = $state("");
  let toastType = $state<"success" | "error">("success");

  function showToast(message: string, type: "success" | "error") {
    toastMessage = message;
    toastType = type;
    toastVisible = true;
    setTimeout(() => toastVisible = false, 5000);
  }

  async function addBook() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Archive Files',
          extensions: ['zip', 'cbz']
        }
      ]
    });

    if (selected) {
      isImporting = true;
      try {
        const result: ImportResult = await libraryApi.importBooksFromArchive(selected);
        
        if (result.imported.length > 0) {
          await loadBooks();
          
          const importedCount = result.imported.length;
          const skippedCount = result.skipped.length;
          
          if (skippedCount > 0) {
            showToast(
              `Imported ${importedCount} book(s). ${skippedCount} duplicate(s) skipped.`,
              "success"
            );
          } else {
            showToast(`Successfully imported ${importedCount} book(s)!`, "success");
          }
        } else if (result.skipped.length > 0) {
          showToast(
            `All books were skipped. ${result.skipped[0].reason}`,
            "error"
          );
        } else {
          showToast("No books found in the archive", "error");
        }
      } catch (error) {
        console.error("Failed to import book:", error);
        showToast(`Failed to import: ${error}`, "error");
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
      showToast(`Failed to load books: ${error}`, "error");
    }
  }

  onMount(async () => {
    await loadBooks();
    isLoading = false;
  });
</script>

{#if isLoading}
  <LibrarySkeleton />
{:else}
  <div class="page-container p-4"> 
    <Search clearable class="mb-6" bind:value={search}></Search>
    
    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h5">Collections</Heading>
    </div>
    
    <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3">
      <Button 
        class="
          flex items-center justify-center w-full h-auto aspect-2/3 
          border-2 rounded-lg transition-colors
          bg-red-50 border-red-200 text-red-500 hover:bg-red-100
          dark:bg-gray-900 dark:border-red-900 dark:text-red-400 dark:hover:bg-gray-700
        "
      >
          <PlusOutline class="w-6 h-6" />
      </Button>
      
    </div>

    <Hr class="my-8"/>

    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h5">All volumes</Heading>
    </div>

    <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3">
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
          <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-red-500"></div>
        {:else}
          <PlusOutline class="w-6 h-6" />
        {/if}
      </Button>
      
      {#each books as book (book.id)}
        <BookItem {book} />
      {/each}
    </div>

  </div>

  <Modal open={isImporting} dismissable={false} autoclose={false} permanent class="z-50">
    <div class="text-center py-8">
      <Spinner size="12" class="mb-4" />
      <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Importing Archive</h3>
      <p class="text-sm text-gray-500 dark:text-gray-400">Processing files and checking for duplicates...</p>
    </div>
  </Modal>

  {#if toastVisible}
    <Toast
      color={toastType === "success" ? "green" : "red"}
      position="bottom-right"
      dismissable={false}
      class="mb-4 mr-4 fixed z-50"
    >
      <div class="flex items-center gap-2">
        {#if toastType === "success"}
          <CheckCircleSolid class="w-5 h-5" />
        {:else}
          <CloseCircleSolid class="w-5 h-5" />
        {/if}
        <span>{toastMessage}</span>
      </div>
    </Toast>
  {/if}
{/if}
<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import {
    Heading,
    Button,
    Label,
    Input,
    Textarea,
    Helper,
    Spinner,
    Modal,
    P,
  } from "flowbite-svelte";
  import {
    ArrowLeftOutline,
    CloseCircleSolid,
  } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";
  import { libraryApi, type Collection } from "$lib";

  let collectionId = $derived(Number(page.params.id));

  let isLoading = $state(true);
  let isSaving = $state(false);
  let collection = $state<Collection | null>(null);
  
  // Form fields
  let name = $state("");
  let description = $state("");
  
  // Validation
  let nameError = $state("");
  
  // Error modal
  let showErrorModal = $state(false);
  let errorMessage = $state("");

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

  async function loadCollection() {
    try {
      collection = await libraryApi.getCollection(collectionId);
      name = collection.name;
      description = collection.description ?? "";
    } catch (error) {
      console.error("Failed to load collection:", error);
      showError(parseError(error));
    } finally {
      isLoading = false;
    }
  }

  function validate(): boolean {
    nameError = "";
    
    if (!name.trim()) {
      nameError = "Collection name is required";
      return false;
    }
    
    return true;
  }

  async function handleSave() {
    if (!validate()) return;
    
    isSaving = true;
    try {
      await libraryApi.updateCollection(collectionId, {
        name: name.trim(),
        description: description.trim() || null,
      });
      goto(`/library/collections/${collectionId}`);
    } catch (error) {
      console.error("Failed to update collection:", error);
      showError(parseError(error));
    } finally {
      isSaving = false;
    }
  }

  function handleCancel() {
    goto(`/library/collections/${collectionId}`);
  }

  onMount(() => {
    loadCollection();
  });
</script>

{#if isLoading}
  <LibrarySkeleton />
{:else if collection}
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
      <Heading tag="h5" class="flex-1">Edit Collection</Heading>
    </div>

    <form
      onsubmit={(e) => {
        e.preventDefault();
        handleSave();
      }}
      class="max-w-xl space-y-6"
    >
      <!-- Name -->
      <div>
        <Label for="collection-name" class="mb-2">Name</Label>
        <Input
          id="collection-name"
          bind:value={name}
          placeholder="Enter collection name"
          color={nameError ? "red" : undefined}
          disabled={isSaving}
        />
        {#if nameError}
          <Helper class="mt-1" color="red">{nameError}</Helper>
        {/if}
      </div>

      <!-- Description -->
      <div>
        <Label for="collection-description" class="mb-2">Description (optional)</Label>
        <Textarea
          id="collection-description"
          bind:value={description}
          placeholder="Enter a description for this collection"
          rows={4}
          disabled={isSaving}
          class="resize-none w-full"
        />
      </div>

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
      Collection not found
    </P>
  </div>
{/if}

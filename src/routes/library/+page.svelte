<script lang="ts">
  import { onMount } from "svelte";
  import {
    Heading,
    P,
    Card,
    Button,
    Search,
    GradientButton,
    Hr,
  } from "flowbite-svelte";
  import { PlusOutline, FolderOpenOutline } from "flowbite-svelte-icons";
  import { LibrarySkeleton } from "$skeletons";

  let isLoading = $state(true);
  let search = $state("");

  onMount(() => {
    isLoading = false;
  });
</script>

{#if isLoading}
  <LibrarySkeleton />
{:else}
  <div class="page-container">
    <Search clearable class="mb-3" bind:value={search}></Search>
    <div class="mb-6 mt-2 flex items-center justify-between">
      <Heading tag="h5">Collections</Heading>
      <GradientButton outline color="red">
        <PlusOutline class="w-4 h-4 me-2" />
        Create new collection
      </GradientButton>
    </div>
    <Card class="flex items-center justify-center p-8">
      <div class="text-center">
        <FolderOpenOutline class="w-12 h-12 text-slate-400 mx-auto mb-4" />
        <P class="text-slate-500 dark:text-slate-400">
          Your collections will appear here.
        </P>
      </div>
    </Card>
    <Hr />
    <div class="mb-6 mt-2 flex items-center justify-between">
      <Heading tag="h5">All volumes</Heading>
      <GradientButton outline color="red">
        <PlusOutline class="w-4 h-4 me-2" />
        Import to Library
      </GradientButton>
    </div>
    <Card class="flex items-center justify-center p-8">
      <div class="text-center">
        <FolderOpenOutline class="w-12 h-12 text-slate-400 mx-auto mb-4" />
        <P class="text-slate-500 dark:text-slate-400">
          Your collections will appear here.
        </P>
      </div>
    </Card>
  </div>
{/if}

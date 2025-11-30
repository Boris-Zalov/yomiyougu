<!--
  LibrarySkeleton - Loading skeleton for library page
-->
<script lang="ts">
  import { Card, Skeleton, ImagePlaceholder } from "flowbite-svelte";
</script>

<div class="page-container">
  <div class="flex items-center justify-between mb-6">
    <Skeleton size="lg" class="w-32" />
    <Skeleton size="md" class="w-20 h-10 rounded-lg" />
  </div>

  <div class="grid gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
    {#each Array(10) as _, i}
      <Card>
        <ImagePlaceholder class="mb-3" />
        <Skeleton size="md" class="mb-2 w-full" />
        <Skeleton size="sm" class="w-2/3" />
      </Card>
    {/each}
  </div>
</div>

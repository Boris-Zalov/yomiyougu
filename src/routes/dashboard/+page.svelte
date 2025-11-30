<script lang="ts">
  import { onMount } from "svelte";
  import { Heading, P, Card } from "flowbite-svelte";
  import { BookOpenOutline, ClockOutline } from "flowbite-svelte-icons";
  import { DashboardSkeleton } from "$skeletons";

  let isLoading = $state(true);

  onMount(() => {
      isLoading = false;
  });
</script>

{#if isLoading}
  <DashboardSkeleton />
{:else}
<div class="page-container">
  <Heading tag="h2" class="mb-6">Dashboard</Heading>

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
    <Card class="flex items-center gap-4 pt-1 justify-center">
      <div class="p-3 rounded-lg bg-primary-100 dark:bg-primary-900/30">
        <ClockOutline class="w-6 h-6 text-primary-600 dark:text-primary-400" />
      </div>
      <div>
        <P class="text-sm text-slate-500 dark:text-slate-400">Continue Reading</P>
        <P class="font-semibold">No recent activity</P>
      </div>
    </Card>

    <Card class="flex items-center gap-4 pt-1 justify-center">
      <div class="p-3 rounded-lg bg-secondary-100 dark:bg-secondary-900/30">
        <BookOpenOutline class="w-6 h-6 text-secondary-600 dark:text-secondary-400" />
      </div>
      <div>
        <P class="text-sm text-slate-500 dark:text-slate-400">Library</P>
        <P class="font-semibold">0 items</P>
      </div>
    </Card>
  </div>

  <div class="mt-8">
    <Heading tag="h3" class="mb-4">Recent</Heading>
    <Card class="text-center p-8">
      <P class="text-slate-500 dark:text-slate-400">
        Your recently read manga will appear here.
      </P>
    </Card>
  </div>
</div>
{/if}
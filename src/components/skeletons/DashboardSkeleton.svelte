<!--
  DashboardSkeleton - Loading skeleton for dashboard page
-->
<script lang="ts">
  import { Card, Skeleton, CardPlaceholder, ListPlaceholder } from "flowbite-svelte";
</script>

<div class="page-container">
  <Skeleton size="lg" class="mb-6 w-48" />

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
    <Card>
      <div class="flex items-center gap-4">
        <Skeleton size="xl" class="w-12 h-12 rounded-lg" />
        <div class="flex-1">
          <Skeleton size="sm" class="mb-2 w-24" />
          <Skeleton size="md" class="w-32" />
        </div>
      </div>
    </Card>

    <Card>
      <div class="flex items-center gap-4">
        <Skeleton size="xl" class="w-12 h-12 rounded-lg" />
        <div class="flex-1">
          <Skeleton size="sm" class="mb-2 w-20" />
          <Skeleton size="md" class="w-28" />
        </div>
      </div>
    </Card>
  </div>

  <div class="mt-8">
    <Skeleton size="lg" class="mb-4 w-32" />
    <Card>
      <ListPlaceholder />
    </Card>
  </div>
</div>

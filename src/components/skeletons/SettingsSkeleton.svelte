<!--
  SettingsSkeleton - Loading skeleton for settings page
-->
<script lang="ts">
  import { Card, Skeleton } from "flowbite-svelte";
</script>

<div class="page-container">
  <div class="flex items-center justify-between mb-6">
    <Skeleton size="lg" class="w-32" />
  </div>

  <div class="space-y-6">
    {#each Array(4) as _, categoryIndex}
      <Card>
        <Skeleton size="lg" class="mb-4 w-40" />
        <div class="space-y-4">
          {#each Array(3) as _, settingIndex}
            <div class="flex items-center justify-between py-2">
              <div class="flex-1">
                <Skeleton size="md" class="mb-2 w-48" />
                <Skeleton size="sm" class="w-64" />
              </div>
              <Skeleton size="md" class="w-16 h-8 rounded-lg" />
            </div>
          {/each}
        </div>
      </Card>
    {/each}
  </div>
</div>

<!--
  SettingCategory - A group of related settings with header
-->
<script lang="ts">
  import { Card, Heading, Helper } from "flowbite-svelte";
  import type { SettingCategory as CategoryType, SettingValue } from "$lib/types/settings";
  import SettingRow from "./SettingRow.svelte";

  interface Props {
    category: CategoryType;
    onchange?: (key: string, value: SettingValue) => void;
  }

  let { category, onchange }: Props = $props();
</script>

<Card class="p-0 overflow-hidden" size="xl">
  <div class="px-4 py-3">
    <Heading tag="h1" class="text-base font-semibold">{category.label}</Heading>
    {#if category.description}
      <Helper>
        {category.description}
      </Helper>
    {/if}
  </div>
  <div class="divide-y divide-slate-100 dark:divide-slate-700/50 px-4">
    {#each category.settings as setting (setting.key)}
      <SettingRow {setting} {onchange} />
    {/each}
  </div>
</Card>

<!--
  SettingCategory - A group of related settings with header
-->
<script lang="ts">
	import { Card, Heading, Helper } from "flowbite-svelte";
	import {
		PaletteOutline,
		BookOpenOutline,
		BookOutline,
		CloudArrowUpOutline,
		CogOutline,
	} from "flowbite-svelte-icons";
	import type { SettingCategory as CategoryType, SettingValue } from "$lib/types/settings";
	import SettingRow from "./SettingRow.svelte";

	interface Props {
		category: CategoryType;
		onchange?: (key: string, value: SettingValue) => void;
	}

	let { category, onchange }: Props = $props();

	// Map icon names to components
	const iconMap: Record<string, any> = {
		palette: PaletteOutline,
		"book-open": BookOpenOutline,
		library: BookOutline,
		cloud: CloudArrowUpOutline,
		cog: CogOutline,
	};

	let IconComponent = $derived(
		category.icon && iconMap[category.icon] ? iconMap[category.icon] : null
	);
</script>

<Card class="p-0 overflow-hidden" size="xl">
	<div class="px-4 py-3">
		<div class="flex items-center gap-2">
			{#if IconComponent}
				<IconComponent class="w-5 h-5 text-gray-500 dark:text-gray-400" />
			{/if}
			<Heading tag="h1" class="text-base font-semibold">{category.label}</Heading>
		</div>
		{#if category.description}
			<Helper class="mt-1">
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

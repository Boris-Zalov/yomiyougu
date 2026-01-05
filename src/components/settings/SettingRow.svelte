<!--
  SettingRow - A single setting with label, description, and widget
-->
<script lang="ts">
	import { Helper, Label } from "flowbite-svelte";
	import type { SettingItem, SettingValue } from "$lib/types/settings";
	import SettingWidget from "./SettingWidget.svelte";

	interface Props {
		setting: SettingItem;
		onchange?: (key: string, value: SettingValue) => void;
	}

	let { setting, onchange }: Props = $props();
</script>

<div class="setting-group py-3">
	<div class="flex items-center justify-between gap-4">
		<div class="flex-1 min-w-0">
			<Label class="setting-label">{setting.label}</Label>
			{#if setting.description}
				<Helper class="setting-description mt-0.5">{setting.description}</Helper>
			{/if}
			{#if setting.requiresRestart}
				<Helper class="text-xs text-amber-600 dark:text-amber-400 mt-0.5">Requires restart</Helper>
			{/if}
		</div>
		<div class="shrink-0">
			<SettingWidget {setting} {onchange} />
		</div>
	</div>
</div>

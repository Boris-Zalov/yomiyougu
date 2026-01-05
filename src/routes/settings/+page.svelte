<script lang="ts">
	import { onMount, tick } from "svelte";
	import { ask } from "@tauri-apps/plugin-dialog";
	import { beforeNavigate, goto } from "$app/navigation";
	import { Button, Spinner, Heading, Alert } from "flowbite-svelte";
	import { CheckCircleSolid, FloppyDiskSolid, RefreshOutline } from "flowbite-svelte-icons";

	import {
		settingsApi,
		authApi,
		type SettingCategory,
		type SettingValue,
		type AuthStatus,
	} from "$lib";
	import { applyTheme } from "$lib/utils/theme";
	import {
		SettingCategory as SettingCategoryComponent,
		SyncSettingsCategory,
	} from "$components/settings";
	import { SettingsSkeleton } from "$skeletons";

	let categories = $state<SettingCategory[]>([]);
	let _originalCategories = $state<SettingCategory[]>([]);
	let pendingChanges = $state<Map<string, SettingValue>>(new Map());
	let isLoading = $state(true);
	let isSaving = $state(false);
	let error = $state<string | null>(null);
	let showSaved = $state(false);
	let showReset = $state(false);
	let authStatus = $state<AuthStatus>({ isAuthenticated: false, needsRefresh: false });

	let hasChanges = $derived(pendingChanges.size > 0);

	// Separate sync category from other categories
	let regularCategories = $derived(categories.filter((c) => c.id !== "sync"));
	let syncCategory = $derived(categories.find((c) => c.id === "sync"));

	beforeNavigate(async ({ cancel, to }) => {
		if (!hasChanges) return;
		cancel();
		await tick();

		const confirmed = await ask(
			"You have unsaved changes. Are you sure you want to leave without saving?",
			{
				title: "Unsaved changes",
				kind: "warning",
			}
		);

		if (confirmed) {
			pendingChanges.clear();
			pendingChanges = new Map();

			if (to) {
				goto(to.url);
			}
		}
	});

	async function loadSettings() {
		isLoading = true;
		error = null;
		try {
			const [settingsResult, authResult] = await Promise.all([
				settingsApi.getSettingsSchema(),
				authApi.getAuthStatus(),
			]);
			categories = settingsResult;
			authStatus = authResult;
		} catch (err) {
			console.error("Failed to load settings:", err);
			error = typeof err === "string" ? err : "Failed to load settings";
		} finally {
			isLoading = false;
		}
	}

	async function refreshAuthStatus() {
		try {
			authStatus = await authApi.getAuthStatus();
		} catch (err) {
			console.error("Failed to refresh auth status:", err);
		}
	}

	function handleChange(key: string, value: SettingValue) {
		for (const category of categories) {
			for (const setting of category.settings) {
				if (setting.key === key) {
					setting.value = value;
					break;
				}
			}
		}

		pendingChanges.set(key, value);
		pendingChanges = new Map(pendingChanges);
	}

	async function saveChanges() {
		if (!hasChanges) return;

		isSaving = true;
		error = null;

		try {
			const formData: Record<string, SettingValue> = {};
			pendingChanges.forEach((value, key) => {
				formData[key] = value;
			});

			await settingsApi.saveSettings(formData);

			const themeChange = pendingChanges.get("appearance.theme");
			if (themeChange) {
				applyTheme(themeChange as "light" | "dark" | "system");
			}

			pendingChanges.clear();
			pendingChanges = new Map();

			showSaved = true;
			setTimeout(() => (showSaved = false), 2000);
		} catch (err) {
			console.error("Failed to save settings:", err);
			error = typeof err === "string" ? err : "Failed to save settings";
		} finally {
			isSaving = false;
		}
	}

	async function resetAll() {
		const confirmed = await confirm("Reset all settings to defaults?");
		if (!confirmed) return;

		isLoading = true;
		error = null;
		try {
			await settingsApi.resetAllSettings();
			await loadSettings();

			pendingChanges.clear();
			pendingChanges = new Map();

			showReset = true;
			setTimeout(() => (showReset = false), 2000);
		} catch (err) {
			console.error("Failed to reset settings:", err);
			error = typeof err === "string" ? err : "Failed to reset settings";
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadSettings();
	});
</script>

<div class="page-container">
	<div class="flex items-center justify-between mb-6">
		<Heading tag="h2">Settings</Heading>
		{#if hasChanges}
			<Button size="sm" color="primary" onclick={saveChanges} disabled={isSaving}>
				{#if isSaving}
					<Spinner size="4" class="me-2" />
				{:else}
					<FloppyDiskSolid class="w-4 h-4 me-2" />
				{/if}
				Save Changes
			</Button>
		{/if}
	</div>

	{#if error}
		<Alert color="red" class="mb-4">
			{error}
		</Alert>
	{/if}

	{#if isLoading}
		<SettingsSkeleton />
	{:else}
		<div class="space-y-6 w-full">
			{#each regularCategories as category (category.id)}
				<SettingCategoryComponent {category} onchange={handleChange} />
			{/each}

			{#if syncCategory}
				<SyncSettingsCategory
					category={syncCategory}
					{authStatus}
					onchange={handleChange}
					onAuthChange={refreshAuthStatus}
				/>
			{/if}

			<div>
				<Button color="alternative" size="sm" onclick={resetAll}>
					<RefreshOutline class="w-4 h-4 me-2" />
					Reset All to Defaults
				</Button>
			</div>
		</div>
	{/if}
</div>

{#if showSaved}
	<div
		class="fixed right-4 flex items-center gap-2 px-4 py-3 rounded-lg bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-100 shadow-lg top-4 md:bottom-4 md:top-auto"
	>
		<CheckCircleSolid class="w-5 h-5" />
		<span class="text-sm font-medium">Settings saved</span>
	</div>
{/if}

{#if showReset}
	<div
		class="fixed right-4 flex items-center gap-2 px-4 py-3 rounded-lg bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-100 shadow-lg top-4 md:bottom-4 md:top-auto"
	>
		<CheckCircleSolid class="w-5 h-5" />
		<span class="text-sm font-medium">Settings reset to defaults</span>
	</div>
{/if}

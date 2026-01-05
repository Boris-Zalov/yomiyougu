<!--
  SetupWizard - First-run setup flow
  Shows only essential settings to get users started quickly
-->
<script lang="ts">
	import { fade } from "svelte/transition";
	import {
		StepIndicator,
		Button,
		Card,
		Checkbox,
		Heading,
		P,
		Spinner,
		Modal,
		List,
		Li,
		A,
	} from "flowbite-svelte";
	import {
		BookOpenOutline,
		CloudArrowUpOutline,
		FolderOpenOutline,
		ExclamationCircleOutline,
		CodeBranchOutline,
	} from "flowbite-svelte-icons";

	import { settingsApi, applyTheme, type ThemeMode } from "$lib";
	import { RadioDropdown } from "$components/settings";

	interface Props {
		onFinish: () => void;
	}

	let { onFinish }: Props = $props();

	const steps = [
		{ id: "welcome", label: "Welcome" },
		{ id: "settings", label: "Settings" },
		{ id: "done", label: "Ready" },
	];

	let activeIndex = $state(0);
	let isLoading = $state(false);
	let errorMessage = $state("");
	let showError = $state(false);

	let acknowledgedInfo = $state(false);
	let themeMode = $state<ThemeMode>("system");
	let readingDirection = $state("rtl");

	let activeStep = $derived(steps[activeIndex]);
	let isFirstStep = $derived(activeIndex === 0);
	let isLastStep = $derived(activeIndex === steps.length - 1);
	let stepLabels = $derived(steps.map((s) => s.label));

	// Apply theme preview in real-time
	$effect(() => {
		applyTheme(themeMode);
	});

	// Navigation
	function canProceed(): boolean {
		if (activeStep.id === "welcome") return acknowledgedInfo;
		return true;
	}

	function next() {
		if (isLastStep) {
			finishSetup();
		} else if (canProceed()) {
			activeIndex++;
		}
	}

	function back() {
		errorMessage = "";
		showError = false;
		if (!isFirstStep) activeIndex--;
	}

	async function finishSetup() {
		isLoading = true;
		errorMessage = "";

		try {
			await settingsApi.completeSetup({
				"appearance.theme": themeMode,
				"reading.direction": readingDirection,
			});
			onFinish();
		} catch (err) {
			console.error("Setup failed:", err);
			errorMessage = typeof err === "string" ? err : "Failed to save configuration.";
			showError = true;
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="flex flex-col h-screen w-full" in:fade>
	<!-- Step indicator -->
	<div class="w-full pt-8 px-4 shrink-0 flex justify-center">
		<div class="w-full max-w-xl">
			<StepIndicator steps={stepLabels} currentStep={activeIndex + 1} />
		</div>
	</div>

	<!-- Content area -->
	<div class="flex-1 flex flex-col items-center justify-center px-6 py-8 overflow-y-auto">
		<div class="w-full max-w-lg">
			{#if activeStep.id === "welcome"}
				<div class="text-center space-y-6" in:fade={{ duration: 200 }}>
					<div>
						<Heading tag="h2">Welcome to Yomiyougu</Heading>
						<P class="mt-2 text-slate-600 dark:text-slate-400">A personal comic and manga reader</P>
					</div>

					<Card class="text-left overflow-hidden p-6" size="xl">
						<div
							class="max-h-64 overflow-y-auto text-sm text-slate-600 dark:text-slate-400 space-y-4"
						>
							<div>
								<Heading
									tag="h6"
									class="mb-1 text-slate-800 dark:text-slate-200 flex items-center gap-2"
								>
									<BookOpenOutline class="w-4 h-4" />
									What This App Does
								</Heading>
								<p class="leading-relaxed">
									Yomiyougu is a personal reader for your own comic and manga files. It does <strong
										>not</strong
									> distribute, download, or provide access to any copyrighted content. You are responsible
									for ensuring you have the rights to read any files you import.
								</p>
							</div>

							<div>
								<Heading
									tag="h6"
									class="mb-1 text-slate-800 dark:text-slate-200 flex items-center gap-2"
								>
									<CloudArrowUpOutline class="w-4 h-4" />
									Cloud Sync
								</Heading>
								<p class="leading-relaxed">
									If you enable sync, your files will be stored in <strong
										>your personal Google Drive</strong
									>. Be mindful of your storage limits — comic files can be large and may consume
									significant space.
								</p>
							</div>

							<div>
								<Heading
									tag="h6"
									class="mb-1 text-slate-800 dark:text-slate-200 flex items-center gap-2"
								>
									<FolderOpenOutline class="w-4 h-4" />
									Supported Formats
								</Heading>
								<p class="leading-relaxed">
									ZIP, CBZ, RAR*, and CBR* archives containing images.
									<span class="text-xs">(*RAR/CBR not supported on Android)</span>
								</p>
							</div>

							<div>
								<Heading
									tag="h6"
									class="mb-1 text-slate-800 dark:text-slate-200 flex items-center gap-2"
								>
									<ExclamationCircleOutline class="w-4 h-4" />
									Disclaimer
								</Heading>
								<p class="leading-relaxed">
									This software is provided "as is", without warranty of any kind. Use at your own
									risk.
								</p>
							</div>

							<div>
								<Heading
									tag="h6"
									class="mb-1 text-slate-800 dark:text-slate-200 flex items-center gap-2"
								>
									<CodeBranchOutline class="w-4 h-4" />
									Open Source
								</Heading>
								<p class="leading-relaxed">
									Found a bug or want to contribute? Visit us on
									<A href="https://github.com/Boris-Zalov/yomiyougu" target="_blank" class="inline"
										>GitHub</A
									>.
								</p>
							</div>
						</div>
					</Card>

					<div class="flex">
						<Checkbox bind:checked={acknowledgedInfo}>I understand and want to continue</Checkbox>
					</div>
				</div>
			{:else if activeStep.id === "settings"}
				<div class="space-y-8" in:fade={{ duration: 200 }}>
					<div class="text-center">
						<Heading tag="h2">Quick Setup</Heading>
						<P class="mt-2 text-slate-600 dark:text-slate-400">
							Configure the essentials. You can change these later.
						</P>
					</div>

					<div class="space-y-2">
						<Heading class="setting-label" tag="h6">Theme</Heading>
						<RadioDropdown
							bind:value={themeMode}
							options={[
								{ value: "light", label: "Light Mode", description: "Clean, bright appearance" },
								{ value: "dark", label: "Dark Mode", description: "Easy on the eyes" },
								{ value: "system", label: "System", description: "Match your device" },
							]}
						/>
					</div>

					<div class="space-y-2">
						<Heading class="setting-label" tag="h6">Default reading direction</Heading>
						<RadioDropdown
							bind:value={readingDirection}
							options={[
								{ value: "rtl", label: "Right to Left", description: "Traditional manga style" },
								{ value: "ltr", label: "Left to Right", description: "Western comic style" },
								{
									value: "vertical",
									label: "Vertical Scroll",
									description: "Webtoon/Manhwa style",
								},
							]}
						/>
					</div>
				</div>
			{:else if activeStep.id === "done"}
				<div class="text-center space-y-6" in:fade={{ duration: 200 }}>
					<div class="text-6xl">🎉</div>
					<div>
						<Heading tag="h2">You're All Set!</Heading>
						<P class="mt-2 text-slate-600 dark:text-slate-400">
							Your reading experience awaits. Explore more settings anytime.
						</P>
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Footer navigation -->
	<div class="shrink-0 border-t border-slate-200 dark:border-slate-700 px-6 py-4">
		<div class="max-w-lg mx-auto flex justify-between items-center">
			<Button color="alternative" onclick={back} disabled={isFirstStep || isLoading}>Back</Button>

			<Button
				color="primary"
				onclick={next}
				disabled={!canProceed() || isLoading}
				class="min-w-[120px]"
			>
				{#if isLoading}
					<Spinner class="me-2" size="4" />
					Saving...
				{:else}
					{isLastStep ? "Get Started" : "Next"}
				{/if}
			</Button>
		</div>
	</div>

	<!-- Error modal -->
	<Modal bind:open={showError} size="md" autoclose>
		<div class="text-center">
			<Heading tag="h3" class="mb-4 text-lg">{errorMessage}</Heading>
			<Button color="red" onclick={() => (showError = false)}>Close</Button>
		</div>
	</Modal>
</div>

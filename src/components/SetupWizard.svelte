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
  } from "flowbite-svelte";

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

  let acceptedLicense = $state(false);
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
    if (activeStep.id === "welcome") return acceptedLicense;
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
      errorMessage =
        typeof err === "string" ? err : "Failed to save configuration.";
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
            <P class="mt-2 text-slate-600 dark:text-slate-400">
              Let's get you set up to start reading.
            </P>
          </div>

          <Card class="text-left overflow-hidden p-6">
            <div class="max-h-48 overflow-y-auto text-sm text-slate-600 dark:text-slate-400">
              <Heading tag="h5" class="mb-2">MIT License</Heading>
              <p class="leading-relaxed">
                Permission is hereby granted, free of charge, to any person
                obtaining a copy of this software and associated documentation
                files (the "Software"), to deal in the Software without
                restriction, including without limitation the rights to use,
                copy, modify, merge, publish, distribute, sublicense, and/or
                sell copies of the Software...
              </p>
            </div>
          </Card>

          <div class="flex">
            <Checkbox bind:checked={acceptedLicense}>
              I accept the license terms
            </Checkbox>
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
            <span class="setting-label">Theme</span>
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
            <span class="setting-label">Default reading Direction</span>
            <RadioDropdown
              bind:value={readingDirection}
              options={[
                { value: "rtl", label: "Right to Left", description: "Traditional manga style" },
                { value: "ltr", label: "Left to Right", description: "Western comic style" },
                { value: "vertical", label: "Vertical Scroll", description: "Webtoon/Manhwa style" },
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
      <Button
        color="alternative"
        onclick={back}
        disabled={isFirstStep || isLoading}
      >
        Back
      </Button>

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
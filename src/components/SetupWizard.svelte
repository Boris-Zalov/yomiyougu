<script>
    import { invoke } from "@tauri-apps/api/core";
    import { fade } from "svelte/transition";

    import {
        CloudArrowUpOutline,
        CheckCircleSolid,
        SunSolid,
        MoonSolid,
        DesktopPcOutline,
    } from "flowbite-svelte-icons";

    import {
        P,
        StepIndicator,
        ButtonToggleGroup,
        ButtonToggle,
        Button,
        Input,
        Label,
        Modal,
        Card,
        Checkbox,
        Heading,
        Spinner,
        Alert,
    } from "flowbite-svelte";

    let { onFinish } = $props();

    // ---------------------------------------------------------------------------
    // CONFIGURATION
    // ---------------------------------------------------------------------------
    const steps = [
        { id: "welcome", label: "Welcome" },
        { id: "profile", label: "Appearance" },
        { id: "sync", label: "Cloud Sync" },
    ];

    // ---------------------------------------------------------------------------
    // STATE
    // ---------------------------------------------------------------------------
    let activeIndex = $state(0);
    let isLoading = $state(false);
    let errorMessage = $state("");
    let showError = $state(false);

    let activeStep = $derived(steps[activeIndex]);
    let isFirstStep = $derived(activeIndex === 0);
    let isLastStep = $derived(activeIndex === steps.length - 1);

    // Derived state specifically for the BreadcrumbStepper structure
    let stepIndicator = $derived(
        steps.map((step) => (step.label)),
    );

    let config = $state({
        username: "",
        theme_mode: "system",
        accepted_license: false,
        google_drive_enabled: false,
    });

    // ---------------------------------------------------------------------------
    // EFFECTS
    // ---------------------------------------------------------------------------
    $effect(() => {
        const html = document.documentElement;
        const theme = config.theme_mode;

        html.classList.remove("dark");

        if (theme === "dark") {
            html.classList.add("dark");
        } else if (theme === "system") {
            if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
                html.classList.add("dark");
            }
        }
    });

    // ---------------------------------------------------------------------------
    // ACTIONS
    // ---------------------------------------------------------------------------
    /**
     * @param {string | string[] | null} value
     */
    function handleThemeSelect(value) {
        if (typeof value === "string") {
            config.theme_mode = value;
        }
    }

    function next() {
        if (!isLastStep && canProceed()) {
            activeIndex++;
        } else if (isLastStep) {
            finishSetup();
        }
    }

    function back() {
        errorMessage = "";
        showError = false;
        if (!isFirstStep) activeIndex--;
    }

    function canProceed() {
        if (activeStep.id === "welcome") return config.accepted_license;
        if (activeStep.id === "profile")
            return config.username.trim().length > 0;
        return true;
    }

    async function finishSetup() {
        isLoading = true;
        errorMessage = "";

        try {
            await invoke("save_config", { config: config });
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

<div
    class="flex flex-col h-screen w-full bg-gray-50 dark:bg-gray-900 transition-colors duration-300"
    in:fade
>
    <div class="w-full pt-8 px-4 shrink-0 flex justify-center">
        <div class="w-full max-w-4xl flex justify-center">
            <StepIndicator class="w-full" steps={stepIndicator} currentStep={activeIndex + 1}/>
        </div>
    </div>

    <div
        class="grow flex flex-col items-center justify-center w-full max-w-2xl mx-auto px-6 py-4 overflow-y-auto"
    >
        {#if activeStep.id === "welcome"}
            <div
                class="text-center w-full space-y-4"
                in:fade={{ duration: 200 }}
            >
                <Heading tag="h2" class="mb-2">Welcome</Heading>
                <P class="text-gray-500 dark:text-gray-400"
                    >Let's get you set up to read.</P
                >

                <Card
                    class="w-full max-w-none text-left p-0 overflow-hidden shadow-none border border-gray-200 dark:border-gray-700"
                >
                    <div
                        class="h-48 md:h-64 overflow-y-auto p-6 bg-white dark:bg-gray-800 text-sm"
                    >
                        <Heading tag="h5" class="mb-2">MIT License</Heading>
                        <P
                            class="mb-3 text-xs text-gray-600 dark:text-gray-300 leading-relaxed"
                        >
                            Permission is hereby granted, free of charge, to any
                            person obtaining a copy of this software and
                            associated documentation files (the "Software"), to
                            deal in the Software without restriction... (Full
                            license text placeholder)
                        </P>
                    </div>
                </Card>

                <div class="pt-2">
                    <Checkbox
                        bind:checked={config.accepted_license}
                        class="inline-flex"
                    >
                        I accept the license terms
                    </Checkbox>
                </div>
            </div>
        {/if}

        {#if activeStep.id === "profile"}
            <div class="w-full max-w-md space-y-8" in:fade={{ duration: 200 }}>
                <div class="text-center">
                    <Heading tag="h2">Customize</Heading>
                    <P class="mt-2 text-gray-500"
                        >Personalize your reading experience.</P
                    >
                </div>

                <div>
                    <Label for="username" class="mb-2">Username</Label>
                    <Input
                        id="username"
                        bind:value={config.username}
                        placeholder="ReaderOne"
                        size="lg"
                    />
                </div>

                <div>
                    <Label class="mb-2">Theme Preference</Label>

                    <ButtonToggleGroup
                        class="w-full"
                        value={config.theme_mode}
                        onSelect={handleThemeSelect}
                    >
                        <ButtonToggle
                            value="light"
                            selected={config.theme_mode === "light"}
                            class="w-full"
                        >
                            {#snippet iconSlot()}
                                <SunSolid class="me-2 w-4 h-4" />
                            {/snippet}
                            Light
                        </ButtonToggle>
                        <ButtonToggle
                            value="dark"
                            selected={config.theme_mode === "dark"}
                            class="w-full"
                        >
                            {#snippet iconSlot()}
                                <MoonSolid class="me-2 w-4 h-4" />
                            {/snippet}
                            Dark
                        </ButtonToggle>
                        <ButtonToggle
                            value="system"
                            selected={config.theme_mode === "system"}
                            class="w-full"
                        >
                            {#snippet iconSlot()}
                                <DesktopPcOutline class="me-2 w-4 h-4" />
                            {/snippet}
                            System
                        </ButtonToggle>
                    </ButtonToggleGroup>
                </div>
            </div>
        {/if}

        {#if activeStep.id === "sync"}
            <div
                class="w-full max-w-md text-center space-y-6"
                in:fade={{ duration: 200 }}
            >
                <Heading tag="h2">Cloud Sync</Heading>

                <div
                    class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 bg-gray-50 dark:bg-gray-800"
                >
                    <div class="mb-4 flex justify-center text-gray-400">
                        <CloudArrowUpOutline class="w-12 h-12" />
                    </div>

                    {#if !config.google_drive_enabled}
                        <div class="space-y-4">
                            <P class="text-sm text-gray-500"
                                >Sync your library across devices.</P
                            >
                            <Button
                                class="bg-[#4285F4] hover:bg-[#3367d6] text-white w-full shadow-md"
                                onclick={() =>
                                    (config.google_drive_enabled = true)}
                            >
                                Connect Google Drive
                            </Button>
                        </div>
                    {:else}
                        <div class="space-y-4">
                            <Alert
                                color="green"
                                class="flex items-center justify-center"
                            >
                                <span
                                    class="font-medium flex items-center gap-2"
                                >
                                    <CheckCircleSolid class="w-4 h-4" /> Drive Connected
                                </span>
                            </Alert>
                            <Button
                                color="red"
                                outline
                                size="xs"
                                onclick={() =>
                                    (config.google_drive_enabled = false)}
                            >
                                Disconnect
                            </Button>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}
    </div>

    <div
        class="w-full max-w-3xl mx-auto px-6 py-6 border-t border-gray-200 dark:border-gray-700 shrink-0 mt-auto"
    >
        <div class="flex justify-between items-center">
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
                    <Spinner class="me-3" size="4" color="primary" />
                    Saving...
                {:else}
                    {isLastStep ? "Finish Setup" : "Next"}
                {/if}
            </Button>
        </div>
    </div>

    <Modal bind:open={showError} size="xs" autoclose>
        <div class="text-center">
            <Heading
                tag="h3"
                class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400"
            >
                {errorMessage}
            </Heading>
            <Button color="red" onclick={() => (showError = false)}
                >Close</Button
            >
        </div>
    </Modal>
</div>

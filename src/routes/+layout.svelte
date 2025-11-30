<script lang="ts">
  import "../app.css";
  import { platform } from "@tauri-apps/plugin-os";
  import { fade } from "svelte/transition";
  import { page } from "$app/state";

  import { settingsApi, applyTheme, type ThemeMode } from "$lib";
  import SplashScreen from "$components/SplashScreen.svelte";
  import SetupWizard from "$components/SetupWizard.svelte";
  import DesktopNavigation from "$components/DesktopNavigation.svelte";
  import MobileNavigation from "$components/MobileNavigation.svelte";

  let { children } = $props();

  const currentPlatform = platform();
  const isMobile = currentPlatform === "android" || currentPlatform === "ios";

  let showSplash = $state(true);
  let showSetup = $state(false);
  let appReady = $state(false);

  let activePath = $derived(page.url.pathname);

  async function handleSplashComplete() {
    try {
      const exists = await settingsApi.checkSettingsExists();
      
      if (exists) {
        const settings = await settingsApi.getSettings();
        const theme = (settings.categories
          .find((c) => c.id === "appearance")
          ?.settings.find((s) => s.key === "appearance.theme")
          ?.value || "system") as ThemeMode;
        applyTheme(theme);
        
        showSetup = false;
        appReady = true;
      } else {
        showSetup = true;
      }
    } catch (e) {
      console.error("Config check failed:", e);
      showSetup = true;
    } finally {
      showSplash = false;
    }
  }

  async function handleSetupFinished() {
    showSetup = false;
    appReady = true;
    
    try {
      const settings = await settingsApi.getSettings();
      const theme = (settings.categories
        .find((c) => c.id === "appearance")
        ?.settings.find((s) => s.key === "appearance.theme")
        ?.value || "system") as ThemeMode;
      applyTheme(theme);
    } catch (e) {
      console.error("Failed to load theme after setup:", e);
    }
  }
</script>

{#if showSplash}
  <SplashScreen onComplete={handleSplashComplete} />
{:else if showSetup}
  <SetupWizard onFinish={handleSetupFinished} />
{:else if appReady}
  <div class="h-screen w-screen flex flex-col overflow-hidden" transition:fade>
    {#if isMobile}
      <main class="flex-1 overflow-y-auto overscroll-contain pt-7 pb-26 relative isolate">
        {@render children()}
      </main>
      <MobileNavigation />
    {:else}
      <DesktopNavigation {activePath} />
      <main class="flex-1 overflow-y-auto relative isolate">
        {@render children()}
      </main>
    {/if}
  </div>
{/if}

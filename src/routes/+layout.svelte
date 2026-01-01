<script lang="ts">
  import "../app.css";
  import { platform } from "@tauri-apps/plugin-os";
  import { fade } from "svelte/transition";
  import { page, navigating } from "$app/state";

  import { settingsApi, applyTheme, type ThemeMode, setIsAndroid } from "$lib";
  import SplashScreen from "$components/SplashScreen.svelte";
  import SetupWizard from "$components/SetupWizard.svelte";
  import DesktopNavigation from "$components/DesktopNavigation.svelte";
  import MobileNavigation from "$components/MobileNavigation.svelte";
  import { LibrarySkeleton, DashboardSkeleton, SettingsSkeleton, ReaderSkeleton } from "$components/skeletons";

  let { children } = $props();

  // Determine which skeleton to show during navigation
  let targetPath = $derived(navigating?.to?.url.pathname);
  let isNavigating = $derived(!!navigating);

  function getSkeletonForPath(path: string | undefined) {
    if (!path) return null;
    if (path.startsWith('/reader')) return 'reader';
    if (path.startsWith('/library')) return 'library';
    if (path.startsWith('/dashboard') || path === '/') return 'dashboard';
    if (path.startsWith('/settings')) return 'settings';
    return null;
  }

  let skeletonType = $derived(getSkeletonForPath(targetPath));
  
  // Check if we're currently on the reader page (hide navigation)
  let isReaderPage = $derived(page.url.pathname.startsWith('/reader'));

  const currentPlatform = platform();
  const isMobile = currentPlatform === "android" || currentPlatform === "ios";
  const isAndroid = currentPlatform === "android";
  
  setIsAndroid(isAndroid);

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
  {#if isReaderPage}
    <!-- Reader has no navigation wrapper -->
    {#if isNavigating && skeletonType === 'reader'}
      <ReaderSkeleton />
    {:else}
      {@render children()}
    {/if}
  {:else}
    <div class="h-screen w-screen flex flex-col overflow-hidden" transition:fade>
      {#if isMobile}
        <main class="flex-1 overflow-y-auto overscroll-contain pt-7 pb-26 relative isolate">
          {#if isNavigating && skeletonType}
            {#if skeletonType === 'library'}
              <LibrarySkeleton />
            {:else if skeletonType === 'dashboard'}
              <DashboardSkeleton />
            {:else if skeletonType === 'settings'}
              <SettingsSkeleton />
            {:else if skeletonType === 'reader'}
              <ReaderSkeleton />
            {/if}
          {:else}
            {@render children()}
          {/if}
        </main>
        <MobileNavigation />
      {:else}
        <DesktopNavigation {activePath} />
        <main class="flex-1 overflow-y-auto relative isolate">
          {#if isNavigating && skeletonType}
            {#if skeletonType === 'library'}
              <LibrarySkeleton />
            {:else if skeletonType === 'dashboard'}
              <DashboardSkeleton />
            {:else if skeletonType === 'settings'}
              <SettingsSkeleton />
            {:else if skeletonType === 'reader'}
              <ReaderSkeleton />
            {/if}
          {:else}
            {@render children()}
          {/if}
        </main>
      {/if}
    </div>
  {/if}
{/if}

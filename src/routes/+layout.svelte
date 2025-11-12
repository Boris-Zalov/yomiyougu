<script>
  import '../app.css';
  import { invoke } from '@tauri-apps/api/core';
  import { platform } from '@tauri-apps/plugin-os';
  import { fade } from 'svelte/transition';
  import { page } from '$app/state';
  
  import SplashScreen from '../components/SplashScreen.svelte';
  import SetupWizard from '../components/SetupWizard.svelte';
  
  import DesktopNavigation from '../components/DesktopNavigation.svelte';
  import MobileNavigation from '../components/MobileNavigation.svelte';

  let { children } = $props();

  const currentPlatform = platform();
  const isMobile = currentPlatform === 'android' || currentPlatform === 'ios';

  let showSplash = $state(true);
  let showSetup = $state(false);
  let appReady = $state(false);

  let activePath = $derived(page.url.pathname);

  /**
   * @param {string} mode
   */
  function applyTheme(mode) {
    const html = document.documentElement;
    html.classList.remove('dark'); 

    if (mode === 'dark') {
      html.classList.add('dark');
    } else if (mode === 'system') {
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        html.classList.add('dark');
      }
    }
  }

  async function handleSplashComplete() {
    try {
      const exists = await invoke('check_config_exists');
      if (exists) {
        showSetup = false;
        appReady = true;

        const config = await invoke('get_config');
        applyTheme(config.theme_mode || 'system');
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
      const config = await invoke('get_config');
      applyTheme(config.theme_mode || 'system');
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
  <div
    class="min-h-screen w-screen transition-colors duration-300"
    transition:fade
  >
    {#if !isMobile}
      <DesktopNavigation {children} {activePath} />
    {:else}
      <div class="flex flex-col h-screen">
        <main class="grow overflow-y-auto pb-24 p-8">
          {@render children()}
        </main>
        <MobileNavigation />
      </div>
    {/if}
  </div>
{/if}

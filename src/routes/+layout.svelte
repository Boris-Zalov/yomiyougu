<script>
  import '../app.css';
  import SplashScreen from '../components/SplashScreen.svelte';

  let { children } = $props();
  let showSplash = $state(true);

  function handleSplashComplete() {
    showSplash = false;
  }
</script>

{#if showSplash}
  <SplashScreen onComplete={handleSplashComplete} />
{/if}

<main class="h-screen w-screen overflow-hidden overscroll-none bg-platinum">
  {@render children()}
</main>
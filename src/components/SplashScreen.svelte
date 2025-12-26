<script>
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';

  let { onComplete } = $props();

  const fullText = "読み用具"; 
  let visibleText = $state(""); 
  
  /**
   * Initialize device ID on first launch (or retrieve existing one)
   */
  async function initializeDeviceId() {
    try {
      const deviceId = await invoke('get_or_create_device_id');
      console.log('Device ID initialized:', deviceId);
    } catch (error) {
      console.error('Failed to initialize device ID:', error);
      // Non-fatal - continue with app load
    }
  }
  
  onMount(async () => {
    const deviceIdPromise = initializeDeviceId();
    
    let currentIndex = 0;
    const typingSpeed = 100; 
    const minDisplayTime = 1000; 
    const startTime = Date.now();

    const interval = setInterval(() => {
      visibleText += fullText[currentIndex];
      currentIndex++;

      if (currentIndex === fullText.length) {
        clearInterval(interval);
        
        const elapsedTime = Date.now() - startTime;
        const remainingTime = Math.max(0, minDisplayTime - elapsedTime);

        Promise.all([
          deviceIdPromise,
          new Promise(resolve => setTimeout(resolve, remainingTime + 500))
        ]).then(() => {
          onComplete();
        });
      }
    }, typingSpeed);
  });
</script>

<div 
  class="fixed inset-0 z-50 flex flex-col items-center justify-center touch-none overscroll-none bg-surface-dark"
  out:fade={{ duration: 500 }}
>
  <h1 class="text-6xl font-black tracking-widest text-primary-700 select-none">
    {visibleText}<span class="animate-pulse text-white">|</span>
  </h1>
</div>
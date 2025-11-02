<script>
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';

  let { onComplete } = $props();

  const fullText = "ヨミヨウグ"; 
  let visibleText = $state(""); 
  
  onMount(() => {
    let currentIndex = 0;
    const typingSpeed = 150; 
    const minDisplayTime = 1500; 
    const startTime = Date.now();

    const interval = setInterval(() => {
      visibleText += fullText[currentIndex];
      currentIndex++;

      if (currentIndex === fullText.length) {
        clearInterval(interval);
        
        const elapsedTime = Date.now() - startTime;
        const remainingTime = Math.max(0, minDisplayTime - elapsedTime);

        setTimeout(() => {
          onComplete();
        }, remainingTime + 500);
      }
    }, typingSpeed);
  });
</script>

<div 
  class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-platinum touch-none overscroll-none"
  out:fade={{ duration: 500 }}
>
  <h1 class="text-6xl font-black tracking-widest text-primary-700 select-none">
    {visibleText}<span class="animate-pulse text-shadow-grey">|</span>
  </h1>
</div>
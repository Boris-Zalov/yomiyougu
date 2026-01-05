<!-- Reader layout - navigation is hidden by root layout when on /reader/* routes -->
<script lang="ts">
	let { children } = $props();

	let isDarkMode = $derived(
		typeof document !== "undefined" && document.documentElement.classList.contains("dark")
	);
</script>

<div
	class="h-screen w-screen overflow-hidden transition-colors"
	class:bg-black={isDarkMode}
	class:bg-gray-100={!isDarkMode}
>
	{@render children()}
</div>

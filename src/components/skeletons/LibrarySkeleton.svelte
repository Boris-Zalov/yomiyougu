<!--
  LibrarySkeleton - Loading skeleton for library page
  Uses pure CSS animations without external dependencies for instant rendering
-->

<div class="page-container p-4">
	<!-- Header skeleton -->
	<div class="flex items-center justify-between mb-6">
		<div class="skeleton h-8 w-32 rounded"></div>
		<div class="skeleton h-10 w-24 rounded-lg"></div>
	</div>

	<!-- Collections section -->
	<div class="skeleton h-6 w-28 mb-4 rounded"></div>
	<div
		class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3 mb-8"
	>
		{#each Array(7) as _}
			<div class="skeleton aspect-2/3 rounded-lg"></div>
		{/each}
	</div>

	<!-- Divider -->
	<div class="skeleton h-px w-full my-8"></div>

	<!-- Books section -->
	<div class="skeleton h-6 w-28 mb-4 rounded"></div>
	<div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3">
		{#each Array(14) as _}
			<div class="skeleton aspect-2/3 rounded-lg"></div>
		{/each}
	</div>
</div>

<style>
	.skeleton {
		background: linear-gradient(
			90deg,
			rgb(229 231 235) 0%,
			rgb(243 244 246) 50%,
			rgb(229 231 235) 100%
		);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	:global(.dark) .skeleton {
		background: linear-gradient(90deg, rgb(55 65 81) 0%, rgb(75 85 99) 50%, rgb(55 65 81) 100%);
		background-size: 200% 100%;
	}

	@keyframes shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}
</style>

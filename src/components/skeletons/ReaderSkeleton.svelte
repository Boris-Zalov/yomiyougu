<!--
  ReaderSkeleton - Loading skeleton for reader page
-->

<div class="reader-skeleton-container h-screen w-screen flex items-center justify-center">
	<div class="skeleton w-full max-w-3xl aspect-2/3 mx-auto"></div>
</div>

<style>
	.reader-skeleton-container {
		background-color: rgb(243 244 246); /* gray-100 for light mode */
	}

	:global(.dark) .reader-skeleton-container {
		background-color: rgb(0 0 0); /* black for dark mode */
	}

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
		background: linear-gradient(90deg, rgb(31 41 55) 0%, rgb(55 65 81) 50%, rgb(31 41 55) 100%);
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

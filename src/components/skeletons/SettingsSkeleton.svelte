<!--
  SettingsSkeleton - Loading skeleton for settings page
  Uses pure CSS animations without external dependencies for instant rendering
-->

<div class="page-container p-4">
  <div class="skeleton h-8 w-32 mb-6 rounded"></div>

  <div class="space-y-6">
    {#each Array(4) as _}
      <div class="skeleton-card">
        <div class="skeleton h-6 w-40 mb-4 rounded"></div>
        <div class="space-y-4">
          {#each Array(3) as __}
            <div class="flex items-center justify-between py-2">
              <div class="flex-1">
                <div class="skeleton h-5 w-48 mb-2 rounded"></div>
                <div class="skeleton h-4 w-64 rounded"></div>
              </div>
              <div class="skeleton w-16 h-8 rounded-lg"></div>
            </div>
          {/each}
        </div>
      </div>
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

  .skeleton-card {
    background: white;
    border: 1px solid rgb(229 231 235);
    border-radius: 0.5rem;
    padding: 1.25rem;
  }

  :global(.dark) .skeleton {
    background: linear-gradient(
      90deg,
      rgb(55 65 81) 0%,
      rgb(75 85 99) 50%,
      rgb(55 65 81) 100%
    );
    background-size: 200% 100%;
  }

  :global(.dark) .skeleton-card {
    background: rgb(31 41 55);
    border-color: rgb(55 65 81);
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

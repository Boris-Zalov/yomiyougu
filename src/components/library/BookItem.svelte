<script lang="ts">
    import type { Book } from '$lib/types/library';
    import { getCoverPath, calculateProgress } from '$lib/types/library';
    import { Progressbar, Badge } from 'flowbite-svelte';
    import { StarSolid, ClockOutline, ImageOutline } from 'flowbite-svelte-icons';

    let { book, onclick }: { book: Book; onclick?: () => void } = $props();

    let imageLoadFailed = $state(false);

    function formatStatus(status: string): string {
        return status.replace('_', ' ').replace(/\b\w/g, (l) => l.toUpperCase());
    }

    function getStatusColor(
        status: string
    ): 'gray' | 'blue' | 'green' | 'yellow' | 'red' | 'indigo' | 'purple' | 'pink' {
        switch (status) {
            case 'reading': return 'blue';
            case 'completed': return 'green';
            case 'on_hold': return 'yellow';
            case 'dropped': return 'red';
            default: return 'gray';
        }
    }

    function formatLastRead(dateStr: string | null): string {
        if (!dateStr) return 'Never';
        const date = new Date(dateStr);
        const now = new Date();
        const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

        if (diffDays === 0) return 'Today';
        if (diffDays === 1) return 'Yesterday';
        if (diffDays < 7) return `${diffDays}d ago`;
        if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`;
        return date.toLocaleDateString();
    }

    const progress = $derived(calculateProgress(book));
    const coverPath = $derived(getCoverPath(book.id));
</script>

<button 
    type="button" 
    class="book-item group relative w-full h-auto aspect-2/3 overflow-hidden rounded-lg shadow-sm hover:shadow-md border border-gray-200 dark:border-gray-700 bg-gray-200 dark:bg-gray-700" 
    {onclick}
>
    <div class="absolute inset-0 w-full h-full">
        {#if imageLoadFailed}
            <div class="flex items-center justify-center w-full h-full bg-gray-100 dark:bg-gray-800 text-gray-400 dark:text-gray-500">
                <ImageOutline class="w-10 h-10" />
            </div>
        {:else}
            <img 
                src={coverPath} 
                alt={`${book.title} cover`} 
                class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105" 
                loading="lazy" 
                onerror={() => imageLoadFailed = true}
            />
        {/if}
    </div>

    {#if book.is_favorite}
        <div class="favorite-badge">
            <StarSolid class="w-3.5 h-3.5" />
        </div>
    {/if}

    <div class="absolute bottom-0 left-0 right-0 z-10 flex flex-col justify-end
                p-2 sm:p-3
                bg-white/90 dark:bg-gray-800/90 
                backdrop-blur-md
                border-t border-gray-100 dark:border-gray-700">
        
        <h5 class="text-xs sm:text-sm font-bold text-gray-900 dark:text-white line-clamp-2 leading-tight mb-2 text-left" title={book.title}>
            {book.title}
        </h5>

        <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
                <Badge color={getStatusColor(book.reading_status)} class="px-1.5 py-0.5 text-[10px] border-0">
                    {formatStatus(book.reading_status)}
                </Badge>
                
                {#if book.total_pages > 0}
                    <span class="text-[10px] font-medium text-gray-600 dark:text-gray-300">
                        {book.current_page}/{book.total_pages}
                    </span>
                {/if}
            </div>

            <div class="flex items-center gap-1 text-[10px] text-gray-500 dark:text-gray-400">
                <ClockOutline class="w-3 h-3 shrink-0" />
                <span class="truncate">{formatLastRead(book.last_read_at)}</span>
            </div>
        </div>
    </div>

    {#if book.current_page > 0}
        <div class="absolute bottom-0 left-0 right-0 z-20">
            <Progressbar {progress} size="h-1" color="blue" labelInside={false} class="rounded-none" />
        </div>
    {/if}
</button>

<style>
    .book-item {
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        isolation: isolate;
    }
    
    @media (hover: hover) {
        .book-item:hover {
            transform: translateY(-2px);
        }
    }

    .book-item:focus-visible {
        outline: none;
        box-shadow: 0 0 0 2px var(--color-primary-500, #3b82f6);
    }

    .favorite-badge {
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        background-color: rgba(250, 204, 21, 0.95);
        color: white;
        border-radius: 9999px;
        padding: 0.3rem;
        box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        z-index: 20;
    }
</style>
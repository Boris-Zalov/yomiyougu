<script lang="ts">
    import type { CollectionWithCount } from '$lib/types/library';
    import { Dropdown, DropdownItem, DropdownDivider } from 'flowbite-svelte';
    import { DotsVerticalOutline, EditOutline, TrashBinOutline } from 'flowbite-svelte-icons';

    let { 
        collection, 
        ondelete 
    }: { 
        collection: CollectionWithCount;
        ondelete?: (collection: CollectionWithCount) => void;
    } = $props();

    // Unique ID for dropdown trigger to avoid conflicts between multiple items
    let dropdownId = $derived(`collection-menu-${collection.id}`);

    function handleDelete(e: Event) {
        e.preventDefault();
        e.stopPropagation();
        ondelete?.(collection);
    }

    function handleEdit(e: Event) {
        e.preventDefault();
        e.stopPropagation();
        // Navigate will happen via the link
    }
</script>

<div class="relative group">
    <a href={`/library/collections/${collection.id}`} class="block">
        <div
            class="w-full aspect-2/3 rounded-lg bg-linear-to-br from-primary-500 to-primary-700 dark:from-primary-600 dark:to-primary-800 flex items-center justify-center cursor-pointer hover:opacity-90 transition-opacity"
        >
            <span class="text-white text-center px-2 font-medium text-sm line-clamp-3">
                {collection.name}
            </span>
        </div>
        <div
            class="absolute bottom-0 left-0 right-0 bg-black/60 text-white text-xs text-center py-1 rounded-b-lg"
        >
            {collection.book_count}
            {collection.book_count === 1 ? "book" : "books"}
        </div>
    </a>
    
    <!-- Dropdown menu button - always visible on touch, hover on desktop -->
    <button
        type="button"
        class="{dropdownId} dropdown-trigger absolute top-1 right-1 p-1.5 rounded-full bg-black/40 hover:bg-black/60 text-white transition-opacity z-10"
        onclick={(e) => e.preventDefault()}
    >
        <DotsVerticalOutline class="w-4 h-4" />
    </button>
    
    <Dropdown simple triggeredBy=".{dropdownId}" class="w-40 z-50">
        <DropdownItem 
            href={`/library/collections/${collection.id}/edit`}
            class="flex items-center gap-2"
        >
            <EditOutline class="w-4 h-4" />
            Edit
        </DropdownItem>
        <DropdownDivider />
        <DropdownItem 
            onclick={handleDelete}
            class="flex items-center gap-2 text-red-600 dark:text-red-500"
        >
            <TrashBinOutline class="w-4 h-4" />
            Delete
        </DropdownItem>
    </Dropdown>
</div>

<style>
    /* Show dropdown trigger on hover for desktop, always visible on touch */
    .dropdown-trigger {
        opacity: 1;
    }

    @media (hover: hover) {
        .dropdown-trigger {
            opacity: 0;
        }
        .group:hover .dropdown-trigger,
        .dropdown-trigger:focus {
            opacity: 1;
        }
    }
</style>

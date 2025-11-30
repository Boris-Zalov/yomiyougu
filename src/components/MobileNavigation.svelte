<script>
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import {
        GridSolid,
        AdjustmentsVerticalSolid,
        HomeSolid,
    } from "flowbite-svelte-icons";

    const navItems = [
        { label: "Library", href: "/library", icon: GridSolid },
        { label: "Home", href: "/dashboard", icon: HomeSolid },
        {
            label: "Settings",
            href: "/settings",
            icon: AdjustmentsVerticalSolid,
        },
    ];

    let activeUrl = $derived(page.url.pathname);

    const containerClass = "fixed bottom-0 left-0 z-50 w-full h-16 bg-white border-t border-gray-200 dark:bg-gray-700 dark:border-gray-600";
    const gridClass = "grid h-full max-w-lg grid-cols-3 mx-auto font-medium";
    
    const itemClass = "inline-flex flex-col items-center justify-center px-5 hover:bg-gray-50 dark:hover:bg-gray-800 group";
</script>

<div class={containerClass}>
    <div class={gridClass}>
        {#each navItems as item}
            {@const isActive = activeUrl === item.href}
            
            <a
                href={item.href}
                class={itemClass}
                onclick={(e) => {
                    e.preventDefault();
                    goto(item.href);
                }}
            >
                <item.icon 
                    class={`w-5 h-5 mb-1 group-hover:text-primary-600 dark:group-hover:text-primary-500 ${
                        isActive 
                        ? "text-primary-600 dark:text-primary-500" 
                        : "text-gray-500 dark:text-gray-400"
                    }`} 
                />
                <span 
                    class={`text-sm group-hover:text-primary-600 dark:group-hover:text-primary-500 ${
                        isActive 
                        ? "text-primary-600 dark:text-primary-500" 
                        : "text-gray-500 dark:text-gray-400"
                    }`}
                >
                    {item.label}
                </span>
            </a>
        {/each}
    </div>
</div>
<script>
    import { goto } from "$app/navigation";
    import { Tabs, TabItem } from "flowbite-svelte";
    import {
        GridSolid,
        AdjustmentsVerticalSolid,
        BookOpenOutline,
    } from "flowbite-svelte-icons";
    let { children, activePath } = $props();

    const navItems = [
        { label: "Dashboard", href: "/dashboard", icon: GridSolid },
        { label: "Library", href: "/library", icon: BookOpenOutline },
        {
            label: "Settings",
            href: "/settings",
            icon: AdjustmentsVerticalSolid,
        },
    ];
</script>

<Tabs tabStyle="underline">
    {#each navItems as item}
        {@const isActive = activePath.startsWith(item.href)}

        <TabItem
            open={isActive}
            onclick={(e) => {
                e.preventDefault();
                goto(item.href);
            }}
        >
            {#snippet titleSlot()}
                <div
                    class="flex items-center gap-2 cursor-pointer w-full h-full"
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => e.key === "Enter" && goto(item.href)}
                >
                    <item.icon size="md" />
                    {item.label}
                </div>
            {/snippet}

            {#if isActive}
                <div>
                    {@render children()}
                </div>
            {:else}{/if}
        </TabItem>
    {/each}
</Tabs>

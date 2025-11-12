<script>
    import { page } from "$app/state";
    import { BottomNav, BottomNavItem } from "flowbite-svelte";
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
</script>

<BottomNav
    {activeUrl}
    position="absolute"
    navType="border"
    classes={{ inner: "grid-cols-3" }}
>
    {#each navItems as item}
        <BottomNavItem btnName={item.label} href={item.href}>
            <item.icon />
        </BottomNavItem>
    {/each}
</BottomNav>

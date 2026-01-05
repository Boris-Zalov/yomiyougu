<script>
	import { goto } from "$app/navigation";
	import { GridSolid, AdjustmentsVerticalSolid, BookOpenOutline } from "flowbite-svelte-icons";

	let { activePath } = $props();

	const navItems = [
		{ label: "Dashboard", href: "/dashboard", icon: GridSolid },
		{ label: "Library", href: "/library", icon: BookOpenOutline },
		{ label: "Settings", href: "/settings", icon: AdjustmentsVerticalSolid },
	];

	const ulClass =
		"flex flex-wrap -mb-px text-sm font-medium text-center text-gray-500 dark:text-gray-400 border-b border-gray-200 dark:border-gray-700";

	const activeClass =
		"inline-flex items-center justify-center p-4 text-primary-600 border-b-2 border-primary-600 rounded-t-lg active group dark:text-primary-500 dark:border-primary-500";
	const inactiveClass =
		"inline-flex items-center justify-center p-4 border-b-2 border-transparent rounded-t-lg hover:text-gray-600 hover:border-gray-300 dark:hover:text-gray-300 group";
</script>

<ul class={ulClass}>
	{#each navItems as item}
		{@const isActive = activePath.startsWith(item.href)}
		{@const Icon = item.icon}

		<li class="me-2">
			<a
				href={item.href}
				class={isActive ? activeClass : inactiveClass}
				data-sveltekit-preload-code="eager"
				onclick={(e) => {
					e.preventDefault();
					goto(item.href);
				}}
			>
				<Icon
					class={`w-4 h-4 me-2 ${isActive ? "text-primary-600 dark:text-primary-500" : "text-gray-400 group-hover:text-gray-500 dark:text-gray-500 dark:group-hover:text-gray-300"}`}
				/>
				{item.label}
			</a>
		</li>
	{/each}
</ul>

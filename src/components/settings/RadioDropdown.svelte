<!--
  RadioDropdown - Reusable radio button dropdown selector
  Used for selecting options with descriptions
-->
<script lang="ts">
	import { Button, Dropdown, DropdownItem, Radio, Helper } from "flowbite-svelte";
	import { ChevronDownOutline } from "flowbite-svelte-icons";

	interface Option {
		value: string;
		label: string;
		description?: string;
	}

	interface Props {
		options: Option[];
		value: string | null;
		onchange?: (value: string) => void;
		displayValue?: string;
	}

	let { options, value = $bindable(), onchange, displayValue }: Props = $props();

	// Unique ID for dropdown trigger to avoid conflicts between multiple instances
	const dropdownId = crypto.randomUUID();

	// Find the current option to display its label
	let currentLabel = $derived(
		displayValue ??
			(value ? options.find((opt) => opt.value === value)?.label : null) ??
			value ??
			"Select..."
	);

	function handleChange(newValue: string) {
		value = newValue;
		onchange?.(newValue);
	}
</script>

<div class="inline-block relative">
	<Button color="alternative" class="justify-between dropdown-{dropdownId}">
		{currentLabel}
		<ChevronDownOutline class="ms-2 h-4 w-4" />
	</Button>
	<Dropdown class="list-none" triggeredBy=".dropdown-{dropdownId}" placement="bottom-start">
		{#each options as option (option.value)}
			<DropdownItem class="list-none">
				<Radio
					name={crypto.randomUUID()}
					bind:group={value}
					value={option.value}
					onchange={() => handleChange(option.value)}
				>
					<div class="flex flex-col">
						{option.label}
						{#if option.description}
							<Helper class="ps-6">{option.description}</Helper>
						{/if}
					</div>
				</Radio>
			</DropdownItem>
		{/each}
	</Dropdown>
</div>

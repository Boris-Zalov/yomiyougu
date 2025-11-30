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
    value: string;
    onchange?: (value: string) => void;
    displayValue?: string;
  }

  let { options, value = $bindable(), onchange, displayValue }: Props = $props();

  // Find the current option to display its label
  let currentLabel = $derived(
    displayValue ?? options.find((opt) => opt.value === value)?.label ?? value
  );

  function handleChange(newValue: string) {
    value = newValue;
    onchange?.(newValue);
  }
</script>

<div class="space-y-0 relative">
  <Button color="alternative" class="w-full justify-between">
    {currentLabel}
    <ChevronDownOutline class="ms-2 h-4 w-4" />
  </Button>
  <Dropdown class="list-none" placement="bottom" strategy="absolute">
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

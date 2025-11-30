<!--
  SettingWidget - Renders the appropriate widget based on setting type
  Handles: Toggle, Select, Input, Slider, Color
-->
<script lang="ts">
  import { Toggle, Input, Range } from "flowbite-svelte";
  import type { SettingItem, SettingValue } from "$lib/types/settings";
  import RadioDropdown from "./RadioDropdown.svelte";

  interface Props {
    setting: SettingItem;
    onchange?: (key: string, value: SettingValue) => void;
  }

  let { setting, onchange }: Props = $props();

  function handleToggle(checked: boolean) {
    onchange?.(setting.key, checked);
  }

  function handleSelect(value: string) {
    onchange?.(setting.key, value);
  }

  function handleInput(value: string) {
    onchange?.(setting.key, value);
  }

  function handleSlider(value: number) {
    onchange?.(setting.key, value);
  }

  // Calculate slider progress percentage for styling
  function getSliderProgress(value: number, min: number, max: number): number {
    return ((value - min) / (max - min)) * 100;
  }
</script>

{#if setting.widget.type === "toggle"}
  <Toggle
    checked={setting.value as boolean}
    onchange={(e) => handleToggle((e.target as HTMLInputElement).checked)}
  />
{:else if setting.widget.type === "select"}
  {@const options = setting.widget.options.map((opt) => ({
    value: opt.value,
    label: opt.label,
  }))}
  <RadioDropdown
    {options}
    value={setting.value as string}
    onchange={(value) => handleSelect(value)}
  />
{:else if setting.widget.type === "input"}
  <Input
    type="text"
    value={setting.value as string}
    oninput={(e) => handleInput((e.target as HTMLInputElement).value)}
  />
{:else if setting.widget.type === "slider"}
  {@const { min, max, step } = setting.widget}
  {@const progress = getSliderProgress(setting.value as number, min, max)}
  <div class="flex items-center gap-4">
    <Range
      {min}
      {max}
      {step}
      bind:value={setting.value as number}
      onchange={(e) => handleSlider(Number((e.target as HTMLInputElement).value))}
      class="flex-1"
      style="--range-progress: {progress}%"
    />
    <span class="text-sm font-medium w-12 text-right">{setting.value}</span>
  </div>
{:else if setting.widget.type === "color"}
  <input
    type="color"
    value={setting.value as string}
    oninput={(e) => handleInput((e.target as HTMLInputElement).value)}
    class="w-12 h-8 rounded cursor-pointer"
  />
{/if}

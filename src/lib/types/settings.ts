/**
 * TypeScript types matching the Rust settings API
 * These mirror the structures in src-tauri/src/settings/types.rs
 */

// Setting value types (matches Rust SettingValue enum)
export type SettingValue = boolean | string | number;

// Widget types for UI rendering
export interface SelectOption {
  value: string;
  label: string;
  description?: string;
}

export type WidgetType =
  | { type: "toggle" }
  | { type: "input" }
  | { type: "select"; options: SelectOption[] }
  | { type: "slider"; min: number; max: number; step: number }
  | { type: "color" };

// Individual setting with metadata
export interface SettingItem {
  key: string;
  label: string;
  description: string;
  widget: WidgetType;
  value: SettingValue;
  defaultValue: SettingValue;
  requiresRestart?: boolean;
  platforms?: string[];
}

// Category grouping related settings
export interface SettingCategory {
  id: string;
  label: string;
  description: string;
  icon?: string;
  settings: SettingItem[];
}

// Complete settings structure
export interface AppSettings {
  version: number;
  setupCompleted: boolean;
  acceptedLicense: boolean;
  categories: SettingCategory[];
}

// Helper type for setting updates
export type SettingUpdates = Record<string, SettingValue>;

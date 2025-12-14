/**
 * Settings API service
 * Centralized Tauri command invocations for settings management
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  SettingCategory,
  SettingValue,
  SettingUpdates,
} from "$lib/types/settings";

/**
 * Check if settings file exists (for first-run detection)
 */
export async function checkSettingsExists(): Promise<boolean> {
  return invoke<boolean>("check_settings_exists");
}

/**
 * Get complete settings with schema
 */
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

/**
 * Get settings organized by category (for settings page)
 */
export async function getSettingsSchema(): Promise<SettingCategory[]> {
  return invoke<SettingCategory[]>("get_settings_schema");
}

/**
 * Get a single setting value by key
 */
export async function getSetting(key: string): Promise<SettingValue | null> {
  return invoke<SettingValue | null>("get_setting", { key });
}

/**
 * Update multiple settings at once
 */
export async function saveSettings(
  formData: SettingUpdates
): Promise<AppSettings> {
  return invoke<AppSettings>("save_settings_from_schema", { formData });
}

/**
 * Update a single setting
 */
export async function updateSetting(
  key: string,
  value: SettingValue
): Promise<AppSettings> {
  return invoke<AppSettings>("update_setting", { key, value });
}

/**
 * Complete initial setup wizard
 */
export async function completeSetup(
  initialSettings?: SettingUpdates
): Promise<void> {
  return invoke<void>("complete_setup", { initialSettings });
}

/**
 * Reset all settings to defaults
 */
export async function resetAllSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("reset_all_settings");
}

/**
 * Reset a specific setting to its default
 */
export async function resetSetting(key: string): Promise<AppSettings> {
  return invoke<AppSettings>("reset_setting", { key });
}

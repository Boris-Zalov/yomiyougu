/**
 * Theme utilities
 * Centralized theme management for consistent theming across the app
 */

export type ThemeMode = "light" | "dark" | "system";

/**
 * Apply theme to the document
 */
export function applyTheme(mode: ThemeMode): void {
	const html = document.documentElement;
	html.classList.remove("dark");

	if (mode === "dark") {
		html.classList.add("dark");
	} else if (mode === "system") {
		if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
			html.classList.add("dark");
		}
	}
}

/**
 * Get current effective theme (resolves "system" to actual theme)
 */
export function getEffectiveTheme(mode: ThemeMode): "light" | "dark" {
	if (mode === "system") {
		return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
	}
	return mode;
}

/**
 * Listen for system theme changes
 */
export function onSystemThemeChange(callback: (isDark: boolean) => void): () => void {
	const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
	const handler = (e: MediaQueryListEvent) => callback(e.matches);
	mediaQuery.addEventListener("change", handler);
	return () => mediaQuery.removeEventListener("change", handler);
}

/**
 * Capitalize first letter of a string
 */
export function capitalize(str: string): string {
	return str ? str.charAt(0).toUpperCase() + str.slice(1) : "";
}

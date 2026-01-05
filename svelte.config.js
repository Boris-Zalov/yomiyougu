// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Disable certain a11y warnings for the reader component
		warningFilter: (warning) => {
			if (
				warning.code === "a11y_click_events_have_key_events" &&
				warning.filename?.includes("/reader/")
			) {
				return false;
			}
			if (
				warning.code === "a11y_no_noninteractive_element_interactions" &&
				warning.filename?.includes("/reader/")
			) {
				return false;
			}
			return true;
		},
	},
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			fallback: "index.html",
		}),
		alias: {
			$components: "src/components",
			"$components/*": "src/components/*",
			$skeletons: "src/components/skeletons",
			"$skeletons/*": "src/components/skeletons/*",
		},
	},
};

export default config;

import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from '@tailwindcss/vite';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    tailwindcss(), 
    sveltekit()
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // Production build optimizations
  build: {
    // Minify for production
    minify: 'esbuild',
    // Target modern browsers for smaller bundle
    target: 'esnext',
    // Increase chunk size warning limit (Tauri bundles everything)
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        // Optimize chunk splitting
        manualChunks: (/** @type {string} */ id) => {
          if (id.includes('node_modules')) {
            // Group large dependencies
            if (id.includes('flowbite')) return 'vendor-flowbite';
            if (id.includes('fuse.js')) return 'vendor-fuse';
            return 'vendor';
          }
        },
      },
    },
    // Enable source maps only in dev
    sourcemap: false,
  },

  // Optimize dependencies
  optimizeDeps: {
    include: [
      'fuse.js', 
      'flowbite-svelte',
      'flowbite-svelte-icons',
      '@tauri-apps/api/core',
      '@tauri-apps/plugin-dialog',
      '@tauri-apps/plugin-os',
    ],
  },
});

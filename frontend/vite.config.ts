import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	build: {
		// Excalidraw ships a ~1.8 MB WASM blob (rough.js/emscripten) that cannot be split further.
		// For a local-first app this is cached after first load and irrelevant to perf.
		chunkSizeWarningLimit: 2000,
	},
});

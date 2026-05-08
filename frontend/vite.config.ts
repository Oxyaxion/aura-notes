import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { execSync } from 'child_process';

export default defineConfig(({ command }) => {
	const gitVersion = (() => {
		try {
			return execSync('git describe --tags', { encoding: 'utf-8' }).trim();
		} catch {
			return 'dev';
		}
	})();

	const appVersion = command === 'serve' ? `${gitVersion}-dev` : gitVersion;

	return {
		define: {
			__APP_VERSION__: JSON.stringify(appVersion),
		},
		plugins: [sveltekit()],
		build: {
			// Excalidraw ships a ~1.8 MB WASM blob (rough.js/emscripten) that cannot be split further.
			// For a local-first app this is cached after first load and irrelevant to perf.
			chunkSizeWarningLimit: 2000,
		},
	};
});

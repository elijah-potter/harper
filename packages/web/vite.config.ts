import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	server: {
		port: 3000,
		fs: {
			allow: ['../harper.js/dist']
		}
	},
	plugins: [sveltekit(), wasm(), topLevelAwait()]
});

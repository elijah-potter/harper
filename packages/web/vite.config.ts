import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	server: {
		port: 3000,
		proxy: {
			'/parse': 'http://localhost:3001',
			'/lint': 'http://localhost:3001',
			'/apply': 'http://localhost:3001'
		}
	},
	plugins: [sveltekit(), wasm(), topLevelAwait()]
});

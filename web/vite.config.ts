import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	server: {
		port: 3000,
		proxy: {
			'/parse': 'http://localhost:3001'
		}
	},
	plugins: [sveltekit()]
});

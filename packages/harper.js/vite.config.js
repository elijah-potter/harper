import { resolve } from 'path';
import dts from 'vite-plugin-dts';
import { defineConfig } from 'vite';

export default defineConfig({
	build: {
		lib: {
			entry: resolve(__dirname, 'src/main.ts'),
			fileName: `harper`,
			name: 'harper',
			formats: ['es']
		}
	},
	base: './',
	plugins: [dts({ rollupTypes: true, tsconfigPath: './tsconfig.json' })],
	worker: {
		plugins: [],
		format: 'es'
	},
	server: {
		fs: {
			allow: ['../../harper-wasm/pkg']
		}
	},
	test: {
		browser: {
			provider: 'playwright',
			enabled: true,
			name: 'chromium'
		}
	}
});

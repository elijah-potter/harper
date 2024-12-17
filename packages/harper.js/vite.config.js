import { resolve } from 'path';
import dts from 'vite-plugin-dts';
import { defineConfig } from 'vite';
import virtual from 'vite-plugin-virtual';

export default defineConfig({
	build: {
		lib: {
			entry: resolve(__dirname, 'src/main.ts'),
			fileName: `harper`,
			name: 'harper',
			formats: ['es']
		},
		rollupOptions: {
			output: {
				inlineDynamicImports: true
			}
		}
	},
	base: './',
	plugins: [
		dts({ rollupTypes: true, tsconfigPath: './tsconfig.json' }),
		virtual({
			'virtual:wasm': `import wasmUri from 'wasm/harper_wasm_bg.wasm?inline'; export default wasmUri`
		})
	],
	worker: {
		plugins: [
			virtual({
				'virtual:wasm': `export default ''`
			})
		],
		format: 'es',
		rollupOptions: {
			output: {
				inlineDynamicImports: true
			}
		}
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
	},
	assetsInclude: ['**/*.wasm']
});

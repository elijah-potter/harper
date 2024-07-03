import { wasm } from '@rollup/plugin-wasm';
import typescript from '@rollup/plugin-typescript';
import external from 'rollup-plugin-peer-deps-external';
import { nodeResolve } from '@rollup/plugin-node-resolve';

export default {
	input: 'src/index.js',
	output: {
		file: 'main.js',
		format: 'cjs'
	},
	external: ['obsidian', 'electron'],
	plugins: [
		external(),
		wasm({ maxFileSize: Math.pow(2, 32), publicPath: './' }),
		nodeResolve(),
		typescript({ compilerOptions: { lib: ['es5', 'es6', 'dom'], target: 'es5' } })
	]
};

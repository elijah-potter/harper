import wasmUrl from 'wasm/harper_wasm_bg.wasm?url';

/** Load the WebAssembly manually and dynamically, making sure to setup infrastructure. */
export default async function loadWasm() {
	const wasm = await import('wasm');
	await wasm.default(wasmUrl);

	return wasm;
}

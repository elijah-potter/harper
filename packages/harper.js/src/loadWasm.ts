/** Load the WebAssembly manually and dynamically, making sure to setup infrastructure. */
export default async function loadWasm() {
	const wasm = await import('wasm');
	await wasm.default();

	return wasm;
}

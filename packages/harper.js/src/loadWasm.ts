// @ts-ignore
import wasmUri from 'virtual:wasm';

let curWasmUri = wasmUri;

/** Get the currently set data URI for the WebAssembly module. */
export function getWasmUri(): string {
	return curWasmUri;
}

/** Set the data URI for the WebAssembly module. */
export function setWasmUri(uri: string) {
	curWasmUri = uri;
}

/** Load the WebAssembly manually and dynamically, making sure to setup infrastructure.
 * You can use an optional data URL for the WebAssembly file if the module is being loaded from a Web Worker.
 * */
export default async function loadWasm() {
	const wasm = await import('wasm');
	await wasm.default(getWasmUri());

	return wasm;
}

// @ts-expect-error because this virtual module hasn't been added to a `d.ts` file.
import wasmUri from 'virtual:wasm';

let curWasmUri = wasmUri;

/** Get the currently set data URI for the WebAssembly module.
 * I'm not a huge of the singleton, but we're swapping out same data, just from a different source, so the state doesn't meaningfully change. */
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
	// @ts-ignore
	await wasm.default(getWasmUri());

	return wasm;
}

import { setWasmUri } from '../loadWasm';
import LocalLinter from '../LocalLinter';
import { deserialize, serializeArg } from './communication';

const linter = new LocalLinter();

/** @param {SerializedRequest} v  */
async function processRequest(v) {
	const { procName, args } = await deserialize(v);

	let res = await linter[procName](...args);
	postMessage(await serializeArg(res));
}

self.onmessage = function (e) {
	setWasmUri(e.data);

	self.onmessage = function (e) {
		processRequest(e.data);
	};
};

// Notify the main thread that we are ready
postMessage('ready');

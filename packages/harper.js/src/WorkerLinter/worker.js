import LocalLinter from '../LocalLinter';
import { deserialize, serializeArg } from './communication';

const linter = new LocalLinter();

console.log('Linter created');

self.onmessage = function (e) {
	const { procName, args } = deserialize(e.data);

	linter[procName](...args).then((res) => postMessage(serializeArg(res)));
};

// Notify the main thread that we are ready
postMessage('ready');

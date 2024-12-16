import { DeserializedRequest, deserializeArg, serialize } from './communication';
import { Lint, Suggestion, Span } from 'wasm';
import Linter from '../Linter';
import Worker from './worker.js?worker';

/** The data necessary to complete a request once the worker has responded. */
type RequestItem = {
	resolve: (item: unknown) => void;
	reject: (item: unknown) => void;
	request: DeserializedRequest;
};

/** A Linter that spins up a dedicated web worker to do processing on a separate thread.
 * Main benefit: this Linter will not block the event loop for large documents.
 *
 * NOTE: This class will not work properly in Node. In that case, just use `LocalLinter`. */
export default class WorkerLinter implements Linter {
	private worker;
	private requestQueue: RequestItem[];
	private working = true;

	constructor() {
		this.worker = new Worker();
		this.requestQueue = [];

		this.worker.onmessage = () => {
			this.setupMainEventListeners();
			this.working = false;
			this.submitRemainingRequests();
		};
	}

	private setupMainEventListeners() {
		this.worker.onmessage = (e: MessageEvent) => {
			const { resolve } = this.requestQueue.shift()!;
			resolve(deserializeArg(e.data));
			this.working = false;

			this.submitRemainingRequests();
		};

		this.worker.onmessageerror = (e: MessageEvent) => {
			const { reject } = this.requestQueue.shift()!;
			reject(e.data);
			this.working = false;

			this.submitRemainingRequests();
		};
	}

	async setup(): Promise<void> {
		return await this.rpc('setup', []);
	}

	async lint(text: string): Promise<Lint[]> {
		return await this.rpc('lint', [text]);
	}
	async applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		return await this.rpc('applySuggestion', [text, suggestion, span]);
	}
	async isLikelyEnglish(text: string): Promise<boolean> {
		return await this.rpc('isLikelyEnglish', [text]);
	}

	async isolateEnglish(text: string): Promise<string> {
		return await this.rpc('isolateEnglish', [text]);
	}

	/** Run a procedure on the remote worker. */
	private async rpc(procName: string, args: any[]): Promise<any> {
		const promise = new Promise((resolve, reject) => {
			this.requestQueue.push({
				resolve,
				reject,
				request: { procName, args }
			});

			this.submitRemainingRequests();
		});

		return promise;
	}

	private submitRemainingRequests() {
		if (this.working) {
			return;
		}

		this.working = true;

		if (this.requestQueue.length > 0) {
			const { request } = this.requestQueue[0];

			this.worker.postMessage(serialize(request));
		} else {
			this.working = false;
		}
	}
}

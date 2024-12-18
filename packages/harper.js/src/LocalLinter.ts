import type { Lint, Span, Suggestion, Linter as WasmLinter } from 'wasm';
import Linter from './Linter';
import loadWasm from './loadWasm';

/** A Linter that runs in the current JavaScript context (meaning it is allowed to block the event loop). */
export default class LocalLinter implements Linter {
	private inner: WasmLinter | undefined;

	async setup(): Promise<void> {
		await this.initialize();
		this.inner!.lint('');
	}

	async lint(text: string): Promise<Lint[]> {
		await this.initialize();
		let lints = this.inner!.lint(text);

		// We only want to show fixable errors.
		lints = lints.filter((lint) => lint.suggestion_count() > 0);

		return lints;
	}

	async applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		const wasm = await loadWasm();
		return wasm.apply_suggestion(text, span, suggestion);
	}

	async isLikelyEnglish(text: string): Promise<boolean> {
		await this.initialize();
		return this.inner!.is_likely_english(text);
	}

	async isolateEnglish(text: string): Promise<string> {
		await this.initialize();
		return this.inner!.isolate_english(text);
	}

	/// Initialize the WebAssembly and construct the inner Linter.
	private async initialize(): Promise<void> {
		const wasm = await loadWasm();
		wasm.setup();
		this.inner = wasm.Linter.new();
	}
}

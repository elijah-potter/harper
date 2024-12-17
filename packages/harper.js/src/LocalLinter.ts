import type { Lint, Span, Suggestion } from 'wasm';
import Linter from './Linter';
import loadWasm from './loadWasm';

/** A Linter that runs in the current JavaScript context (meaning it is allowed to block the event loop). */
export default class LocalLinter implements Linter {
	async setup(): Promise<void> {
		const wasm = await loadWasm();
		wasm.setup();
		wasm.lint('');
	}

	async lint(text: string): Promise<Lint[]> {
		const wasm = await loadWasm();
		let lints = wasm.lint(text);

		// We only want to show fixable errors.
		lints = lints.filter((lint) => lint.suggestion_count() > 0);

		return lints;
	}

	async applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		const wasm = await loadWasm();
		return wasm.apply_suggestion(text, span, suggestion);
	}

	async isLikelyEnglish(text: string): Promise<boolean> {
		const wasm = await loadWasm();
		return wasm.is_likely_english(text);
	}

	async isolateEnglish(text: string): Promise<string> {
		const wasm = await loadWasm();
		return wasm.isolate_english(text);
	}
}

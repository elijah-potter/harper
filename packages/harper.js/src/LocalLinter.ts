import type { Lint, Span, Suggestion } from 'wasm';
import Linter from './Linter';

/** A Linter that runs in the current JavaScript context (meaning it is allowed to block the event loop). */
export default class LocalLinter implements Linter {
	async setup(): Promise<void> {
		const wasm = await import('wasm');
		wasm.setup();
		wasm.lint('');
	}

	async lint(text: string): Promise<Lint[]> {
		const wasm = await import('wasm');
		let lints = wasm.lint(text);

		// We only want to show fixable errors.
		lints = lints.filter((lint) => lint.suggestion_count() > 0);

		return lints;
	}

	async applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		const wasm = await import('wasm');
		return wasm.apply_suggestion(text, span, suggestion);
	}

	async isLikelyEnglish(text: string): Promise<boolean> {
		const wasm = await import('wasm');
		return wasm.is_likely_english(text);
	}

	async isolateEnglish(text: string): Promise<string> {
		const wasm = await import('wasm');
		return wasm.isolate_english(text);
	}
}

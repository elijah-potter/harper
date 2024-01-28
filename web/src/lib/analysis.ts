const defaultUseWasm = true;

export interface ParseResponse {
	tokens: Token[];
}

export interface Token {
	content: string[];
	kind: TokenKind;
}

export type TokenKind =
	| { kind: 'Word' }
	| { kind: 'Punctuation'; value: Punctuation }
	| { kind: 'Number'; value: number }
	| { kind: 'Space'; value: number }
	| { kind: 'Newline'; value: number };

export type Punctuation =
	| 'Period'
	| 'Bang'
	| 'Question'
	| 'Colon'
	| 'Semicolon'
	| 'Quote'
	| 'Comma'
	| 'Hyphen'
	| 'Apostrophe'
	| 'OpenSquare'
	| 'CloseSquare'
	| 'OpenRound'
	| 'CloseRound'
	| 'Hash';

export async function parseText(text: string, useWasm = defaultUseWasm): Promise<Token[]> {
	if (useWasm) {
		const wasm = await import('wasm');

		const tokens = wasm.parse(text);
		return tokens;
	} else {
		const req = await fetch('/parse', {
			method: 'POST',
			body: JSON.stringify({ text }),
			headers: {
				'CONTENT-TYPE': 'application/json'
			}
		});

		const res: ParseResponse = await req.json();
		return res.tokens;
	}
}

export function contentToString(content: string[]): string {
	return content.reduce((p, c) => p + c, '');
}

export interface LintResponse {
	lints: Lint[];
}

export interface Lint {
	span: Span;
	lint_kind: 'Capitalization' | 'Spelling';
	suggestions: Suggestion[];
	message: string;
}

export interface Suggestion {
	ReplaceWith: string[];
}

export interface Span {
	start: number;
	end: number;
}

export function spanContent(span: Span, source: string): string {
	return source.substring(span.start, span.end);
}

export async function lintText(text: string, useWasm = defaultUseWasm): Promise<Lint[]> {
	let lints;

	if (useWasm) {
		const wasm = await import('wasm');

		lints = wasm.lint(text);
	} else {
		const req = await fetch(`/lint`, {
			method: 'POST',
			body: JSON.stringify({ text }),
			headers: {
				'CONTENT-TYPE': 'application/json'
			}
		});

		const res: LintResponse = await req.json();

		lints = res.lints;
	}

	// We only want to show fixable errors.
	lints = lints.filter((lint) => lint.suggestions.length > 0);
	// The `Underlines` component assumes the lints do not overlap.
	lints = removeOverlaps(lints);

	return lints;
}

export async function applySuggestion(
	text: string,
	suggestion: Suggestion,
	span: Span,
	useWasm = defaultUseWasm
): Promise<string> {
	if (useWasm) {
		const wasm = await import('wasm');

		const applied = wasm.apply_suggestion(text, span, suggestion);
		return applied;
	} else {
		const req = await fetch(`/apply`, {
			method: 'POST',
			body: JSON.stringify({ text, suggestion, span }),
			headers: {
				'CONTENT-TYPE': 'application/json'
			}
		});

		const res = await req.json();
		return res.text;
	}
}

/** Removes lints whose spans overlap.
 * NOTE: __Will__ reorder the lints. */
function removeOverlaps(lints: Lint[]) {
	const sorted = lints.sort((a, b) => a.span.start - b.span.start);

	let overlapsFound = false;

	const filtered = sorted.filter((value, idx, arr) => {
		const next = arr[idx + 1];

		if (next != null && next.span.start < value.span.end) {
			overlapsFound = true;
			return false;
		} else {
			return true;
		}
	});

	if (overlapsFound) {
		return removeOverlaps(filtered);
	} else {
		return filtered;
	}
}

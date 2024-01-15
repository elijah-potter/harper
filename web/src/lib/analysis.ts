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

export async function parseText(text: string, use_wasm = true): Promise<Token[]> {
	if (use_wasm) {
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

export async function lintText(text: string, use_wasm = true): Promise<Lint[]> {
	if (use_wasm) {
		const wasm = await import('wasm');

		const lints = wasm.lint(text);
		return lints;
	} else {
		const req = await fetch(`/lint`, {
			method: 'POST',
			body: JSON.stringify({ text }),
			headers: {
				'CONTENT-TYPE': 'application/json'
			}
		});

		const res: LintResponse = await req.json();
		return res.lints;
	}
}

export async function applySuggestion(
	text: string,
	suggestion: Suggestion,
	span: Span,
	use_wasm = true
): Promise<string> {
	if (use_wasm) {
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

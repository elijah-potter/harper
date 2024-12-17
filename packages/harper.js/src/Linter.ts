import type { Lint, Span, Suggestion } from 'wasm';

/** A interface for an object that can perform linting actions. */
export default interface Linter {
	/** Complete any setup that is necessary before linting. This may include downloading and compiling the WebAssembly binary.
	 * This setup will complete when needed regardless of whether you call this function.
	 * This function exists to allow you to do this work when it is of least impact to the user experiences (i.e. while you're loading something else). */
	setup(): Promise<void>;
	/** Lint the provided text. */
	lint(text: string): Promise<Lint[]>;
	/** Apply a suggestion to the given text, returning the transformed result. */
	applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string>;
	/** Determine if the provided text is likely to be intended to be English.
	 * The algorithm can be described as "proof of concept" and as such does not work terribly well.*/
	isLikelyEnglish(text: string): Promise<boolean>;
	/** Determine which parts of a given string are intended to be English, returning those bits.
	 * The algorithm can be described as "proof of concept" and as such does not work terribly well.*/
	isolateEnglish(text: string): Promise<string>;
}

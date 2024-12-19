import type { Lint, Span, Suggestion } from 'wasm';
import Linter from './Linter';
import LocalLinter from './LocalLinter';
import WorkerLinter from './WorkerLinter';

export { LocalLinter, WorkerLinter };
export type { Linter, Lint, Span, Suggestion };

export enum SuggestionKind {
	Replace = 0,
	Remove = 1
}

export type LintConfig = Record<string, boolean | undefined>;

import type { Lint, Span, Suggestion } from 'wasm';
import Linter from './Linter';
import LocalLinter from './LocalLinter';
import WorkerLinter from './WorkerLinter';

export { Lint, Span, Suggestion, LocalLinter, WorkerLinter };
export type { Linter };

import { expect, test } from 'vitest';
import { deserializeArg, serializeArg } from './communication';
import { Span } from 'wasm';
import LocalLinter from '../LocalLinter';

test('works with strings', () => {
	const start = 'This is a string';

	const end = deserializeArg(structuredClone(serializeArg(start)));

	expect(end).toBe(start);
	expect(typeof end).toBe(typeof start);
});

test('works with false booleans', () => {
	const start = false;

	const end = deserializeArg(structuredClone(serializeArg(start)));

	expect(end).toBe(start);
	expect(typeof end).toBe(typeof start);
});

test('works with true booleans', () => {
	const start = true;

	const end = deserializeArg(structuredClone(serializeArg(start)));

	expect(end).toBe(start);
	expect(typeof end).toBe(typeof start);
});

test('works with numbers', () => {
	const start = 123;

	const end = deserializeArg(structuredClone(serializeArg(start)));

	expect(end).toBe(start);
	expect(typeof end).toBe(typeof start);
});

test('works with Spans', () => {
	const start = Span.new(123, 321);

	const end = deserializeArg(structuredClone(serializeArg(start)));

	expect(end.start).toBe(start.start);
	expect(end.len()).toBe(start.len());
	expect(typeof end).toBe(typeof start);
});

test('works with Lints', async () => {
	const linter = new LocalLinter();
	const lints = await linter.lint('This is an test.');
	const start = lints[0];

	expect(start).not.toBeNull();

	const end = deserializeArg(structuredClone(serializeArg(start)));

	expect(end.message()).toBe(start.message());
	expect(end.lint_kind()).toBe(start.lint_kind());
});

import { expect, test } from 'vitest';
import LocalLinter from './LocalLinter';

test('detects repeated words', async () => {
	const linter = new LocalLinter();

	const lints = await linter.lint('The the problem is...');

	expect(lints.length).toBe(1);
});

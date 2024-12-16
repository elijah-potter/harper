import { expect, test } from 'vitest';
import WorkerLinter from './WorkerLinter';

test('detects repeated words', async () => {
	const linter = new WorkerLinter();

	const lints = await linter.lint('The the problem is...');

	expect(lints.length).toBe(1);
});

test('detects repeated words with multiple synchronous requests', async () => {
	const linter = new WorkerLinter();

	const promises = [
		linter.lint('The problem is that that...'),
		linter.lint('The problem is...'),
		linter.lint('The the problem is...')
	];

	const results = [];

	for (const promise of promises) {
		results.push(await promise);
	}

	expect(results[0].length).toBe(1);
	expect(results[0][0].suggestions().length).toBe(1);
	expect(results[1].length).toBe(0);
	expect(results[2].length).toBe(1);
});

test('detects repeated words with concurrent requests', async () => {
	const linter = new WorkerLinter();

	const promises = [
		linter.lint('The problem is that that...'),
		linter.lint('The problem is...'),
		linter.lint('The the problem is...')
	];

	const results = await Promise.all(promises);

	expect(results[0].length).toBe(1);
	expect(results[0][0].suggestions().length).toBe(1);
	expect(results[1].length).toBe(0);
	expect(results[2].length).toBe(1);
});

test('detects lorem ipsum paragraph as not english', async () => {
	const linter = new WorkerLinter();

	const result = await linter.isLikelyEnglish(
		'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.'
	);

	expect(result).toBeTypeOf('boolean');
	expect(result).toBe(false);
});

test('can run setup without issues', async () => {
	const linter = new WorkerLinter();

	await linter.setup();
});

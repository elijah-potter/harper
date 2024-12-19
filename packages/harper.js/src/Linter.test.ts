import { expect, test } from 'vitest';
import WorkerLinter from './WorkerLinter';
import LocalLinter from './LocalLinter';

const linters = {
	WorkerLinter: WorkerLinter,
	LocalLinter: LocalLinter
};

for (const [linterName, Linter] of Object.entries(linters)) {
	test(`${linterName} detects repeated words`, async () => {
		const linter = new Linter();

		const lints = await linter.lint('The the problem is...');

		expect(lints.length).toBe(1);
	});

	test(`${linterName} detects repeated words with multiple synchronous requests`, async () => {
		const linter = new Linter();

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

	test(`${linterName} detects repeated words with concurrent requests`, async () => {
		const linter = new Linter();

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

	test(`${linterName} detects lorem ipsum paragraph as not english`, async () => {
		const linter = new Linter();

		const result = await linter.isLikelyEnglish(
			'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.'
		);

		expect(result).toBeTypeOf('boolean');
		expect(result).toBe(false);
	});

	test(`${linterName} can run setup without issues`, async () => {
		const linter = new Linter();

		await linter.setup();
	});

	test(`${linterName} contains configuration option for repetition`, async () => {
		const linter = new Linter();

		const lintConfig = await linter.getLintConfig();
		expect(lintConfig).toHaveProperty('repeated_words');
	});

	test(`${linterName} can both get and set its configuration`, async () => {
		const linter = new Linter();

		let lintConfig = await linter.getLintConfig();

		for (const key of Object.keys(lintConfig)) {
			lintConfig[key] = true;
		}

		await linter.setLintConfig(lintConfig);
		lintConfig = await linter.getLintConfig();

		for (const key of Object.keys(lintConfig)) {
			expect(lintConfig[key]).toBe(true);
		}
	});
}

test('Linters have the same config format', async () => {
	const configs = [];

	for (const Linter of Object.values(linters)) {
		const linter = new Linter();

		configs.push(await linter.getLintConfig());
	}

	for (const config of configs) {
		expect(config).toEqual(configs[0]);
		expect(config).toBeTypeOf('object');
	}
});

test('Linters have the same JSON config format', async () => {
	const configs = [];

	for (const Linter of Object.values(linters)) {
		const linter = new Linter();

		configs.push(await linter.getLintConfigAsJSON());
	}

	for (const config of configs) {
		expect(config).toEqual(configs[0]);
		expect(config).toBeTypeOf('string');
	}
});

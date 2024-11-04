import {
	activateHarper,
	compareActualVsExpectedDiagnostics,
	createExpectedDiagnostics,
	createRange,
	getActualDiagnostics,
	openFile,
	sleep
} from './helper';

describe('Languages >', () => {
	beforeAll(async () => {
		await activateHarper();
		// Wait for `harper-ls` to start
		await sleep(500);
	});

	[
		// Uncomment when #265 is fixed.
		// { type: 'JavaScript JSX', file: 'javascriptreact.jsx', row: 1, column: 36 },
		// Uncomment when #65 is fixed.
		// { type: 'Shellscript without extension', file: 'shellscript', row: 2, column: 2 },

		// VSCode doesn't support Nix and TOML files out of the box. Uncomment when you figure out how
		// to support them during testing.
		// { type: 'Nix', file: 'nix.nix', row: 1, column: 2 },
		// { type: 'TOML', file: 'toml.toml', row: 1, column: 2 },

		{ type: 'C', file: 'c.c', row: 2, column: 3 },
		{ type: 'C++', file: 'cpp.cpp', row: 3, column: 5 },
		{ type: 'H', file: 'cpp.h', row: 0, column: 3 },
		{ type: 'C#', file: 'csharp.cs', row: 2, column: 2 },
		{ type: 'Git Commit', file: 'git-commit', row: 0, column: 0 },
		{ type: 'Go', file: 'go.go', row: 4, column: 4 },
		{ type: 'HTML', file: 'html.html', row: 8, column: 6 },
		{ type: 'Java', file: 'java.java', row: 2, column: 17 },
		{ type: 'JavaScript', file: 'javascript.js', row: 1, column: 3 },
		{ type: 'Lua', file: 'lua.lua', row: 0, column: 24 },
		{ type: 'Plaintext without extension', file: 'plaintext', row: 0, column: 0 },
		{ type: 'Plaintext with extension', file: 'plaintext.txt', row: 4, column: 0 },
		{ type: 'Python', file: 'python.py', row: 1, column: 2 },
		{ type: 'Ruby', file: 'ruby.rb', row: 3, column: 16 },
		{ type: 'Rust', file: 'rust.rs', row: 0, column: 4 },
		{ type: 'Shellscript with .bash extension', file: 'shellscript.bash', row: 7, column: 9 },
		{ type: 'Shellscript with .sh extension', file: 'shellscript.sh', row: 0, column: 22 },
		{ type: 'Swift', file: 'swift.swift', row: 9, column: 26 },
		{ type: 'TypeScript', file: 'typescript.ts', row: 0, column: 32 },
		{ type: 'TypeScript JSX', file: 'typescriptreact.tsx', row: 3, column: 7 }
	].forEach((testCase) => {
		it(`gives correct diagnostics for ${testCase.type} files`, async () => {
			const uri = await openFile('languages', testCase.file);

			// Wait for `harper-ls` to send diagnostics
			await sleep(50);

			compareActualVsExpectedDiagnostics(
				getActualDiagnostics(uri),
				createExpectedDiagnostics({
					message: 'Did you mean to spell “Errorz” this way?',
					range: createRange(testCase.row, testCase.column, testCase.row, testCase.column + 6)
				})
			);
		});
	});
});

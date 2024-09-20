import type { Extension } from 'vscode';

import {
	ConfigurationTarget,
	Diagnostic,
	DiagnosticSeverity,
	extensions,
	languages,
	Position,
	Range,
	Uri,
	window,
	workspace
} from 'vscode';

describe('Harper Extension', () => {
	let harper: Extension<void>;
	let documentUri: Uri;

	beforeAll(async () => {
		harper = extensions.getExtension('elijah-potter.harper')!;
		await harper.activate();

		// Open test file so diagnostics can occur
		documentUri = Uri.joinPath(Uri.file(workspace.workspaceFolders![0].uri.path), 'integration.md');
		await window.showTextDocument(await workspace.openTextDocument(documentUri));

		// Wait for `harper-ls` to start
		await sleep(500);
	});

	it('runs', () => {
		expect(harper.isActive).toBe(true);
	});

	it('gives correct diagnostics', () => {
		const actual = languages.getDiagnostics(documentUri);
		const expected: Diagnostic[] = [
			{
				source: 'Harper',
				message: 'Did you mean to repeat this word?',
				severity: DiagnosticSeverity.Information,
				range: new Range(new Position(2, 39), new Position(2, 48))
			},
			{
				source: 'Harper',
				message: 'Did you mean to spell “errorz” this way?',
				severity: DiagnosticSeverity.Information,
				range: new Range(new Position(2, 26), new Position(2, 32))
			}
		];

		expect(actual.length).toBe(expected.length);
		for (let i = 0; i < actual.length; i++) {
			expect(actual[i].source).toBe(expected[i].source);
			expect(actual[i].message).toBe(expected[i].message);
			expect(actual[i].severity).toBe(expected[i].severity);
			expect(actual[i].range).toEqual(expected[i].range);
		}
	});

	it('updates diagnostics on configuration change', async () => {
		const config = workspace.getConfiguration('harper-ls.linters');
		await config.update('repeated_words', false, ConfigurationTarget.Workspace);
		// Wait for `harper-ls` to restart
		await sleep(1000);

		const actual = languages.getDiagnostics(documentUri);
		const expected: Diagnostic[] = [
			{
				source: 'Harper',
				message: 'Did you mean to spell “errorz” this way?',
				severity: DiagnosticSeverity.Information,
				range: new Range(new Position(2, 26), new Position(2, 32))
			}
		];

		expect(actual.length).toBe(expected.length);
		for (let i = 0; i < actual.length; i++) {
			expect(actual[i].source).toBe(expected[i].source);
			expect(actual[i].message).toBe(expected[i].message);
			expect(actual[i].severity).toBe(expected[i].severity);
			expect(actual[i].range).toEqual(expected[i].range);
		}

		// Set config back to default value
		await config.update('repeated_words', true, ConfigurationTarget.Workspace);
	});
});

function sleep(duration: number) {
	return new Promise((resolve) => setTimeout(resolve, duration));
}

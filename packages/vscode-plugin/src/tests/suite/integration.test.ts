import type { Extension } from 'vscode';

import { ConfigurationTarget, Uri, workspace } from 'vscode';

import {
	activateHarper,
	compareActualVsExpectedDiagnostics,
	createExpectedDiagnostics,
	createRange,
	getActualDiagnostics,
	openFile,
	sleep
} from './helper';

describe('Integration >', () => {
	let harper: Extension<void>;
	let markdownUri: Uri;

	beforeAll(async () => {
		harper = await activateHarper();
		// Open test file so diagnostics can occur
		markdownUri = await openFile('integration.md');
		// Wait for `harper-ls` to start
		await sleep(500);
	});

	it('runs', () => {
		expect(harper.isActive).toBe(true);
	});

	it('gives correct diagnostics', () => {
		compareActualVsExpectedDiagnostics(
			getActualDiagnostics(markdownUri),
			createExpectedDiagnostics(
				{
					message: 'Did you mean to repeat this word?',
					range: createRange(2, 39, 2, 48)
				},
				{
					message: 'Did you mean to spell “errorz” this way?',
					range: createRange(2, 26, 2, 32)
				}
			)
		);
	});

	it('updates diagnostics on configuration change', async () => {
		const config = workspace.getConfiguration('harper-ls.linters');
		await config.update('repeated_words', false, ConfigurationTarget.Workspace);
		// Wait for `harper-ls` to update diagnostics
		await sleep(250);

		compareActualVsExpectedDiagnostics(
			getActualDiagnostics(markdownUri),
			createExpectedDiagnostics({
				message: 'Did you mean to spell “errorz” this way?',
				range: createRange(2, 26, 2, 32)
			})
		);

		// Set config back to default value
		await config.update('repeated_words', true, ConfigurationTarget.Workspace);
	});
});

import { runTests } from '@vscode/test-electron';
import path from 'node:path';

(async () => {
	try {
		await runTests({
			extensionDevelopmentPath: path.join(__dirname, '..', '..'),
			extensionTestsPath: path.join(__dirname, 'suite'),
			launchArgs: ['--disable-extensions']
		});
	} catch (error) {
		console.error('Failed to run tests', error);
		process.exit(1);
	}
})();

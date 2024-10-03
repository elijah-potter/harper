import type { ExtensionContext } from 'vscode';
import type { Executable, LanguageClientOptions } from 'vscode-languageclient/node';

import { commands, Uri, window, workspace } from 'vscode';
import { LanguageClient, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
const serverOptions: Executable = { command: '', transport: TransportKind.stdio };
const clientOptions: LanguageClientOptions = {
	documentSelector: [
		{ language: 'html' },
		{ language: 'markdown' },
		{ language: 'rust' },
		{ language: 'typescriptreact' },
		{ language: 'typescript' },
		{ language: 'py' },
		{ language: 'javascript' },
		{ language: 'javascriptreact' },
		{ language: 'go' },
		{ language: 'c' },
		{ language: 'cpp' },
		{ language: 'ruby' },
		{ language: 'swift' },
		{ language: 'csharp' },
		{ language: 'toml' },
		{ language: 'lua' },
		{ language: 'sh' },
		{ language: 'java' }
	]
};

export async function activate(context: ExtensionContext): Promise<void> {
	serverOptions.command = Uri.joinPath(
		context.extensionUri,
		'bin',
		`harper-ls${process.platform === 'win32' ? '.exe' : ''}`
	).fsPath;

	clientOptions.outputChannel = window.createOutputChannel('Harper');
	context.subscriptions.push(clientOptions.outputChannel);

	try {
		const manifest: { contributes: { configuration: { properties: { [key: string]: object } } } } =
			JSON.parse(
				(await workspace.fs.readFile(Uri.joinPath(context.extensionUri, 'package.json'))).toString()
			);
		const configs = Object.keys(manifest.contributes.configuration.properties);
		context.subscriptions.push(
			workspace.onDidChangeConfiguration(async (event) => {
				if (configs.find((c) => event.affectsConfiguration(c))) {
					clientOptions.outputChannel?.appendLine('Configuration changed, restarting server');
					await startLanguageServer();
				}
			})
		);
	} catch (error) {
		showError('Failed to read manifest file', error);
	}

	context.subscriptions.push(
		commands.registerCommand('harper.languageserver.restart', startLanguageServer)
	);

	await startLanguageServer();
}

async function startLanguageServer(): Promise<void> {
	if (client && client.needsStop()) {
		if (client.diagnostics) {
			client.diagnostics.clear();
		}

		try {
			await client.stop(2000);
		} catch (error) {
			showError('Failed to stop harper-ls', error);
			return;
		}
	}

	try {
		client = new LanguageClient('harper', 'Harper', serverOptions, clientOptions);
		await client.start();
	} catch (error) {
		showError('Failed to start harper-ls', error);
		client = undefined;
	}
}

function showError(message: string, error: Error | unknown): void {
	let info = '';
	if (error instanceof Error) {
		info = error.stack ? error.stack : error.message;
	}

	window.showErrorMessage(message, 'Show Info', 'Dismiss').then((selected) => {
		if (selected === 'Show Info') {
			clientOptions.outputChannel?.appendLine('---');
			clientOptions.outputChannel?.appendLine(message);
			clientOptions.outputChannel?.appendLine(info);
			clientOptions.outputChannel?.appendLine(
				'If the issue persists, please report at https://github.com/elijah-potter/harper/issues'
			);
			clientOptions.outputChannel?.appendLine('---');
			clientOptions.outputChannel?.show();
		}
	});
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}

	return client.stop();
}

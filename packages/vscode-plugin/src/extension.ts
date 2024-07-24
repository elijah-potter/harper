import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate() {
	const server: Executable = {
		command: 'harper-ls',
		transport: TransportKind.stdio
	};

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

	// Create the language client and start the client.
	client = new LanguageClient('harper-ls', 'Harper', server, clientOptions);

	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}

import type { Diagnostic, Extension } from 'vscode';

import {
	DiagnosticSeverity,
	extensions,
	languages,
	Position,
	Range,
	Uri,
	window,
	workspace
} from 'vscode';

export async function activateHarper(): Promise<Extension<void>> {
	const harper = extensions.getExtension('elijah-potter.harper')!;

	if (!harper.isActive) {
		await harper.activate();
	}

	return harper;
}

export async function openFile(...pathSegments: string[]): Promise<Uri> {
	const uri = Uri.joinPath(Uri.file(workspace.workspaceFolders![0].uri.path), ...pathSegments);
	await window.showTextDocument(await workspace.openTextDocument(uri));
	return uri;
}

export function getActualDiagnostics(resource: Uri): Diagnostic[] {
	return languages.getDiagnostics(resource).filter((d) => d.source === 'Harper');
}

export function createExpectedDiagnostics(
	...data: { message: string; range: Range }[]
): Diagnostic[] {
	return data.map((d) => ({ ...d, source: 'Harper', severity: DiagnosticSeverity.Information }));
}

export function compareActualVsExpectedDiagnostics(
	actual: Diagnostic[],
	expected: Diagnostic[]
): void {
	expect(actual.length).toBe(expected.length);
	for (let i = 0; i < actual.length; i++) {
		expect(actual[i].source).toBe(expected[i].source);
		expect(actual[i].message).toBe(expected[i].message);
		expect(actual[i].severity).toBe(expected[i].severity);
		expect(actual[i].range).toEqual(expected[i].range);
	}
}

export function createRange(
	startRow: number,
	startColumn: number,
	endRow: number,
	endColumn: number
): Range {
	return new Range(new Position(startRow, startColumn), new Position(endRow, endColumn));
}

export function sleep(duration: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, duration));
}

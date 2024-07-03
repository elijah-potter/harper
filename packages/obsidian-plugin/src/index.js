import { linter } from './lint';
import { Plugin } from 'obsidian';
import wasm from 'wasm/harper_wasm_bg.wasm';
import init, { lint, use_spell_check } from 'wasm';

function contentToString(content) {
	return content.reduce((p, c) => p + c, '');
}

function suggestionToLabel(sug) {
	if (sug === 'Remove') {
		return 'Remove';
	} else {
		return `Replace with "${contentToString(sug.ReplaceWith)}"`;
	}
}

const harperLinter = linter(
	async (view) => {
		let text = view.state.doc.sliceString(-1);

		await init(await wasm());
		use_spell_check(false);
		let lints = lint(text);

		return lints.map((lint) => {
			console.log('Rendering lints...');

			return {
				from: lint.span.start,
				to: lint.span.end,
				severity: 'error',
				title: lint.lint_kind,
				message: lint.message,
				actions: lint.suggestions.map((sug) => {
					return {
						name: suggestionToLabel(sug),
						apply: (view) => {
							if (sug === 'Remove') {
								view.dispatch({
									changes: {
										from: lint.span.start,
										to: lint.span.end,
										insert: ''
									}
								});
							} else {
								view.dispatch({
									changes: {
										from: lint.span.start,
										to: lint.span.end,
										insert: contentToString(sug.ReplaceWith)
									}
								});
							}
						}
					};
				})
			};
		});
	},
	{ delay: -1 }
);

export default class HarperPlugin extends Plugin {
	async onload() {
		this.registerEditorExtension([harperLinter]);
	}
}

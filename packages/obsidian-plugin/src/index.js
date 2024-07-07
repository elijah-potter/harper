import { linter } from './lint';
import { Plugin } from 'obsidian';
import wasm from 'wasm/harper_wasm_bg.wasm';
import init, { lint, use_spell_check } from 'wasm';

function suggestionToLabel(sug) {
	if (sug.kind() == 'Remove') {
		return 'Remove';
	} else {
		return `Replace with "${sug.get_replacement_text()}"`;
	}
}

const harperLinter = linter(
	async (view) => {
		const text = view.state.doc.sliceString(-1);

		await init(await wasm());

		use_spell_check(false);
		const lints = lint(text);

		return lints.map((lint) => {
			let span = lint.span();

			return {
				from: span.start,
				to: span.end,
				severity: 'error',
				title: lint.lint_kind(),
				message: lint.message(),
				actions: lint.suggestions().map((sug) => {
					return {
						name: suggestionToLabel(sug),
						apply: (view) => {
							if (sug === 'Remove') {
								view.dispatch({
									changes: {
										from: span.start,
										to: span.end,
										insert: ''
									}
								});
							} else {
								view.dispatch({
									changes: {
										from: span.start,
										to: span.end,
										insert: sug.get_replacement_text()
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

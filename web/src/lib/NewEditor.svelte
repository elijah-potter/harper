<script lang="ts">
	import { onMount } from 'svelte';
	import { minimalSetup, EditorView } from 'codemirror';
	import { contentToString, lintText, type Suggestion } from './analysis';
	import { linter } from '@codemirror/lint';

	let editorDiv: HTMLDivElement;

	function suggestionToLabel(sug: Suggestion) {
		if (sug === 'Remove') {
			return 'Remove';
		} else {
			return `Replace with "${contentToString(sug.ReplaceWith)}"`;
		}
	}

	const harperLinter = linter(
		async (view) => {
			let text = view.state.doc.sliceString(0);

			let lints = await lintText(text);

			return lints.map((lint) => {
				return {
					from: lint.span.start,
					to: lint.span.end,
					severity: 'warning',
					message: lint.message,
					actions: lint.suggestions.map((sug, i) => {
						return {
							name: suggestionToLabel(sug),
							apply: (view) => {
								if (sug === 'Remove') {
									view.dispatch({
										changes: { from: lint.span.start, to: lint.span.end, insert: '' }
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
		{ delay: 0 }
	);

	onMount(() => {
		new EditorView({
			doc: 'You cannot add new lines in this editor\n\n\n\n',
			extensions: [minimalSetup, harperLinter],
			parent: editorDiv
		});
	});
</script>

<div bind:this={editorDiv} />

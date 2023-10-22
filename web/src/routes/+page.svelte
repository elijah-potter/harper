<script lang="ts">
	import Suggestion from '$lib/Suggestion.svelte';
	import alice from '../../../alice.txt?raw';
	import { position } from 'caret-pos';
	import { highlightText } from '$lib/parse';

	let content = alice;
	let editor: HTMLDivElement;

	function updateContent() {
		if (editor !== null) {
			content = editor.innerText;
		}
	}

	function highlight(node: HTMLDivElement, text: string) {
		function action(text: string) {
			if (editor) {
				const pos = position(editor);
				highlightText(text).then((highlighted) => {
					node.innerHTML = highlighted;
					position(editor, pos.pos);
				});
			} else {
				highlightText(text).then((highlighted) => {
					node.innerHTML = highlighted;
				});
			}
		}
		action(text);
		return {
			update(text: string) {
				action(text);
			},
			onchange(text: string) {
				action(text);
			}
		};
	}
</script>

<div class="flex flex-row w-full h-screen">
	<div
		class="flex-auto h-full p-5"
		contenteditable="true"
		use:highlight={content}
		bind:this={editor}
		placeholder="Tell your story..."
		on:input={updateContent}
	/>
	<div class="flex flex-col flex-none">
		<Suggestion category="grammar" title="Word Repeated">
			<p>The word "the" was repeated twice</p>
		</Suggestion>

		<Suggestion category="spelling" title="Mispelling">
			<p>Replace <b>"thk"</b> with <b>"the"</b>.</p>
		</Suggestion>
	</div>
</div>

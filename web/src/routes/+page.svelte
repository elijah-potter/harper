<script lang="ts">
	import Suggestion from '$lib/Suggestion.svelte';
	import { onMount } from 'svelte';
	import alice from '../../../alice.txt?raw';
	import { position, offset } from 'caret-pos';

	let content = alice;
	let editor: HTMLDivElement;

	function updateContent() {
		if (editor !== null) {
			content = editor.textContent;
		}
	}

	function highlight(node: HTMLDivElement, text: string) {
		function action(text: string) {
			if (editor) {
				const pos = position(editor);
				if (text) node.innerHTML = text.replaceAll('Duchess', `<b class="text-bold">Duchess</b>`);
				position(editor, pos.pos);
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
		on:input={updateContent}
	/>
	<div class="flex flex-col">
		<Suggestion category="grammar" title="Word Repeated">
			<p>The word "the" was repeated twice</p>
		</Suggestion>

		<Suggestion category="spelling" title="Mispelling">
			<p>Replace <b>"thk"</b> with <b>"the"</b>.</p>
		</Suggestion>
	</div>
</div>

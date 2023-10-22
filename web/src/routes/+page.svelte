<script lang="ts">
	import Suggestion from '$lib/Suggestion.svelte';
	import alice from '../../../alice.txt?raw';
	import { highlightText } from '$lib/parse';

	let content = alice;
	let highlighted = '';
	let highlight: HTMLDivElement;
	let editor: HTMLTextAreaElement;

	$: highlightText(content).then((v) => (highlighted = v));
</script>

<div class="flex flex-row w-full h-screen">
	<div class="flex-auto h-full p-5 grid z-10">
		<div class="overflow-auto" style="grid-row: 1; grid-column: 1" bind:this={highlight}>
			<div class="whitespace-pre-wrap break-words">
				{@html highlighted}
			</div>
		</div>
		<textarea
			class="w-full h-full m-0 rounded-none p-0 z-0 bg-transparent border-none"
			style="grid-row: 1; grid-column: 1"
			bind:value={content}
			on:scroll={() => highlight.scrollTo(editor.scrollTop)}
			bind:this={editor}
		></textarea>
	</div>
	<div class="flex flex-col flex-none">
		<Suggestion category="grammar" title="Word Repeated">
			<p>The word "the" was repeated twice</p>
		</Suggestion>

		<Suggestion category="spelling" title="Mispelling">
			<p>Replace <b>"thk"</b> with <b>"the"</b>.</p>
		</Suggestion>
	</div>
</div>

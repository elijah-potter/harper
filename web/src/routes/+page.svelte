<script lang="ts">
	import Suggestion from '$lib/Suggestion.svelte';
	import alice from '../../../alice.txt?raw';
	import Highlights from '$lib/Highlights.svelte';
	import { Button } from 'flowbite-svelte';
	import { lintText, applySuggestion } from '$lib/analysis';
	import type { Lint } from '$lib/analysis';

	let content = alice;
	let editor: HTMLTextAreaElement;

	let lints: Lint[] = [];

	$: lintText(content).then((newLints) => (lints = newLints));

	function ping() {
		lintText(content).then((newLints) => (lints = newLints));
		setTimeout(ping, 1000);
	}

	ping();
</script>

<div class="flex flex-row w-full h-screen">
	<div class="flex-auto h-full p-5 grid z-10">
		<div class="overflow-auto p-0" style="grid-row: 1; grid-column: 1">
			<Highlights {content}></Highlights>
		</div>
		<textarea
			class="w-full h-full m-0 rounded-none p-0 z-0 bg-transparent border-none"
			style="grid-row: 1; grid-column: 1"
			bind:value={content}
			bind:this={editor}
		></textarea>
	</div>
	<div class="flex flex-col flex-grow">
		{#each lints as lint}
			{#each lint.suggestions as suggestion}
				<Suggestion category={'mild'} title={lint.lint_kind}>
					<Button
						color="alternative"
						class="w-full h-full m-3"
						on:click={() =>
							applySuggestion(content, suggestion, lint.span).then((edited) => (content = edited))}
					>
						Replace "{content.substring(lint.span.start, lint.span.end)}" with "{suggestion.ReplaceWith.reduce(
							(p, c) => p + c
						)}"
					</Button>
				</Suggestion>
			{/each}
		{/each}
	</div>
</div>

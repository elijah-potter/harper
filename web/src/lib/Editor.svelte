<script lang="ts">
	import { Card } from 'flowbite-svelte';
	import alice from '../../../alice.txt?raw';
	import Underlines from '$lib/Underlines.svelte';
	import { Button } from 'flowbite-svelte';
	import { lintText, applySuggestion, spanContent } from '$lib/analysis';
	import type { Lint } from '$lib/analysis';

	let content = alice;

	let lints: Lint[] = [];
	let focused: number | undefined;
	let editor: HTMLTextAreaElement | null;

	$: lintText(content).then((newLints) => (lints = newLints));
	$: boxHeight = calcHeight(content);

	function calcHeight(boxContent: string): number {
		let numberOfLineBreaks = (boxContent.match(/\n/g) || []).length;
		let newHeight = 20 + numberOfLineBreaks * 30 + 12 + 2;
		console.log(newHeight);
		return newHeight;
	}
</script>

<div class="flex flex-row w-full h-full p-5">
	<Card
		class="flex-grow h-full p-5 grid z-10 max-w-full text-lg overflow-auto mr-5"
		on:click={() => editor && editor.focus()}
	>
		<div class="m-0 p-0" style="grid-row: 1; grid-column: 1">
			<Underlines {content} focusLintIndex={focused} />
		</div>
		<textarea
			bind:this={editor}
			class="w-full m-0 rounded-none p-0 z-0 bg-transparent border-none text-lg resize-none focus:border-0"
			spellcheck="false"
			style={`grid-row: 1; grid-column: 1; height: ${boxHeight}px`}
			bind:value={content}
		></textarea>
	</Card>
	<Card class="flex flex-col flex-none basis-[400px] overflow-auto h-full">
		<h2 class="text-2xl font-bold m-1">Suggestions</h2>
		{#each lints as lint, i}
			<Card class="m-1 hover:translate-x-3 transition-all" on:click={() => (focused = i)}>
				<div class="pl-2 border-l-[3px] border-l-primary-500">
					<div class="flex flex-row">
						<h3 class="font-bold">
							{lint.lint_kind} - “<span class="italic">
								{spanContent(lint.span, content)}
							</span> ”
						</h3>
					</div>
					<div
						class="transition-all overflow-hidden flex flex-col justify-evenly"
						style={`height: ${
							focused === i ? `calc(55px * ${lint.suggestions.length + 1})` : '0px'
						}`}
					>
						<p style="height: 50px">{lint.message}</p>
						{#each lint.suggestions as suggestion}
							<div class="w-full p-[4px]">
								<Button
									color="primary"
									class="w-full"
									style="height: 40px; margin: 5px 0px;"
									on:click={() =>
										applySuggestion(content, suggestion, lint.span).then(
											(edited) => (content = edited)
										)}
								>
									Replace "{content.substring(lint.span.start, lint.span.end)}" with "{suggestion.ReplaceWith.reduce(
										(p, c) => p + c
									)}"
								</Button>
							</div>
						{/each}
					</div>
				</div>
			</Card>
		{/each}
	</Card>
</div>

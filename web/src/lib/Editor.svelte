<script lang="ts">
	import { Card } from 'flowbite-svelte';
	import demo from '../../../demo.md?raw';
	import Underlines from '$lib/Underlines.svelte';
	import { Button } from 'flowbite-svelte';
	import { lintText, applySuggestion, spanContent } from '$lib/analysis';
	import type { Lint } from '$lib/analysis';

	let content = demo;

	let lints: Lint[] = [];
	let lintCards: HTMLButtonElement[] = [];
	let focused: number | undefined;
	let editor: HTMLTextAreaElement | null;

	$: lintText(content).then((newLints) => (lints = newLints));
	$: boxHeight = calcHeight(content);
	$: if (focused != null && lintCards[focused])
		lintCards[focused].scrollIntoView({ behavior: 'smooth' });

	$: if (editor != null && focused != null) {
		let lint = lints[focused % lints.length];
		if (lint != null) {
			let p = lint.span.end;
			editor.selectionStart = p;
			editor.selectionEnd = p;
		}
	}

	function calcHeight(boxContent: string): number {
		let numberOfLineBreaks = (boxContent.match(/\n/g) || []).length;
		let newHeight = 20 + numberOfLineBreaks * 30 + 12 + 2;
		return newHeight;
	}
</script>

<div class="flex flex-row w-full h-full p-5">
	<Card
		class="flex-grow h-full p-5 grid z-10 max-w-full text-lg overflow-auto mr-5"
		on:click={() => editor && editor.focus()}
	>
		<textarea
			bind:this={editor}
			class="w-full text-nowrap m-0 rounded-none p-0 z-0 bg-transparent overflow-hidden border-none text-lg resize-none focus:border-0"
			spellcheck="false"
			style={`grid-row: 1; grid-column: 1; height: ${boxHeight}px`}
			on:keydown={() => (focused = undefined)}
			bind:value={content}
		></textarea>
		<div class="m-0 p-0 z-10 pointer-events-none" style="grid-row: 1; grid-column: 1">
			<Underlines {content} bind:focusLintIndex={focused} />
		</div>
	</Card>
	<Card class="flex-none basis-[400px] h-full p-0">
		<h2 class="text-2xl font-bold m-1">Suggestions</h2>
		<div class="flex flex-col overflow-y-scroll overflow-x-hidden m-0 p-0">
			{#each lints as lint, i}
				<button
					class="block max-w-sm p-6 bg-white dark:bg-gray-800 border border-gray-200 rounded-lg shadow m-1 hover:translate-x-1 transition-all"
					on:click={() => (focused = i)}
					bind:this={lintCards[i]}
				>
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
											(p, c) => p + c,
											''
										)}"
									</Button>
								</div>
							{/each}
						</div>
					</div>
				</button>
			{/each}
		</div>
	</Card>
</div>

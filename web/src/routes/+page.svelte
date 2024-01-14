<script lang="ts">
	import { Card } from 'flowbite-svelte';
	import alice from '../../../alice.txt?raw';
	import Underlines from '$lib/Underlines.svelte';
	import { Button } from 'flowbite-svelte';
	import { lintText, applySuggestion, spanContent } from '$lib/analysis';
	import type { Lint } from '$lib/analysis';

	let content = alice;
	let editor: HTMLTextAreaElement;

	let lints: Lint[] = [];
	let focused: number | undefined;

	$: lintText(content).then((newLints) => (lints = newLints));

	function ping() {
		lintText(content).then((newLints) => (lints = newLints));
		setTimeout(ping, 1000);
	}

	type Category = 'mild' | 'moderate';

	function categoryToColor(category: Category): string {
		switch (category) {
			case 'mild':
				return '#708b75';
			case 'moderate':
				return '#f3a712';
		}
	}

	ping();

	$: console.log(focused);
</script>

<div class="flex flex-row w-full h-screen [&>*]:m-5">
	<Card class="flex-auto max-w-full h-full p-5 grid z-10 text-lg">
		<div class="overflow-auto p-0" style="grid-row: 1; grid-column: 1">
			<Underlines {content} focusLintIndex={focused} />
		</div>
		<textarea
			class="w-full h-full m-0 rounded-none p-0 z-0 bg-transparent border-none text-lg"
			spellcheck="false"
			style="grid-row: 1; grid-column: 1"
			bind:value={content}
			bind:this={editor}
		></textarea>
	</Card>
	<Card class="flex flex-col flex-grow">
		{#each lints as lint, i}
			<Card class="m-1 hover:translate-x-3 transition-all" on:click={() => (focused = i)}>
				<div class="pl-2" style={`border-left: 3px solid ${categoryToColor('mild')}`}>
					<div class="flex flex-row">
						<h3 class="font-bold">
							{lint.lint_kind} - “<span class="italic">
								{spanContent(lint.span, content)}
							</span> ”
						</h3>
					</div>
					<div
						class="transition-all overflow-hidden"
						style={`height: ${
							focused === i ? `calc(55px * ${lint.suggestions.length + 1})` : '0px'
						}`}
					>
						<p style="height: 50px">{lint.message}</p>
						{#each lint.suggestions as suggestion}
							<Button
								color="alternative"
								class="w-full mb-1"
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
						{/each}
					</div>
				</div>
			</Card>
		{/each}
	</Card>
</div>

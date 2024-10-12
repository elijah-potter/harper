<script lang="ts">
	// This is some of the shittiest code I've ever written.
	// It is quite hard to look at.
	// Someday, I'll return to it and spruce it up.
	// For now, it works.

	import { lintText } from '$lib/analysis';
	import type { Lint } from 'wasm';

	export let content: string;
	export let focusLintIndex: number | undefined;

	let lints: [Lint, number][] = [];
	let lintHighlights: HTMLSpanElement[] = [];

	$: lintText(content).then(
		(newLints) =>
			(lints = newLints
				.map<[Lint, number]>((lint, index) => [lint, index])
				.toSorted(([a], [b]) => a.span().start - b.span().end))
	);
	$: if (focusLintIndex != null && lintHighlights[focusLintIndex] != null)
		lintHighlights[focusLintIndex].scrollIntoView({
			behavior: 'smooth',
			block: 'nearest',
			inline: 'nearest'
		});

	function reOrgString(text: string): (string | undefined)[] {
		let output: (string | undefined)[] = [];

		for (let chunk of text.replaceAll(' ', '\u00A0').split('\n')) {
			if (output.length > 0) {
				output.push(undefined);
			}
			output.push(chunk);
		}

		return output;
	}

	type UnderlineDetails = {
		focused: boolean;
		content: string;
		index: number;
	};

	type UnderlineToken = string | null | undefined | UnderlineDetails;

	function processString(lintMap: [Lint, number][], focusLintIndex?: number) {
		let results: UnderlineToken[] = lintMap
			.map(([lint, lintIndex], index, arr) => {
				let prevStart = 0;
				let prev = arr[index - 1];

				if (prev != null) {
					prevStart = prev[0].span().end;
				}

				let prevEnd = lint.span().start;

				let prevContent = [];

				if (prevStart != prevEnd) {
					prevContent.push(...reOrgString(content.substring(prevStart, prevEnd)));
				}

				let lintContent: UnderlineDetails = {
					focused: lintIndex === focusLintIndex,
					index: lintIndex,
					content: lint.get_problem_text().replaceAll(' ', '\u00A0')
				};

				return [...prevContent, lintContent];
			})
			.flat();

		let lastLint = lints.at(-1);

		let finalChunk;

		if (lastLint != null) {
			finalChunk = content.substring(lastLint[0].span().end);
		} else {
			finalChunk = content;
		}

		results.push(...reOrgString(finalChunk));

		return results;
	}

	// string | [string, string, string, index] | null
	$: modified = processString(lints, focusLintIndex);
</script>

<div class="grid">
	<div class="p-0 m-0 text-nowrap indent-0 text-transparent" style="grid-row: 1; grid-column: 1">
		{#each modified as chunk}
			{#if chunk == null}
				<br />
			{:else if typeof chunk == 'string'}
				<span class="whitespace-pre">{chunk}</span>
			{:else}
				<span class="pointer-events-auto" style={`margin-right: -4px;`}>
					<button
						class={`underlinespecial transition-all rounded-sm ${chunk.focused ? 'animate-after-bigbounce text-white' : ''}`}
						bind:this={lintHighlights[chunk.index]}
						on:click={() =>
							chunk != null && typeof chunk == 'object' && (focusLintIndex = chunk.index)}
						style={`--line-color: #DB2B39; --line-width: ${chunk.focused ? '4px' : '2px'}; --bg-color: ${chunk.focused ? '#dbafb3' : 'transparent'};`}
					>
						{chunk.content}
					</button>
				</span>
			{/if}
		{/each}
	</div>
</div>

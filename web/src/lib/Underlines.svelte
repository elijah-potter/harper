<script lang="ts">
	import type { Lint, Token, TokenKind } from '$lib/analysis';
	import { contentToString, lintText, spanContent } from '$lib/analysis';

	export let content: string;
	export let focusLintIndex: number | undefined;

	let lints: [Lint, number][] = [];

	$: lintText(content).then(
		(newLints) =>
			(lints = newLints
				.map<[Lint, number]>((lint, index) => [lint, index])
				.toSorted(([a], [b]) => a.span.start - b.span.end))
	);

	function reOrgString(text: string): (string | undefined)[] {
		if (text.trim().length == 0) {
			return [''];
		}

		let output: (string | undefined)[] = [];

		for (let chunk of text.replaceAll(' ', '\u00A0').split('\n')) {
			if (output.length > 0) {
				output.push(undefined);
			}
			console.log(chunk);
			output.push(chunk);
		}

		return output;
	}

	function processString(lintMap: [Lint, number][], focusLintIndex?: number) {
		let results = lintMap
			.map(([lint, lintIndex], index, arr) => {
				let prevStart = 0;
				let prev = arr[index - 1];

				if (prev != null) {
					prevStart = prev[0].span.end;
				}

				let prevEnd = lint.span.start;

				let prevContent = [];

				if (prevStart != prevEnd) {
					prevContent.push(...reOrgString(content.substring(prevStart, prevEnd)));
				}

				let lintContent = [
					spanContent(lint.span, content).replaceAll(' ', '\u00A0'),
					'red',
					lintIndex === focusLintIndex ? '3px' : '1px'
				];

				return [...prevContent, lintContent];
			})
			.flat();

		let lastLint = lints.at(-1);

		let finalChunk;

		if (lastLint != null) {
			finalChunk = content.substring(lastLint[0].span.end);
		} else {
			finalChunk = content;
		}

		results.push(...reOrgString(finalChunk));

		return results;
	}

	// string | [string, string, string] | null
	$: modified = processString(lints, focusLintIndex);
</script>

<div class="grid">
	<div class="p-0 m-0 indent-0" style="grid-row: 1; grid-column: 1; color: transparent;">
		{#each modified as chunk}
			{#if chunk == null}
				<br />
			{:else if typeof chunk == 'string'}
				<span class="">{chunk}</span>
			{:else}
				<span style={`margin-right: -4px;`}>
					<span class="transition-all" style={`border-bottom: ${chunk[2]} solid ${chunk[1]};`}>
						{chunk[0]}
					</span>
				</span>
			{/if}
		{/each}
	</div>
</div>

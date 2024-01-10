<script lang="ts">
	import type { Lint, Token, TokenKind } from '$lib/analysis';
	import { contentToString, lintText, spanContent } from '$lib/analysis';

	export let content: string;

	let lints: Lint[] = [];
	$: lintText(content).then((newLints) => (lints = newLints));

	function reOrgString(text: string): (string | undefined)[] {
		if (text.trim().length == 0) {
			return [''];
		}

		let output: (string | undefined)[] = [];

		for (let chunk of text.split('\n')) {
			if (output.length > 0) {
				output.push(undefined);
			} else {
				output.push(chunk);
			}
		}

		return output;
	}

	// string | [string, string] | null
	$: modified = [
		...lints
			.toSorted((a, b) => a.span.start - b.span.end)
			.map((lint, index, arr) => {
				let prev = arr[index - 1];

				let prevStart = prev?.span.end ?? 0;
				let prevEnd = lint.span.start;

				let prevContent = [];

				if (prevStart != prevEnd) {
					prevContent.push(reOrgString(content.substring(prevStart, prevEnd)));
				}

				let lintContent = [spanContent(lint.span, content), 'red'];

				return [...prevContent.flat(), lintContent];
			})
			.flat(),
		reOrgString(content.substring(lints.at(-1)?.span.end ?? 0))
	];

	$: console.log(modified);
</script>

<div class="grid">
	<div class="p-0 m-0 indent-0" style="grid-row: 1; grid-column: 1">
		{#each modified as chunk}
			{#if chunk == null}
				<br />
			{:else if typeof chunk == 'string'}
				<p class="inline">{chunk}</p>
			{:else}
				<p class="inline" style={`border-bottom: 1px solid ${chunk[1]}`}>{chunk[0]}</p>
			{/if}
		{/each}
	</div>
</div>

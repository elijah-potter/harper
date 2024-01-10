<script lang="ts">
	import type { Lint, Token, TokenKind } from '$lib/analysis';
	import { parseText, contentToString, lintText } from '$lib/analysis';

	export let content: string;

	let tokens: Token[] = [];

	$: parseText(content).then((parsedTokens) => (tokens = parsedTokens));

	/** Convert a given TokenKind to class */
	function kindToClass(tokenKind: TokenKind): string {
		switch (tokenKind.kind) {
			case 'Word':
				return 'bg-red-100';
			case 'Punctuation':
				return 'bg-blue-100';
			case 'Number':
				return 'bg-green-100';
			case 'Space':
				return '';
			case 'Newline':
				return '';
		}
	}
</script>

<div class="grid">
	<div class="p-0 m-0 indent-0" style="grid-row: 1; grid-column: 1">
		{#each tokens as token}
			{#if token.kind.kind === 'Newline'}
				{#each { length: token.kind.value } as _}
					<br />
				{/each}
			{:else}
				<p class={`inline ${kindToClass(token.kind)}`}>{contentToString(token.content)}</p>
			{/if}
		{/each}
	</div>
</div>

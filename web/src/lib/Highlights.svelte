<script lang="ts">
	export let content: string;

	interface ParseResponse {
		tokens: Token[];
	}

	interface Token {
		content: string[];
		kind: TokenKind;
	}

	type TokenKind =
		| { kind: 'Word' }
		| { kind: 'Punctuation'; value: Punctuation }
		| { kind: 'Number'; value: number }
		| { kind: 'Space'; value: number }
		| { kind: 'Newline'; value: number };

	type Punctuation =
		| 'Period'
		| 'Bang'
		| 'Question'
		| 'Colon'
		| 'Semicolon'
		| 'Quote'
		| 'Comma'
		| 'Hyphen'
		| 'Apostrophe'
		| 'OpenSquare'
		| 'CloseSquare'
		| 'OpenRound'
		| 'CloseRound'
		| 'Hash';

	async function parseText(text: string): Promise<Token[]> {
		const req = await fetch(`/parse?text=${encodeURIComponent(text)}`);

		const res: ParseResponse = await req.json();

		return res.tokens;
	}

	function contentToString(content: string[]): string {
		return content.reduce((p, c) => p + c, '');
	}

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
	<div class="whitespace-pre-line p-0 m-0 indent-0" style="grid-row: 1; grid-column: 1">
		{content}
	</div>
</div>

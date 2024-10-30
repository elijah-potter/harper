<script lang="ts">
	import { draw } from 'svelte/transition';

	export let width = '100%';
	export let height = '100%';
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	fill="none"
	viewBox="0 0 24 24"
	stroke-width="3"
	stroke="currentColor"
	class="size-6"
	{width}
	{height}
>
	<path in:draw stroke-linecap="round" stroke-linejoin="round" d="m4.5 12.75 6 6 9-13.5" />
</svg>

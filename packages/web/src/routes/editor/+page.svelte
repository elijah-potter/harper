<script>
	/// This page exists to be embedded via an `iframe`.
	import Editor from '$lib/Editor.svelte';
	import { page } from '$app/stores';

	let content = $page.url.searchParams.get('initialText') ?? '';

	let width = window.innerWidth;

	window.addEventListener('resize', () => {
		width = window.innerWidth;
	});

	$: mobile = width < 1080;
</script>

{#if mobile}
	<div class="flex flex-row items-center justify-center w-full h-full text-center">
		<h2 class="dark:text-white">
			In order to view the Harper editor, please view this page from a device with a larger screen.
		</h2>
	</div>
{:else}
	<Editor {content}></Editor>
{/if}

<script>
	/// This page exists to be embedded via an `iframe`.
	import Editor from '$lib/Editor.svelte';
	import { page } from '$app/stores';

	let content = $page.url.searchParams.get('initialText') ?? '';
</script>

<div class="absolute top-0 left-0 w-full h-full">
	<Editor {content}></Editor>
</div>

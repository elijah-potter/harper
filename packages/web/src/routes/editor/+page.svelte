<script>
	/// This page exists to be embedded via an `iframe`.
	import Editor from '$lib/Editor.svelte';
	import { page } from '$app/stores';

	let content = $page.url.searchParams.get('initialText') ?? '';
</script>

<Editor {content}></Editor>

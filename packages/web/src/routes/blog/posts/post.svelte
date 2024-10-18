<script>
	import '../../../prism-coldark-cold.css';
	import GutterCenter from '$lib/GutterCenter.svelte';
	export let title;
</script>

<div class="h-20" />

<svelte:head>
	<title>{title}</title>
</svelte:head>

<GutterCenter>
	<slot />
</GutterCenter>

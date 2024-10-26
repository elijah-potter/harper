<script lang="ts">
	import '../app.css';
	import GutterCenter from '$lib/GutterCenter.svelte';
	import { DarkMode } from 'flowbite-svelte';
</script>

<link
	href="https://fonts.googleapis.com/css2?family=Atkinson+Hyperlegible:ital,wght@0,400;0,700;1,400;1,700&display=swap"
	rel="stylesheet"
/>
<div class="h-16" />
<div class="absolute top-8 right-8"><DarkMode /></div>

<GutterCenter>
	<slot />
</GutterCenter>

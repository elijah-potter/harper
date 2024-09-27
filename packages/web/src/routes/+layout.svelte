<script lang="ts">
	import '../app.css';
	import GutterCenter from '$lib/GutterCenter.svelte';
	import { DarkMode } from 'flowbite-svelte';
</script>

<div class="h-16" />
<div class="absolute top-8 right-8"><DarkMode /></div>

<GutterCenter>
	<slot />
</GutterCenter>

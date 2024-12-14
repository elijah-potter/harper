<script lang="ts">
	import '../app.css';
	import GutterCenter from '$lib/GutterCenter.svelte';
	import { DarkMode } from 'flowbite-svelte';
	import AutomatticLogo from '$lib/AutomatticLogo.svelte';

	let names = ['Grammar Guru', 'Grammar Checker', 'Grammar Savior'];
	let displayName = names[Math.floor(Math.random() * names.length)];
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

<div class="w-full flex flex-row justify-center">
	<h2 class="dark:text-white flex items-center">
		An
		<div class="inline-block"><AutomatticLogo height="32px" width="140px" /></div>
		{displayName}
	</h2>
</div>

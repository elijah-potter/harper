<script lang="ts">
	import '../app.css';
	import GutterCenter from '$lib/GutterCenter.svelte';
	import { DarkMode } from 'flowbite-svelte';
</script>

<div class="h-16" />
<div class="absolute top-8 right-8"></div>
<div
	class="bg-white dark:bg-gray-800 z-50 fixed top-0 left-0 right-0 p-8 flex items-center justify-center space-x-4 dark:text-white"
>
	<div class="absolute left-8">
		<span>Harper</span>
	</div>
	<a href="/">Home</a>
	<a href="/blog">Blog</a>
	<div class="absolute right-8">
		<DarkMode />
	</div>
</div>

<GutterCenter>
	<slot />
</GutterCenter>

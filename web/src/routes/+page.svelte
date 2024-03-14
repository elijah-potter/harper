<script>
	import Editor from '$lib/Editor.svelte';
	import GutterCenter from '$lib/GutterCenter.svelte';
	import Logo from '$lib/Logo.svelte';
	import NewEditor from '$lib/NewEditor.svelte';
	import { DarkMode } from 'flowbite-svelte';

	let width = window.innerWidth;

	window.addEventListener('resize', () => {
		width = window.innerWidth;
	});
</script>

<div class="h-16" />

<GutterCenter>
	<div class="w-full flex flex-col items-center">
		<Logo width="200px" />
	</div>
	<h1 class="text-5xl font-bold text-center dark:text-white">Hi. I’m Harper.</h1>
	<h2 class="text-3xl text-center dark:text-white">The Grammar Checker for Developers</h2>
	<h2 class="text-2xl font-light italic text-center dark:text-white">
		Private. Free. Unobtrusive.
	</h2>

	<div class="flex flex-row justify-evenly mt-5">
		<a
			href="https://github.com/elijah-potter/harper"
			class="flex flex-row items-center [&>*]:m-2 dark:text-white"
			><img
				width="40"
				height="40"
				class="hover:scale-105 transition-all bg-white rounded-full"
				src="/icons/github.svg"
				alt="Project Repository"
			/>GitHub</a
		>
		<a href="https://elijahpotter.dev" class="flex flex-row items-center [&>*]:m-2 dark:text-white"
			><img
				width="40"
				height="40"
				class="hover:scale-105 transition-all"
				src="/icons/profile.svg"
				alt="Author"
			/>Author</a
		>
	</div>

	<div class="w-full h-[600px] overflow-hidden">
		<NewEditor />
	</div>
</GutterCenter>

<div class="absolute top-8 right-8"><DarkMode /></div>

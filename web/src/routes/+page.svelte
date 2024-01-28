<script>
	import Editor from '$lib/Editor.svelte';
	import Graph from '$lib/Graph.svelte';

	let width = window.innerWidth;

	window.addEventListener('resize', () => {
		width = window.innerWidth;
	});
</script>

<div class="w-full h-screen flex flex-col items-center m-0 p-0">
	<div class="p-8 flex-initial">
		<h1 class="text-5xl font-bold text-center">Hi. I'm Harper.</h1>
		<h2 class="text-3xl text-center">The Grammar Checker for Artists</h2>
		<h2 class="text-2xl font-light italic text-center">Private. Free. Unobtrusive.</h2>
		<div class="flex flex-row justify-evenly mt-5">
			<a href="https://github.com/chilipepperhott/harper" class="hover:scale-105 transition-all"
				><img width="40" height="40" src="/icons/github.svg" alt="Project Repository" /></a
			>
			<a href="https://elijahpotter.dev" class="hover:scale-105 transition-all"
				><img width="40" height="40" src="/icons/profile.svg" alt="Author" /></a
			>
		</div>
	</div>
	<div class="w-full 2xl:w-3/4 flex-shrink flex-grow overflow-hidden">
		{#if width > 768}
			<Editor />
		{:else}
			<p class="italic text-lg w-full text-center">
				Open this page on a bigger screen to start checking your work.
			</p>
		{/if}
	</div>
</div>

<script>
	import Editor from '$lib/Editor.svelte';
</script>

<div class="w-full h-screen flex flex-col items-center justify-evenly">
	<div>
		<h1 class="text-5xl font-bold text-center">Hi! I'm Audrey</h1>
		<h2 class="text-3xl italic text-center">The Grammar Checker for Artists</h2>
		<div class="flex flex-row justify-evenly mt-5">
			<a href="https://github.com/chilipepperhott/lt-rs" class="hover:scale-105 transition-all"
				><img width="40" height="40" src="/icons/github.svg" alt="Project Repository" /></a
			>
			<a href="https://elijahpotter.dev" class="hover:scale-105 transition-all"
				><img width="40" height="40" src="/icons/profile.svg" alt="Author" /></a
			>
		</div>
	</div>
	<div class="w-full 2xl:w-3/4 h-1/2">
		<Editor />
	</div>
</div>


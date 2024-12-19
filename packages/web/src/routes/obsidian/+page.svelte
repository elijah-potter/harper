<script>
	import GutterCenter from '$lib/GutterCenter.svelte';
	import Section from '$lib/Section.svelte';
	import GitHubLogo from '$lib/GitHubLogo.svelte';
</script>

<GutterCenter>
	<h1 class="text-5xl font-bold text-center">Harper in Obsidian</h1>
	<h2 class="text-3xl text-center">
		A Grammar Checker, Now for <b>Everyone</b>
	</h2>

	<div class="flex flex-row justify-evenly mt-5">
		<a
			href="https://github.com/automattic/harper-obsidian-plugin"
			class="flex flex-row items-center [&>*]:m-2 hover:scale-105 transition-all"
			><GitHubLogo width="40px" height="40px" />GitHub</a
		>
		<a href="/" class="flex flex-row items-center [&>*]:m-2 hover:scale-105 transition-all"
			><span class="text-4xl font-extrabold">?</span>What is Harper?</a
		>
	</div>

	<Section swapped>
		<span slot="title">How does it work?</span>
		<span slot="subtitle">
			Unlike LanguageTool, Harper compiles to WebAssembly so your information never leaves Obsidian. <br
			/><b>Your vault should be exactly that: secure.</b>
		</span>
		<img
			src="/images/obsidian_screenshot.webp"
			class="rounded-2xl"
			alt="A screenshot of the Harper Obsidian plugin at work."
		/>
	</Section>

	<Section>
		<span slot="title">Where can I get it?</span>

		<span slot="subtitle"
			>Easy. Go to Obsidian's settings, click "Community Plugins", and enable them.
			<br /><br />Next, click "Browse" and search for "Harper".
			<br /><br /> Finally, go back and enable the plugin.
		</span>

		<img
			src="/images/obsidian_install_screenshot.webp"
			class="rounded-2xl"
			alt="A screenshot of the Harper Obsidian installation."
		/>
	</Section>
</GutterCenter>

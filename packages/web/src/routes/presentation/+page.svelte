<script lang="ts">
	import 'reveal.js/dist/reveal.css';
	import 'reveal.js/dist/theme/serif.css';
	import Reveal from 'reveal.js';
	import { onMount } from 'svelte';
	import Logo from '$lib/Logo.svelte';

	onMount(() => {
		let deck = new Reveal();
		deck.initialize();
	});
</script>

<div class="reveal">
	<div class="slides">
		<section>
			<Logo width="50%" />
			<h1>Harper</h1>
			<h2>The Grammar Checker for Developers</h2>
		</section>
		<section>
			<h1>The Problem:</h1>
			<h1 class="fragment fade-in"><strong>Grammarly</strong></h1>
		</section>
		<section>
			<h3>It is a privacy nightmare.</h3>
			<p class="fragment fade-in">
				Every request gets sent to their servers and stored <strong>forever.</strong>
			</p>
		</section>
		<section>
			<h3>It is <i>slow.</i></h3>
			<p class="fragment fade-in">Grammarly requests can take as many as 4000ms to resolve.*</p>
		</section>
		<section>
			<h3>LanguageTool</h3>
			<p class="fragment fade-in">You can self-host it, but...</p>
			<p class="fragment fade-in">there are memory issues.</p>
		</section>
		<section>
			<Logo width="40%" />
			<h2>Let me introduce:</h2>
			<h1>Harper</h1>
		</section>
		<section>
			<h2>The mission:</h2>
			<p>Create a grammar checker of the same caliber as Grammarly and LanguageTool.</p>
			<p class="fragment fade-in"><strong>Without the cruft.</strong></p>
		</section>
		<section>
			<h2>That means:</h2>
			<ul>
				<li class="fragment fade-in">It must be completely private.</li>
				<li class="fragment fade-in">It must encourage painless revision <i>by a human</i>.</li>
				<li class="fragment fade-in">It has to get out of the way.</li>
			</ul>
		</section>
		<section>
			<h2>Harper (right now)</h2>
			<ul>
				<li class="fragment fade-in">Runs on-device.</li>
				<li class="fragment fade-in">
					The entire grammar checker fits under 500kb (including datasets).
				</li>
				<li class="fragment fade-in">Completes checking on large documents in under 10-20ms.</li>
			</ul>
		</section>
		<section>
			<h2>Linters</h2>
			<ul>
				<li>"a" v.s. "an" usage.</li>
				<li>Spell-checking.</li>
				<li>Accidental word repetition (e.x. "We ate at <i>the the</i> cafe").</li>
				<li>Extraneous spaces.</li>
				<li>More than 100 curated patterns.</li>
				<li>Run-on sentances.</li>
			</ul>
		</section>
		<section>
			<h2>Availability</h2>
			<h3>Full Support:</h3>
			<ul>
				<li>Neovim (LSP)</li>
				<li>Web (WebAssembly)</li>
			</ul>
			<h3>Early Access (talk to me)</h3>
			<ul><li>Obsidian (CodeMirror + WebAssembly)</li></ul>
			<h3>Planned:</h3>
			<ul>
				<li>JetBrains products (LSP)</li>
				<li>Visual Studio Code (LSP)</li>
			</ul>
		</section>
		<section>
			<h2>Try It!</h2>
			<p>
				Go to <a href="https://writewithharper.com">https://writewithharper.com</a> on a laptop.
				<img
					alt="The QR code for the website."
					src=" https://api.qrserver.com/v1/create-qr-code/?size=300x300&data=https://writewithharper.com"
				/>
			</p>
		</section>
	</div>
</div>

<style>
	.reveal * {
		text-align: left;
	}
</style>

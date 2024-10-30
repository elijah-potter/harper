<script lang="ts">
	import IntersectionObserver from 'svelte-intersection-observer';
	let data = new Map<string, number>();
	data.set('Harper', 10);
	data.set('LanguageTool', 650);
	data.set('Grammarly', 4000);

	let maxW = 0;

	for (let val of data.values()) {
		if (val > maxW) {
			maxW = val;
		}
	}

	let scaledData = new Map();

	for (let [key, val] of data.entries()) {
		scaledData.set(key, val / maxW);
	}

	let els: Record<string, HTMLElement> = {};

	function expand(node: HTMLElement, { width, duration }: { width: number; duration: number }) {
		return {
			duration,
			css: (t: number) => {
				return `width: ${width * 100 * t}%;`;
			}
		};
	}
</script>

<div class="flex flex-col justify-start w-full h-full">
	{#each scaledData as [name, width] (name)}
		<IntersectionObserver element={els[name]} let:intersecting>
			<div bind:this={els[name]}>
				{#if intersecting}
					<div class="relative w-full h-full">
						{name} - {width * maxW} ms
						<div
							class="rounded transition-all mb-4 p-2 font-bold bg-gray-200"
							in:expand={{ width, duration: width * maxW }}
							style={`width: ${width * 100}%;`}
						></div>
					</div>
				{/if}
			</div>
		</IntersectionObserver>
	{/each}
</div>

<script lang="ts">
	export let swapped = false;
	export let noChild = false;
</script>

<section
	class={`py-6 md:py-12 w-full px-4 md:px-6 grid ${noChild ? 'sm:grid-cols-1' : 'sm:grid-cols-2'} gap-8 dark:text-white`}
>
	{#if swapped && !noChild}
		<div>
			<slot />
		</div>
	{/if}

	<div class={`justify-center space-y-2`}>
		<h2 class="font-bold">
			<slot name="title" />
		</h2>
		<p class="text-gray-500 dark:text-gray-400">
			<slot name="subtitle" />
		</p>
	</div>

	{#if !swapped && !noChild}
		<div>
			<slot />
		</div>
	{/if}
</section>

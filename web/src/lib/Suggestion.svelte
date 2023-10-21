<script lang="ts">
	import { Badge, Card } from 'flowbite-svelte';
	import { startCase } from 'lodash-es';

	type Category = 'grammar' | 'spelling';

	export let category: Category;
	export let title: string;

	function categoryToColor(category: Category): string {
		switch (category) {
			case 'grammar':
				return 'green';
			case 'spelling':
				return 'purple';
		}
	}
</script>

<Card class="m-5 hover:translate-x-3 transition-all">
	<div class="flex flex-row justify-between">
		<h3 class="font-bold">{title}</h3>
		<Badge color={categoryToColor(category)}>{startCase(category)}</Badge>
	</div>
	<slot />
</Card>

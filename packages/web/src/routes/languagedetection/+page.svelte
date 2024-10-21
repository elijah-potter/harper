<script lang="ts">
	import { isLikelyEnglish } from '$lib/analysis';
	import { Textarea, Select } from 'flowbite-svelte';
	import demoText from '../../../../../demo.md?raw';

	let isEnglish: boolean | null = null;
	let text = '';

	$: isLikelyEnglish(text).then((v) => (isEnglish = v));

	$: color = isEnglish == null ? '' : isEnglish ? 'bg-green-100' : 'bg-red-100';

	let templates = [
		{
			name: 'Java Code',
			value: `public class Main {
  public static void main(String[] args) {
    System.out.println("Hello World");
  }
}`
		},
		{ name: 'Poor Grammar', value: demoText },
		{
			name: 'Chinese Lorem Ipsum',
			value:
				'食棵支每躲種。奶象打星爪子二細喜才記行在發像原斤！頁固點子衣點豆看身蝴看苗急午公何足，筆娘經色蝶行元香也要。麻了綠尼固世，色北書目登功；因告黑。'
		}
	];

	function selectChange(e: Event) {
		text = (e.target as HTMLSelectElement).value;
	}
</script>

<div class="[&>*]:mt-12 p-4">
	<h1 class="text-2xl">Language Detection Demo</h1>
	<p>
		This is demonstration of Harper's ability to quickly (under 1 ms for large documents) determine
		whether a provided document is intended to be English. The algorithm is flexible to bad grammar.
		<br />
		Since this is used to redact commented-out code, it airs on the side of producing false-positives.
	</p>

	<Select items={templates} on:change={selectChange} />

	<Textarea
		rows={8}
		class={color}
		bind:value={text}
		placeholder="Is your text supposed to be English?"
	/>
</div>

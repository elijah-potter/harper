import flowbitePlugin from 'flowbite/plugin';

export default {
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		'../node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}'
	],
	plugins: [flowbitePlugin],
	darkMode: 'class',
	theme: {
		extend: {
			colors: {
				primary: {
					900: '#133f71',
					800: '#355280',
					700: '#50658f',
					600: '#69799f',
					500: '#818eae',
					400: '#9aa4be',
					300: '#b3bace',
					200: '#ccd0de',
					100: '#e5e7ef',
					50: '#ffffff'
				}
			}
		}
	}
};

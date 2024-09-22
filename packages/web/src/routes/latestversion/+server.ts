import { GithubClient } from '$lib/GitHubClient';

export async function GET() {
	const latestVersion = await GithubClient.getLatestReleaseFromCache('elijah-potter', 'harper');

	if (latestVersion == null) {
		throw new Error('Unable to get latest version.');
	}

	console.log(`Received request for latest version. Responding with ${latestVersion}`);

	return new Response(latestVersion, {
		headers: {
			['Cache-Control']: 'no-cache'
		}
	});
}

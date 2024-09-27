export class GithubClient {
	/// Map of string -> [content, expiration time]
	private static versionCache: Map<string, [string, number]> = new Map();

	public static async getLatestReleaseFromCache(
		repoOwner: string,
		repoName: string
	): Promise<string | null> {
		const key = `${repoOwner}/${repoName}`;

		const val = this.versionCache.get(key);

		if (val == null) {
			const updatedValue = await this.getLatestRelease(repoOwner, repoName);
			this.versionCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

		const [value, expiry] = val;

		if (expiry - Date.now() < 0) {
			this.versionCache.delete(key);
			const updatedValue = await this.getLatestRelease(repoOwner, repoName);
			this.versionCache.set(key, [updatedValue, Date.now() + 3600 * 3000]);
			return updatedValue;
		}

		return value;
	}

	public static async getLatestRelease(repoOwner: string, repoName: string): Promise<string> {
		const resp = await fetch(
			`https://api.github.com/repos/${encodeURIComponent(repoOwner)}/${encodeURIComponent(repoName)}/releases/latest`,
			{
				headers: {
					['ContentType']: 'application/json'
				}
			}
		);

		const body = await resp.json();

		return body.name;
	}
}

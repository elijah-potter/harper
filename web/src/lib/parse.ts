export async function highlightText(text: string): Promise<string> {
	const req = await fetch(`/parse?text=${encodeURIComponent(text)}`);

	return req.text();
}

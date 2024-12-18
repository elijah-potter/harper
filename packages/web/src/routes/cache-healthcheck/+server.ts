export async function GET() {
	return new Response('OK', {
		headers: {
			['Cache-Control']: 'no-cache'
		}
	});
}

import Jasmine from 'jasmine';
import path from 'node:path';

export async function run(): Promise<void> {
	const jasmine = new Jasmine();
	jasmine.exitOnCompletion = false;
	jasmine.loadConfig({
		spec_dir: path.relative(process.cwd(), __dirname),
		spec_files: ['*.test.js'],
		random: false
	});

	const result = await jasmine.execute();
	if (result.overallStatus !== 'passed') {
		throw new Error('Tests failed');
	}
}

import { expect, test } from 'vitest';
import { SuggestionKind as WasmSuggestionKind } from 'wasm';
import { SuggestionKind } from './main';

test('Wasm and JS SuggestionKinds agree', async () => {
	expect(SuggestionKind.Remove).toBe(WasmSuggestionKind.Remove);
	expect(SuggestionKind.Replace).toBe(WasmSuggestionKind.Replace);
});

use std::sync::Mutex;

use harper_core::{remove_overlaps, Document, FullDictionary, LintSet, Linter};
use once_cell::sync::Lazy;
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

static LINTER: Lazy<Mutex<LintSet>> =
    Lazy::new(|| Mutex::new(LintSet::new().with_standard(FullDictionary::create_from_curated())));

/// Create the serializer that preserves types across the JavaScript barrier
fn glue_serializer() -> serde_wasm_bindgen::Serializer {
    serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true)
}

/// Setup the WebAssembly module's logging.
///
/// Not strictly necessary for anything to function, but makes bug-hunting less painful.
#[wasm_bindgen(start)]
pub fn setup() {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();
}

#[wasm_bindgen]
pub fn lint(text: String) -> Vec<JsValue> {
    let document = Document::new_markdown(&text);

    let mut lints = LINTER.lock().unwrap().lint(&document);

    remove_overlaps(&mut lints);

    lints
        .into_iter()
        .map(|lint| lint.serialize(&glue_serializer()).unwrap())
        .collect()
}

#[wasm_bindgen]
pub fn parse(text: String) -> Vec<JsValue> {
    let document = Document::new_markdown(&text);

    document
        .fat_tokens()
        .map(|lint| lint.serialize(&glue_serializer()).unwrap())
        .collect()
}

#[wasm_bindgen]
pub fn apply_suggestion(
    text: String,
    span: JsValue,
    suggestion: JsValue,
) -> Result<String, String> {
    let span = serde_wasm_bindgen::from_value(span).map_err(|e| e.to_string())?;
    let suggestion = serde_wasm_bindgen::from_value(suggestion).map_err(|e| e.to_string())?;

    let mut document = Document::new_markdown(&text);
    document.apply_suggestion(&suggestion, span);

    Ok(document.get_full_string())
}

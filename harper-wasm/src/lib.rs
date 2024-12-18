#![doc = include_str!("../README.md")]

use std::convert::Into;
use std::sync::Mutex;

use harper_core::language_detection::is_doc_likely_english;
use harper_core::linting::{LintGroup, LintGroupConfig, Linter};
use harper_core::parsers::{IsolateEnglish, Markdown, PlainEnglish};
use harper_core::{remove_overlaps, Document, FullDictionary, Lrc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

static LINTER: Lazy<Mutex<LintGroup<Lrc<FullDictionary>>>> = Lazy::new(|| {
    Mutex::new(LintGroup::new(
        LintGroupConfig::default(),
        FullDictionary::curated(),
    ))
});

macro_rules! make_serialize_fns_for {
    ($name:ident) => {
        #[wasm_bindgen]
        impl $name {
            pub fn to_json(&self) -> String {
                serde_json::to_string(&self).unwrap()
            }

            pub fn from_json(json: String) -> Result<Self, String> {
                serde_json::from_str(&json).map_err(|err| err.to_string())
            }
        }
    };
}

make_serialize_fns_for!(Suggestion);
make_serialize_fns_for!(Lint);
make_serialize_fns_for!(Span);

/// Setup the WebAssembly module's logging.
///
/// Not strictly necessary for anything to function, but makes bug-hunting less
/// painful.
#[wasm_bindgen(start)]
pub fn setup() {
    console_error_panic_hook::set_once();

    // If `setup` gets called more than once, we want to allow this error to fall through.
    let _ = tracing_wasm::try_set_as_global_default();
}

/// Helper method to quickly check if a plain string is likely intended to be English
#[wasm_bindgen]
pub fn is_likely_english(text: String) -> bool {
    let document = Document::new_plain_english_curated(&text);
    is_doc_likely_english(&document, &FullDictionary::curated())
}

/// Helper method to remove non-English text from a plain English document.
#[wasm_bindgen]
pub fn isolate_english(text: String) -> String {
    let dict = FullDictionary::curated();

    let document = Document::new_curated(
        &text,
        &mut IsolateEnglish::new(Box::new(PlainEnglish), dict.clone()),
    );

    document.to_string()
}

#[wasm_bindgen]
pub fn get_lint_config_as_object() -> JsValue {
    let linter = LINTER.lock().unwrap();
    serde_wasm_bindgen::to_value(&linter.config).unwrap()
}

#[wasm_bindgen]
pub fn set_lint_config_from_object(object: JsValue) -> Result<(), String> {
    let mut linter = LINTER.lock().unwrap();
    linter.config = serde_wasm_bindgen::from_value(object).map_err(|v| v.to_string())?;
    Ok(())
}

/// Perform the configured linting on the provided text.
#[wasm_bindgen]
pub fn lint(text: String) -> Vec<Lint> {
    let source: Vec<_> = text.chars().collect();
    let source = Lrc::new(source);

    // TODO: Have a way to configure the markdown parser
    let document = Document::new_from_vec(
        source.clone(),
        &mut Markdown::default(),
        &FullDictionary::curated(),
    );

    let mut lints = LINTER.lock().unwrap().lint(&document);

    remove_overlaps(&mut lints);

    lints
        .into_iter()
        .map(|l| Lint::new(l, source.to_vec()))
        .collect()
}

#[wasm_bindgen]
pub fn apply_suggestion(
    text: String,
    span: Span,
    suggestion: &Suggestion,
) -> Result<String, String> {
    let mut source: Vec<_> = text.chars().collect();
    let span: harper_core::Span = span.into();

    suggestion.inner.apply(span, &mut source);

    Ok(source.iter().collect())
}

#[derive(Debug, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Suggestion {
    inner: harper_core::linting::Suggestion,
}

#[derive(Debug, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum SuggestionKind {
    Replace = 0,
    Remove = 1,
}

#[wasm_bindgen]
impl Suggestion {
    pub(crate) fn new(inner: harper_core::linting::Suggestion) -> Self {
        Self { inner }
    }

    /// Get the text that is going to replace error.
    /// If [`Self::kind`] is `SuggestionKind::Remove`, this will return an empty
    /// string.
    pub fn get_replacement_text(&self) -> String {
        match &self.inner {
            harper_core::linting::Suggestion::Remove => "".to_string(),
            harper_core::linting::Suggestion::ReplaceWith(chars) => chars.iter().collect(),
        }
    }

    pub fn kind(&self) -> SuggestionKind {
        match &self.inner {
            harper_core::linting::Suggestion::Remove => SuggestionKind::Remove,
            harper_core::linting::Suggestion::ReplaceWith(_) => SuggestionKind::Replace,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[wasm_bindgen]
pub struct Lint {
    inner: harper_core::linting::Lint,
    source: Vec<char>,
}

#[wasm_bindgen]
impl Lint {
    pub(crate) fn new(inner: harper_core::linting::Lint, source: Vec<char>) -> Self {
        Self { inner, source }
    }

    /// Get the content of the source material pointed to by [`Self::span`]
    pub fn get_problem_text(&self) -> String {
        self.inner.span.get_content_string(&self.source)
    }

    /// Get a string representing the general category of the lint.
    pub fn lint_kind(&self) -> String {
        self.inner.lint_kind.to_string()
    }

    pub fn suggestion_count(&self) -> usize {
        self.inner.suggestions.len()
    }

    pub fn suggestions(&self) -> Vec<Suggestion> {
        self.inner
            .suggestions
            .iter()
            .map(|s| Suggestion::new(s.clone()))
            .collect()
    }

    pub fn span(&self) -> Span {
        self.inner.span.into()
    }

    pub fn message(&self) -> String {
        self.inner.message.clone()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[wasm_bindgen]
impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        Into::<harper_core::Span>::into(*self).len()
    }
}

impl From<Span> for harper_core::Span {
    fn from(value: Span) -> Self {
        harper_core::Span::new(value.start, value.end)
    }
}

impl From<harper_core::Span> for Span {
    fn from(value: harper_core::Span) -> Self {
        Span::new(value.start, value.end)
    }
}

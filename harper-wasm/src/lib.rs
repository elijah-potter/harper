#![doc = include_str!("../README.md")]

use std::sync::Mutex;

use harper_core::linting::{LintGroup, LintGroupConfig, Linter};
use harper_core::parsers::Markdown;
use harper_core::{remove_overlaps, Document, FullDictionary, Lrc};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

static LINTER: Lazy<Mutex<LintGroup<Lrc<FullDictionary>>>> = Lazy::new(|| {
    Mutex::new(LintGroup::new(
        LintGroupConfig::default(),
        FullDictionary::curated(),
    ))
});

/// Setup the WebAssembly module's logging.
///
/// Not strictly necessary for anything to function, but makes bug-hunting less
/// painful.
#[wasm_bindgen(start)]
pub fn setup() {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();
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

    let document =
        Document::new_from_vec(source.clone(), &mut Markdown, &FullDictionary::curated());

    let mut lints = LINTER.lock().unwrap().lint(&document);

    remove_overlaps(&mut lints);

    lints
        .into_iter()
        .map(|l| Lint::new(l, source.clone()))
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

    match &suggestion.inner {
        harper_core::linting::Suggestion::ReplaceWith(chars) => {
            // Avoid allocation if possible
            if chars.len() == span.len() {
                for (index, c) in chars.iter().enumerate() {
                    source[index + span.start] = *c
                }
            } else {
                let popped = source.split_off(span.start);

                source.extend(chars);
                source.extend(popped.into_iter().skip(span.len()));
            }
        }
        harper_core::linting::Suggestion::Remove => {
            for i in span.end..source.len() {
                source[i - span.len()] = source[i];
            }

            source.truncate(source.len() - span.len());
        }
    }

    Ok(source.iter().collect())
}

#[wasm_bindgen]
pub struct Suggestion {
    inner: harper_core::linting::Suggestion,
}

#[wasm_bindgen]
pub enum SuggestionKind {
    Replace,
    Remove,
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

#[wasm_bindgen]
pub struct Lint {
    inner: harper_core::linting::Lint,
    source: Lrc<Vec<char>>,
}

#[wasm_bindgen]
impl Lint {
    pub(crate) fn new(inner: harper_core::linting::Lint, source: Lrc<Vec<char>>) -> Self {
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

use serde::{Deserialize, Serialize};

use crate::{document::Document, span::Span};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lint {
    pub span: Span,
    pub lint_kind: LintKind,
    pub suggestions: Vec<Suggestion>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LintKind {
    Spelling,
    Capitalization,
    UnmatchedQuote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Suggestion {
    ReplaceWith(Vec<char>),
}

pub type Linter = fn(document: &Document) -> Vec<Lint>;

use std::fmt::Display;

use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::{document::Document, span::Span};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lint {
    pub span: Span,
    pub lint_kind: LintKind,
    pub suggestions: Vec<Suggestion>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Is, Default)]
pub enum LintKind {
    Spelling,
    Capitalization,
    Formatting,
    Repetition,
    Readability,
    #[default]
    Miscellaneous,
}

#[derive(Debug, Clone, Serialize, Deserialize, Is)]
pub enum Suggestion {
    ReplaceWith(Vec<char>),
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suggestion::ReplaceWith(with) => {
                write!(f, "Replace with: “{}”", with.iter().collect::<String>())
            }
        }
    }
}

#![allow(dead_code)]

mod document;
mod linting;
mod parsing;
mod span;
mod spell;

pub use document::Document;
pub use linting::run_lint_set;
pub use linting::LintSet;
pub use linting::{Lint, LintKind, Suggestion};
pub use parsing::{FatToken, Punctuation, Token, TokenKind};
pub use span::Span;
pub use spell::Dictionary;
pub use spell::{suggest_correct_spelling, suggest_correct_spelling_str};

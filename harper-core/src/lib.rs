#![allow(dead_code)]

mod document;
mod linting;
mod parsing;
mod span;
mod spell;

pub use document::Document;
pub use linting::LintSet;
pub use linting::{Lint, LintKind, Linter, Suggestion};
pub use parsing::{FatToken, Punctuation, Token, TokenKind};
pub use span::Span;
pub use spell::Dictionary;

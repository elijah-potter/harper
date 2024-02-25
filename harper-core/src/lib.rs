#![allow(dead_code)]

mod document;
mod lexing;
mod linting;
pub mod parsers;
mod span;
mod spell;
mod token;

pub use document::Document;
pub use linting::{Lint, LintKind, LintSet, Linter, Suggestion};
pub use span::Span;
pub use spell::{Dictionary, FullDictionary, MergedDictionary};
pub use token::{FatToken, Punctuation, Token, TokenKind, TokenStringExt};

/// A utility function that removes overlapping lints in a vector,
/// keeping the more important ones.
///
/// Note: this function will change the ordering of the lints.
pub fn remove_overlaps(lints: &mut Vec<Lint>) {
    if lints.len() < 2 {
        return;
    }

    lints.sort_by_key(|l| l.span.start);

    let mut remove_indices = Vec::new();

    for i in 0..lints.len() - 1 {
        let cur = &lints[i];
        let next = &lints[i + 1];

        if cur.span.overlaps_with(next.span) {
            // Remember, lower priority means higher importance.
            if next.priority < cur.priority {
                remove_indices.push(i);
            } else {
                remove_indices.push(i + 1);
            }
        }
    }

    let mut index = 0;
    lints.retain(|_| {
        index += 1;
        !remove_indices.contains(&(index - 1))
    })
}

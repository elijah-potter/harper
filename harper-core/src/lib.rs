#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod char_ext;
mod char_string;
mod document;
pub mod language_detection;
mod lexing;
pub mod linting;
mod mask;
pub mod parsers;
pub mod patterns;
mod punctuation;
mod span;
mod spell;
mod sync;
mod token;
mod vec_ext;
mod word_metadata;

use std::collections::VecDeque;

pub use char_string::{CharString, CharStringExt};
pub use document::Document;
use linting::Lint;
pub use mask::{Mask, Masker};
pub use punctuation::{Punctuation, Quote};
pub use span::Span;
pub use spell::{Dictionary, FullDictionary, MergedDictionary};
pub use sync::Lrc;
pub use token::{FatToken, Token, TokenKind, TokenStringExt};
pub use vec_ext::VecExt;
pub use word_metadata::{Tense, WordMetadata};

/// A utility function that removes overlapping lints in a vector,
/// keeping the more important ones.
///
/// Note: this function will change the ordering of the lints.
pub fn remove_overlaps(lints: &mut Vec<Lint>) {
    if lints.len() < 2 {
        return;
    }

    lints.sort_by_key(|l| l.span.start);

    let mut remove_indices = VecDeque::new();

    for i in 0..lints.len() - 1 {
        let cur = &lints[i];
        let next = &lints[i + 1];

        if cur.span.overlaps_with(next.span) {
            // Remember, lower priority means higher importance.
            if next.priority < cur.priority {
                remove_indices.push_back(i);
            } else {
                remove_indices.push_back(i + 1);
            }
        }
    }

    lints.remove_indices(remove_indices);
}

#[cfg(test)]
mod tests {
    use crate::{
        linting::{LintGroup, LintGroupConfig, Linter},
        remove_overlaps, Document, FullDictionary,
    };

    #[test]
    fn keeps_space_lint() {
        let doc = Document::new_plain_english_curated("Ths  tet");

        let lint_config = LintGroupConfig {
            spell_check: Some(true),
            spaces: Some(true),
            ..LintGroupConfig::none()
        };
        let mut linter = LintGroup::new(lint_config, FullDictionary::curated());

        let mut lints = linter.lint(&doc);

        dbg!(&lints);
        remove_overlaps(&mut lints);
        dbg!(&lints);

        assert_eq!(lints.len(), 3);
    }
}

#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod document;
pub mod language_detection;
mod lexing;
pub mod linting;
mod mask;
pub mod parsers;
pub mod patterns;
mod spell;

use std::collections::VecDeque;

pub use document::Document;
use harper_data::{CharString, CharStringExt, Span, VecExt, WordMetadata};
use linting::Lint;
pub use mask::{Mask, Masker};
pub use spell::{Dictionary, FstDictionary, FullDictionary, MergedDictionary};

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
    use crate::linting::{LintGroup, LintGroupConfig, Linter};
    use crate::{remove_overlaps, Document, FstDictionary};

    #[test]
    fn keeps_space_lint() {
        let doc = Document::new_plain_english_curated("Ths  tet");

        let lint_config = LintGroupConfig {
            spell_check: Some(true),
            spaces: Some(true),
            ..LintGroupConfig::none()
        };
        let mut linter = LintGroup::new(lint_config, FstDictionary::curated());

        let mut lints = linter.lint(&doc);

        dbg!(&lints);
        remove_overlaps(&mut lints);
        dbg!(&lints);

        assert_eq!(lints.len(), 3);
    }
}

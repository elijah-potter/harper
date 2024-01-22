use itertools::Itertools;

use crate::{document::Document, parsing::TokenStringExt, Dictionary};

use super::lint::Suggestion;
use super::{Lint, LintKind, Linter};

#[derive(Debug, Clone, Copy, Default)]
pub struct SentenceCapitalization;

impl Linter for SentenceCapitalization {
    /// A linter that checks to make sure the first word of each sentence is capitalized.
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.sentences() {
            if let Some(first_word) = sentence.first_word() {
                let letters = document.get_span_content(first_word.span);

                if let Some(first_letter) = letters.first() {
                    if first_letter.is_alphabetic() && !first_letter.is_uppercase() {
                        lints.push(Lint {
                            span: first_word.span.with_len(1),
                            lint_kind: LintKind::Capitalization,
                            suggestions: vec![Suggestion::ReplaceWith(
                                first_letter.to_uppercase().collect_vec(),
                            )],
                            message: "This sentance does not start with a capital letter"
                                .to_string(),
                        })
                    }
                }
            }
        }

        lints
    }
}

use super::{Lint, LintKind, Linter};
use crate::token::TokenStringExt;
use crate::{Document, Span};

/// Detect and warn that the sentence is too long.
#[derive(Debug, Clone, Copy, Default)]
pub struct LongSentences;

impl Linter for LongSentences {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut output = Vec::new();

        for sentence in document.iter_sentences() {
            let word_count = sentence.iter_words().count();

            if word_count > 40 {
                output.push(Lint {
                    span: Span::new(sentence[0].span.start, sentence.last().unwrap().span.end),
                    lint_kind: LintKind::Readability,
                    message: format!("This sentence is {} words long.", word_count),
                    ..Default::default()
                })
            }
        }

        output
    }
}

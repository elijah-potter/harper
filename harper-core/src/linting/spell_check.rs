use super::{Lint, LintKind, Linter};
use crate::{
    document::Document,
    spell::{suggest_correct_spelling, Dictionary},
};

use super::lint::Suggestion;

pub struct SpellCheck {
    dictionary: Dictionary,
}

impl SpellCheck {
    pub fn new(dictionary: Dictionary) -> Self {
        Self { dictionary }
    }
}

impl Linter for SpellCheck {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.words() {
            let word_chars = document.get_span_content(word.span);
            if self.dictionary.contains_word(word_chars) {
                continue;
            }

            let mut possibilities = suggest_correct_spelling(word_chars, 10, 3, &self.dictionary);

            // People more likely to misspell words by omission, so show the longest words first.
            possibilities.sort_by_key(|p| usize::MAX - p.len());

            let suggestions = possibilities
                .into_iter()
                .map(|word| Suggestion::ReplaceWith(word.to_vec()));

            lints.push(Lint {
                span: word.span,
                lint_kind: LintKind::Spelling,
                suggestions: suggestions.collect(),
                message: format!(
                    "Did you mean to spell “{}” this way?",
                    document.get_span_content_str(word.span)
                ),
            })
        }

        lints
    }
}

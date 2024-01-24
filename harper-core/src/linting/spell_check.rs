use hashbrown::HashMap;

use super::{Lint, LintKind, Linter};
use crate::{
    document::Document,
    spell::{suggest_correct_spelling, Dictionary},
};

use super::lint::Suggestion;

pub struct SpellCheck {
    dictionary: Dictionary,
    word_cache: HashMap<Vec<char>, Vec<Vec<char>>>,
}

impl SpellCheck {
    pub fn new(dictionary: Dictionary) -> Self {
        Self {
            dictionary,
            word_cache: HashMap::new(),
        }
    }
}

impl SpellCheck {
    fn cached_suggest_correct_spelling(&mut self, word: &[char]) -> Vec<Vec<char>> {
        let word = word.to_vec();

        self.word_cache
            .entry(word.clone())
            .or_insert_with(|| {
                suggest_correct_spelling(&word, 100, 3, &self.dictionary)
                    .into_iter()
                    .map(|v| v.to_vec())
                    .collect()
            })
            .clone()
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

            let mut possibilities = self.cached_suggest_correct_spelling(word_chars);

            if possibilities.len() > 3 {
                possibilities.resize_with(3, || panic!());
            }

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

use harper_core::Document;
use harper_data::{CharString, TokenStringExt};
use harper_spell::{suggest_correct_spelling, Dictionary};
use hashbrown::HashMap;
use smallvec::ToSmallVec;

use super::lint::Suggestion;
use super::{Lint, LintKind, Linter};

pub struct SpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
    word_cache: HashMap<CharString, Vec<CharString>>,
}

impl<T: Dictionary> SpellCheck<T> {
    pub fn new(dictionary: T) -> Self {
        Self {
            dictionary,
            word_cache: HashMap::new(),
        }
    }
}

impl<T: Dictionary> SpellCheck<T> {
    fn cached_suggest_correct_spelling(&mut self, word: &[char]) -> Vec<CharString> {
        let word = word.to_smallvec();

        self.word_cache
            .entry(word.clone())
            .or_insert_with(|| {
                // Back off until we find a match.
                let mut suggestions = Vec::new();
                let mut dist = 2;

                while suggestions.is_empty() && dist < 5 {
                    suggestions = suggest_correct_spelling(&word, 100, dist, &self.dictionary)
                        .into_iter()
                        .map(|v| v.to_smallvec())
                        .collect();

                    dist += 1;
                }

                suggestions
            })
            .clone()
    }
}

impl<T: Dictionary> Linter for SpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let word_chars = document.get_span_content(word.span);
            if self.dictionary.contains_word(word_chars) {
                continue;
            }

            let mut possibilities = self.cached_suggest_correct_spelling(word_chars);

            if possibilities.len() > 3 {
                possibilities.resize_with(3, || panic!());
            }

            // If the misspelled word is capitalized, capitalize the results too.
            if let Some(mis_f) = word_chars.first() {
                if mis_f.is_uppercase() {
                    for sug_f in possibilities.iter_mut().filter_map(|w| w.first_mut()) {
                        *sug_f = sug_f.to_uppercase().next().unwrap();
                    }
                }
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
                priority: 63,
            })
        }

        lints
    }
}

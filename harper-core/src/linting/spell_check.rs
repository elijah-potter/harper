use hashbrown::{HashMap, HashSet};

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
                suggest_correct_spelling(&word, 10, 3, &self.dictionary)
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

            possibilities.sort_by_cached_key(|v| {
                let mut key_dist = usize::MAX;

                for (o, n) in v.iter().zip(word_chars.iter()) {
                    if o != n {
                        key_dist = key_distance(*o, *n)
                            .map(|v| v as usize)
                            .unwrap_or(usize::MAX);
                        break;
                    }
                }

                // The error is likely by omission
                if key_dist > 2 {
                    usize::MAX - v.len()
                }
                // The error is likely by replacement
                else {
                    key_dist
                }
            });

            possibilities.sort_by_key(|v| {
                if self.dictionary.is_common_word(v) {
                    0
                } else {
                    1
                }
            });

            possibilities.shrink_to(5);

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

/// Calculate the approximate distance between two letters on a querty keyboard
fn key_distance(key_a: char, key_b: char) -> Option<f32> {
    let a = key_location(key_a)?;
    let b = key_location(key_b)?;

    Some(((a.0 - b.0) * (a.1 - b.1)).sqrt())
}

/// Calculate the approximate position of a letter on a querty keyboard
fn key_location(key: char) -> Option<(f32, f32)> {
    let keys = "1234567890qwertyuiopasdfghjklzxcvbnm";

    let idx = keys.find(key)?;

    // The starting index of each row of the keyboard
    let mut resets = [0, 10, 20, 29].into_iter().enumerate().peekable();
    // The amount each row is offset (on my keyboard at least)
    let offsets = [0.0, 0.5, 0.75, 1.25];

    while let Some((r_idx, reset)) = resets.next() {
        if idx >= reset {
            if let Some((_, n_reset)) = resets.peek() {
                if idx < *n_reset {
                    return Some(((idx - reset) as f32 + offsets[r_idx], r_idx as f32));
                }
            } else {
                return Some(((idx - reset) as f32 + offsets[r_idx], r_idx as f32));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::key_location;

    #[test]
    fn correct_q_pos() {
        assert_eq!(key_location('q'), Some((0.5, 1.0)))
    }

    #[test]
    fn correct_a_pos() {
        assert_eq!(key_location('a'), Some((0.75, 2.0)))
    }

    #[test]
    fn correct_g_pos() {
        assert_eq!(key_location('g'), Some((4.75, 2.0)))
    }
}

use itertools::Itertools;

use crate::{Document, Lint, LintKind, Linter, Suggestion, TokenStringExt};

#[derive(Debug, Default)]
pub struct AnA;

impl Linter for AnA {
    fn lint(&mut self, document: &Document) -> Vec<crate::Lint> {
        let mut lints = Vec::new();

        for (first, second) in document.iter_words().tuple_windows() {
            let chars_first = document.get_span_content(first.span);
            let chars_second = document.get_span_content(second.span);

            let is_a_an = match chars_first {
                ['a'] => Some(true),
                ['a', 'n'] => Some(false),
                _ => None
            };

            let Some(a_an) = is_a_an else {
                continue;
            };

            let should_be_a_an = !starts_with_vowel(chars_second);

            if a_an != should_be_a_an {
                let replacement = match a_an {
                    true => vec!['a', 'n'],
                    false => vec!['a']
                };

                lints.push(Lint {
                    span: first.span,
                    lint_kind: LintKind::Formatting,
                    suggestions: vec![Suggestion::ReplaceWith(replacement)],
                    message: "This is not vocally correct.".to_string(),
                    priority: 31
                })
            }
        }

        lints
    }
}

// Checks whether a provided word begins with a vowel _sound_.
//
// It was produced through trail and error.
// Matches with 99.71% and 99.77% of vowels and non-vowels in the
// Carnegie-Mellon University word -> pronunciation dataset.
fn starts_with_vowel(word: &[char]) -> bool {
    if matches!(
        word,
        [] | ['u', 'k', ..] | ['e', 'u', 'p', 'h', ..] | ['e', 'u', 'g' | 'l' | 'c', ..]
    ) {
        return false;
    }

    if matches!(
        word,
        ['S', 'V', 'G']
            | ['h', 'o', 'u', 'r', ..]
            | ['h', 'o', 'n', ..]
            | ['u', 'n', 'i', 'n' | 'm', ..]
            | ['u', 'n', 'a' | 'u', ..]
            | ['h', 'e', 'r', 'b', ..]
            | ['u', 'r', 'b', ..]
    ) {
        return true;
    }

    if matches!(word, ['u', 'n' | 's', 'i' | 'a' | 'u', ..]) {
        return false;
    }

    if matches!(word, ['u', 'n', ..]) {
        return true;
    }

    if matches!(word, ['u', 'r', 'g', ..]) {
        return true;
    }

    if matches!(
        word,
        ['u', 't' | 'r' | 'n', ..] | ['e', 'u', 'r', ..] | ['u', 'w', ..] | ['u', 's', 'e', ..]
    ) {
        return false;
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u', 'l' | 'd', ..]) {
        return true;
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u' | '-' | 's', ..]) {
        return false;
    }

    if matches!(
        word,
        ['s', 'o', 's']
            | ['r', 'z', ..]
            | ['n', 'g', ..]
            | ['n', 'v', ..]
            | ['x']
            | ['x', 'b', 'o', 'x']
            | ['h', 'e', 'i', 'r', ..]
            | ['h', 'o', 'n', 'o', 'r', ..]
    ) {
        return true;
    }

    if matches!(
        word,
        ['j', 'u' | 'o', 'n', ..] | ['j', 'u', 'r', 'a' | 'i' | 'o', ..]
    ) {
        return false;
    }

    if matches!(word, ['x', '-' | '\'' | '.' | 'o' | 's', ..]) {
        return true;
    }

    matches!(
        word,
        ['a', ..] | ['e', ..] | ['i', ..] | ['o', ..] | ['u', ..]
    )
}

use std::borrow::Cow;

use itertools::Itertools;

use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, TokenStringExt};

#[derive(Debug, Default)]
pub struct AnA;

impl Linter for AnA {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for (first_idx, second_idx) in chunk.iter_word_indices().tuple_windows() {
                // [`TokenKind::Unlintable`] might be semantic words.
                if chunk[first_idx..second_idx].iter_unlintables().count() > 0 {
                    continue;
                }

                let first = chunk[first_idx];
                let second = chunk[second_idx];

                let chars_first = document.get_span_content(first.span);
                let chars_second = document.get_span_content(second.span);
                // Break the second word on hyphens for this lint.
                // Example: "An ML-based" is an acceptable noun phrase.
                let chars_second = chars_second
                    .split(|c| !c.is_alphanumeric())
                    .next()
                    .unwrap_or(chars_second);

                let is_a_an = match chars_first {
                    ['a'] => Some(true),
                    ['a', 'n'] => Some(false),
                    _ => None,
                };

                let Some(a_an) = is_a_an else {
                    continue;
                };

                let should_be_a_an = !starts_with_vowel(chars_second);

                if a_an != should_be_a_an {
                    let replacement = match a_an {
                        true => vec!['a', 'n'],
                        false => vec!['a'],
                    };

                    lints.push(Lint {
                        span: first.span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::ReplaceWith(replacement)],
                        message: "Incorrect indefinite article.".to_string(),
                        priority: 31,
                    })
                }
            }
        }

        lints
    }
}

fn to_lower_word(word: &[char]) -> Cow<'_, [char]> {
    if word.iter().any(|c| c.is_uppercase()) {
        Cow::Owned(
            word.iter()
                .flat_map(|c| c.to_lowercase())
                .collect::<Vec<_>>(),
        )
    } else {
        Cow::Borrowed(word)
    }
}

/// Checks whether a provided word begins with a vowel _sound_.
///
/// It was produced through trail and error.
/// Matches with 99.71% and 99.77% of vowels and non-vowels in the
/// Carnegie-Mellon University word -> pronunciation dataset.
fn starts_with_vowel(word: &[char]) -> bool {
    let is_likely_initialism = word.iter().all(|c| c.is_uppercase());

    if is_likely_initialism && !word.is_empty() {
        return matches!(
            word[0],
            'A' | 'E' | 'F' | 'H' | 'I' | 'L' | 'M' | 'N' | 'O' | 'R' | 'S' | 'X'
        );
    }

    let word = to_lower_word(word);
    let word = word.as_ref();

    if matches!(
        word,
        [] | ['u', 'k', ..]
            | ['e', 'u', 'p', 'h', ..]
            | ['e', 'u', 'g' | 'l' | 'c', ..]
            | ['o', 'n', 'c', 'e']
    ) {
        return false;
    }

    if matches!(word, |['h', 'o', 'u', 'r', ..]| ['h', 'o', 'n', ..]
        | ['u', 'n', 'i', 'n' | 'm', ..]
        | ['u', 'n', 'a' | 'u', ..]
        | ['h', 'e', 'r', 'b', ..]
        | ['u', 'r', 'b', ..])
    {
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

#[cfg(test)]
mod tests {
    use super::AnA;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn detects_html_as_vowel() {
        assert_lint_count("Here is a HTML document.", AnA, 1);
    }

    #[test]
    fn detects_llm_as_vowel() {
        assert_lint_count("Here is a LLM document.", AnA, 1);
    }

    #[test]
    fn detects_llm_hyphen_as_vowel() {
        assert_lint_count("Here is a LLM-based system.", AnA, 1);
    }

    #[test]
    fn capitalized_fourier() {
        assert_lint_count("Then, perform a Fourier transform.", AnA, 0);
    }

    #[test]
    fn once_over() {
        assert_lint_count("give this a once-over.", AnA, 0);
    }

    #[test]
    fn issue_196() {
        assert_lint_count("This is formatted as an `ext4` file system.", AnA, 0);
    }

    #[test]
    fn allows_lowercase_vowels() {
        assert_lint_count("not an error", AnA, 0);
    }

    #[test]
    fn allows_lowercase_consonants() {
        assert_lint_count("not a crash", AnA, 0);
    }

    #[test]
    fn disallows_lowercase_vowels() {
        assert_lint_count("not a error", AnA, 1);
    }

    #[test]
    fn disallows_lowercase_consonants() {
        assert_lint_count("not an crash", AnA, 1);
    }

    #[test]
    fn allows_uppercase_vowels() {
        assert_lint_count("not an Error", AnA, 0);
    }

    #[test]
    fn allows_uppercase_consonants() {
        assert_lint_count("not a Crash", AnA, 0);
    }

    #[test]
    fn disallows_uppercase_vowels() {
        assert_lint_count("not a Error", AnA, 1);
    }

    #[test]
    fn disallows_uppercase_consonants() {
        assert_lint_count("not an Crash", AnA, 1);
    }
}

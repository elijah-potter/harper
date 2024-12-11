use itertools::Itertools;

use super::lint::Suggestion;
use super::{Lint, LintKind, LintSeverity, Linter};
use crate::document::Document;
use crate::{Token, TokenKind, TokenStringExt};

#[derive(Debug, Clone, Copy, Default)]
pub struct SentenceCapitalization;

impl Linter for SentenceCapitalization {
    /// A linter that checks to make sure the first word of each sentence is
    /// capitalized.
    fn lint(&mut self, document: &Document, severity: Option<LintSeverity>) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            // Allows short, label-like comments in code.
            if paragraph.iter_sentences().count() == 1 {
                let only_sentence = paragraph.iter_sentences().next().unwrap();

                if !only_sentence
                    .iter_chunks()
                    .map(|c| c.iter_words().count())
                    .any(|c| c > 5)
                {
                    continue;
                }
            }

            for sentence in paragraph.iter_sentences() {
                if !is_full_sentence(sentence) {
                    continue;
                }

                if let Some(first_word) = sentence.first_non_whitespace() {
                    if !first_word.kind.is_word() {
                        continue;
                    }

                    let letters = document.get_span_content(first_word.span);

                    if let Some(first_letter) = letters.first() {
                        if first_letter.is_alphabetic() && !first_letter.is_uppercase() {
                            lints.push(Lint {
                                span: first_word.span.with_len(1),
                                lint_kind: LintKind::Capitalization,
                                suggestions: vec![Suggestion::ReplaceWith(
                                    first_letter.to_uppercase().collect_vec(),
                                )],
                                priority: 31,
                                message: "This sentence does not start with a capital letter"
                                    .to_string(),
                                severity,
                            })
                        }
                    }
                }
            }
        }

        lints
    }
}

fn is_full_sentence(toks: &[Token]) -> bool {
    let mut has_noun = false;
    let mut has_verb = false;

    for tok in toks {
        if let TokenKind::Word(metadata) = tok.kind {
            if metadata.is_noun() {
                has_noun = true;
            }

            if metadata.is_verb() {
                has_verb = true;
            }
        }
    }

    has_noun && has_verb
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::SentenceCapitalization;

    #[test]
    fn catches_basic() {
        assert_lint_count(
            "there is no way she is not guilty.",
            SentenceCapitalization,
            1,
        )
    }

    #[test]
    fn no_period() {
        assert_lint_count(
            "there is no way she is not guilty",
            SentenceCapitalization,
            1,
        )
    }

    #[test]
    fn two_sentence() {
        assert_lint_count(
            "i have complete conviction in this. she is absolutely guilty",
            SentenceCapitalization,
            2,
        )
    }

    #[test]
    fn start_with_number() {
        assert_lint_count(
            "53 is the length of the longest word.",
            SentenceCapitalization,
            0,
        );
    }

    #[test]
    fn ignores_unlintable() {
        assert_lint_count(
            "[`misspelled_word`] is assumed to be quite small (n < 100). ",
            SentenceCapitalization,
            0,
        )
    }

    #[test]
    fn unphased_unlintable() {
        assert_lint_count(
            "the linter should not be affected by `this` unlintable.",
            SentenceCapitalization,
            1,
        )
    }

    #[test]
    fn unphased_ellipsis() {
        assert_lint_count(
            "the linter should not be affected by... that ellipsis.",
            SentenceCapitalization,
            1,
        )
    }

    #[test]
    fn unphased_comma() {
        assert_lint_count(
            "the linter should not be affected by, that comma.",
            SentenceCapitalization,
            1,
        )
    }

    #[test]
    fn issue_228_allows_labels() {
        assert_lint_count("python lsp (fork of pyright)", SentenceCapitalization, 0)
    }
}
